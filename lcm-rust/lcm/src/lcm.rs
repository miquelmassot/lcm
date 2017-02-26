#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
include!(concat!(env!("OUT_DIR"), "/lcm-bindings.rs"));

use std::io::{Error, ErrorKind, Result};
use std::ffi::CString;
use message::Message;
use std::cmp::Ordering;
use std::ptr;
use std::boxed::Box;
use std::rc::Rc;
use std::ops::Deref;
use std::slice;
use time::Duration;

/// An LCM instance that handles publishing and subscribing,
/// as well as encoding and decoding messages.
pub struct Lcm {
    lcm: *mut lcm_t,
    subscriptions: Vec<Rc<LcmSubscription>>,
}


pub struct LcmSubscription {
    subscription: *mut lcm_subscription_t,
    handler: Box<FnMut(*const lcm_recv_buf_t)>,
}


impl Lcm {
    /// Creates a new `Lcm` instance.
    ///
    /// ```
    /// use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// ```
    pub fn new() -> Result<Lcm> {
        trace!("Creating LCM instance");
        let lcm = unsafe { lcm_create(ptr::null()) };
        match lcm.is_null() {
            true => Err(Error::new(ErrorKind::Other, "Failed to initialize LCM.")),
            false => {
                Ok(Lcm {
                    lcm: lcm,
                    subscriptions: Vec::new(),
                })
            }
        }
    }

    pub fn get_fileno(&self) -> ::std::os::raw::c_int {
        unsafe { lcm_get_fileno(self.lcm) }
    }

    /// Subscribes a callback to a particular topic.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// use std::sync::mpsc::channel;
    ///
    /// let mut lcm = Lcm::new().unwrap();
    /// let (tx, rx) = channel::<exlcm::Example>();
    /// lcm.subscribe("POSITION", |pos| { tx.send(pos).unwrap(); }
    /// ```
    pub fn subscribe<M, F>(&mut self, channel: &str, mut callback: Box<F>) -> Rc<LcmSubscription>
        where M: Message + Default,
              F: FnMut(M) + 'static
    {
        trace!("Subscribing handler to channel {}", channel);

        let channel = CString::new(channel).unwrap();

        let handler = Box::new(move |rbuf: *const lcm_recv_buf_t| {
            trace!("Running handler");
            let mut buf = unsafe {
                let ref rbuf = *rbuf;
                let data = rbuf.data as *mut u8;
                let len = rbuf.data_size as usize;
                slice::from_raw_parts(data, len)
            };
            trace!("Decoding buffer: {:?}", buf);
            match M::decode_with_hash(&mut buf) {
                Ok(msg) => callback(msg),
                Err(_) => error!("Failed to decode buffer: {:?}", buf),
            }
        });

        let mut subscription = Rc::new(LcmSubscription {
            subscription: ptr::null_mut(),
            handler: handler,
        });

        let user_data = (subscription.deref() as *const _) as *mut _;

        let c_subscription = unsafe {
            lcm_subscribe(self.lcm,
                          channel.as_ptr(),
                          Some(Lcm::handler_callback::<M>),
                          user_data)
        };

        Rc::get_mut(&mut subscription).unwrap().subscription = c_subscription;
        self.subscriptions.push(subscription.clone());

        subscription
    }

    /// Unsubscribes a message handler.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// let handler = lcm.subscribe("POSITION", handler_function);
    /// // ...
    /// lcm.unsubscribe(handler);
    /// ```
    pub fn unsubscribe(&mut self, handler: LcmSubscription) -> Result<()> {
        trace!("Unsubscribing handler {:?}", handler.subscription);
        let result = unsafe { lcm_unsubscribe(self.lcm, handler.subscription) };
        match result {
            0 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "LCM: Failed to unsubscribe")),
        }
    }

    /// Publishes a message on the specified channel.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    ///
    /// let mut my_data = exlcm::Example::new();
    /// my_data.timestamp = 0;
    /// my_data.position[0] = 1.0;
    /// my_data.position[1] = 2.0;
    /// my_data.position[2] = 3.0;
    ///
    /// lcm.publish("POSITION", &my_data).unwrap();
    /// ```
    pub fn publish<M>(&mut self, channel: &str, message: &M) -> Result<()>
        where M: Message + Sized
    {
        let channel = CString::new(channel).unwrap();
        let buffer = message.encode_with_hash()?;
        let result = unsafe {
            lcm_publish(self.lcm,
                        channel.as_ptr(),
                        buffer.as_ptr() as *mut _,
                        buffer.len() as _)
        };
        match result {
            0 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "LCM Error")),
        }
    }

    /// Waits for and dispatches the next incoming message.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// loop {
    ///     lcm.handle().unwrap();
    /// }
    /// ```
    pub fn handle(&mut self) -> Result<()> {
        let result = unsafe { lcm_handle(self.lcm) };
        match result {
            0 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "LCM Error")),
        }
    }

    /// Waits for and dispatches the next incoming message, up to a time limit.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// loop {
    ///     lcm.handle_timeout(1000).unwrap();
    /// }
    /// ```
    pub fn handle_timeout(&mut self, timeout: &Duration) -> Result<()> {
        let result = unsafe { lcm_handle_timeout(self.lcm, timeout.num_milliseconds() as i32) };
        match result.cmp(&0) {
            Ordering::Less => Err(Error::new(ErrorKind::Other, "LCM Error")),
            Ordering::Equal => Err(Error::new(ErrorKind::Other, "LCM Timeout")),
            Ordering::Greater => Ok(()),
        }
    }

    /// Adjusts the maximum number of received messages that can be queued up for a subscription.
    /// The default is `30`.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// let handler = lcm.subscribe("POSITION", handler_function);
    /// lcm.subscription_set_queue_capacity(handler, 30);
    /// ```
    pub fn subscription_set_queue_capacity(handler: Rc<LcmSubscription>, num_messages: usize) {
        let handler = handler.subscription;
        let num_messages = num_messages as _;
        unsafe { lcm_subscription_set_queue_capacity(handler, num_messages) };
    }



    extern "C" fn handler_callback<M>(rbuf: *const lcm_recv_buf_t,
                                      _: *const ::std::os::raw::c_char,
                                      user_data: *mut ::std::os::raw::c_void)
        where M: Message
    {
        trace!("Received data");
        let sub = user_data as *mut LcmSubscription;
        let sub = unsafe { &mut *sub };
        (sub.handler)(rbuf);
    }
}

impl Drop for Lcm {
    fn drop(&mut self) {
        trace!("Destroying Lcm instance");
        unsafe { lcm_destroy(self.lcm) };
    }
}
