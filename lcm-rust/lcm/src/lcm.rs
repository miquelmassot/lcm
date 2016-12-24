use std::io::{Error, ErrorKind, Result};
use std::ffi::{CString};
use libc;
use message::Message;
use std::cmp::Ordering;
use std::ptr;
use std::boxed::Box;
use std::rc::Rc;
use std::ops::Deref;

enum CLcm {}

/// TODO: Struct documentation.
pub struct Lcm {
    lcm: *mut CLcm,
    subscriptions: Vec<Rc<LcmSubscription>>
}

type CLcmHandler = extern fn(*const CLcmRecvBuf, *const libc::c_char, *const libc::c_void);

#[derive(Eq, PartialEq, Hash)]
enum CLcmSubscription {}
pub struct LcmSubscription {
    subscription: *mut CLcmSubscription,
    handler: Box<FnMut(*const CLcmRecvBuf)>
}

#[repr(C)]
#[derive(Debug)]
struct CLcmRecvBuf {
    data: *const libc::c_void,
    data_size: libc::uint32_t,
    recv_utime: libc::int64_t,
    lcm: *const CLcm
}

#[link(name = "lcm")]
extern {
    fn lcm_create(provider: *const libc::c_char) -> *mut CLcm;

    fn lcm_destroy(lcm: *mut CLcm);

    fn lcm_get_fileno(lcm: *mut CLcm) -> libc::c_int;

    fn lcm_subscribe(lcm: *mut CLcm, channel: *const libc::c_char, handler: CLcmHandler, userdata: *const libc::c_void) -> *mut CLcmSubscription;

    fn lcm_unsubscribe(lcm: *mut CLcm, handler: *const CLcmSubscription) -> libc::c_int;

    fn lcm_publish(lcm: *mut CLcm, channel: *const libc::c_char, data: *const libc::c_void, datalen: libc::c_uint) -> libc::c_int;

    fn lcm_handle(lcm: *mut CLcm) -> libc::c_int;

    fn lcm_handle_timeout(lcm: *mut CLcm, timeout_millis: libc::c_int) -> libc::c_int;

    fn lcm_subscription_set_queue_capacity(handler: *const CLcmSubscription, num_messages: libc::c_int) -> libc::c_int;
}

impl Lcm {
    /// Creates a new `Lcm` instance.
    ///
    /// ```
    /// use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// ```
    pub fn new() -> Result<Lcm> {
        let lcm = unsafe { lcm_create(ptr::null()) };
        match lcm.is_null() {
            true => Err(Error::new(ErrorKind::Other, "Failed to initialize LCM.")),
            false => Ok(Lcm{
                lcm: lcm,
                subscriptions: Vec::new()
            })
        }
    }

    pub fn get_fileno(&self) -> libc::c_int {
        unsafe { lcm_get_fileno(self.lcm) }
    }

    /// Subscribes a callback to a particular topic.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// use std::sync::mpsc::channel;
    ///
    /// let mut lcm = Lcm::new().unwrap();
    /// let (tx, rx) = channel::<exlcm::ExampleT>();
    /// lcm.subscribe("POSITION", |pos| { tx.send(pos).unwrap(); }
    /// ```
    pub fn subscribe<M, F>(&mut self, channel: &str, mut callback: Box<F>) -> Rc<LcmSubscription>
        where M: Message + Default, F: FnMut(M) + 'static {
        let channel = CString::new(channel).unwrap();

        let handler = Box::new(move |rbuf: *const CLcmRecvBuf| {
            let mut msg = M::default();
            let buf = unsafe {
                let ref rbuf = *rbuf;
                let data = rbuf.data as *mut u8;
                let len = rbuf.data_size as usize;
                Vec::from_raw_parts(data, len, len)
            };
            match msg.decode_with_hash(&mut buf.as_slice()) {
                Ok(()) => callback(msg),
                Err(_) => {}
            }
        });

        let mut subscription = Rc::new(LcmSubscription{
            subscription: ptr::null_mut(),
            handler: handler
        });

        let user_data = (subscription.deref() as *const LcmSubscription) as *const libc::c_void;

        let c_subscription = unsafe { lcm_subscribe(self.lcm, channel.as_ptr(), Lcm::handler_callback::<M>, user_data) };

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
        let result = unsafe { lcm_unsubscribe(self.lcm, handler.subscription) };
        match result {
            0 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "LCM: Failed to unsubscribe"))
        }
    }

    /// Publishes a message on the specified channel.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    ///
    /// let mut my_data = exlcm::ExampleT::new();
    /// my_data.timestamp = 0;
    /// my_data.position[0] = 1.0;
    /// my_data.position[1] = 2.0;
    /// my_data.position[2] = 3.0;
    ///
    /// lcm.publish("POSITION", &my_data).unwrap();
    /// ```
    pub fn publish(&mut self, channel: &str, message: &Message) -> Result<()> {
        let channel = CString::new(channel).unwrap();
        let buffer = message.encode_with_hash()?;
        let datalen = buffer.len() as libc::c_uint;
        let result = unsafe { lcm_publish(self.lcm, channel.as_ptr(), buffer.as_ptr() as *mut libc::c_void, datalen) };
        match result {
            0 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "LCM Error"))
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
            _ => Err(Error::new(ErrorKind::Other, "LCM Error"))
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
    pub fn handle_timeout(&mut self, timeout_millis: i64) -> Result<()> {
        let timeout = timeout_millis as libc::c_int;
        let result = unsafe { lcm_handle_timeout(self.lcm, timeout) };
        match result.cmp(&0) {
            Ordering::Less => Err(Error::new(ErrorKind::Other, "LCM Error")),
            Ordering::Equal => Err(Error::new(ErrorKind::Other, "LCM Timeout")),
            Ordering::Greater => Ok(())
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
        let num_messages = num_messages as libc::c_int;
        unsafe { lcm_subscription_set_queue_capacity(handler, num_messages) };
    }



    extern fn handler_callback<M>(rbuf: *const CLcmRecvBuf,
                                  _ /*channel*/: *const libc::c_char,
                                  user_data: *const libc::c_void)
        where M: Message {

        let sub = user_data as *mut LcmSubscription;
        let sub = unsafe { &mut *sub };
        (sub.handler)(rbuf);
    }
}

impl Drop for Lcm {
    fn drop(&mut self) {
        unsafe { lcm_destroy(self.lcm) };
    }
}