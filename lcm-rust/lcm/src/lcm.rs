use std::io::{Error, ErrorKind, Result};
use std::ffi::CString;
use std::cmp::Ordering;
use std::time::Duration;
use std::{ptr, slice, fmt};
use std::collections::HashMap;
use ffi::*;
use message::Message;

/// An LCM instance that handles publishing and subscribing,
/// as well as encoding and decoding messages.
pub struct Lcm {
    lcm: *mut lcm_t,

    // This has to be Box<Box<..>> due to the fact that &Box<..> is what needs to be sent
    // across the FFI boundary. Sending &*Box<..> results in not being able to
    // reconstruct the trait object when it returns to the Rust side of the boundary.
    subscriptions: HashMap<*mut lcm_subscription_t, Box<Box<FnMut(*const lcm_recv_buf_t)>>>,
}


#[derive(Debug)]
pub struct LcmSubscription {
    subscription: *mut lcm_subscription_t,
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
                    subscriptions: HashMap::new(),
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
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.subscribe("GREETINGS", |name: String| println!("Hello, {}!", name) );
    /// ```
    pub fn subscribe<M, F>(&mut self, channel: &str, mut callback: F) -> LcmSubscription
        where M: Message,
              F: FnMut(M) + 'static
    {
        trace!("Subscribing handler to channel {}", channel);

        let channel = CString::new(channel).unwrap();

        // This is a double box for a reason
        let handler = {
            let handler: Box<FnMut(*const lcm_recv_buf_t)> =
                Box::new(move |rbuf: *const lcm_recv_buf_t| {
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
            Box::new(handler)
        };

        let subscription = unsafe {
            lcm_subscribe(self.lcm,
                          channel.as_ptr(),
                          Some(Lcm::handler_callback::<M>),
                          &*handler as *const _  as *mut _)
        };

        assert!(!self.subscriptions.contains_key(&subscription));
        self.subscriptions.insert(subscription, handler);

        LcmSubscription { subscription }
    }

    /// Unsubscribes a message handler.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// # let mut lcm = Lcm::new().unwrap();
    /// let subscription = lcm.subscribe("GREETINGS", handler_function);
    /// // ...
    /// lcm.unsubscribe(subscription);
    /// ```
    pub fn unsubscribe(&mut self, subscription: LcmSubscription) -> Result<()> {
        trace!("Unsubscribing handler {:?}", subscription.subscription);
        let result = unsafe { lcm_unsubscribe(self.lcm, subscription.subscription) };

        self.subscriptions.remove(&subscription.subscription);

        match result {
            0 => {
                self.subscriptions.remove(&subscription.subscription);
                Ok(())
            },
            _ => Err(Error::new(ErrorKind::Other, "LCM: Failed to unsubscribe")),
        }
    }

    /// Publishes a message on the specified channel.
    ///
    /// ```
    /// # use lcm::Lcm;
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.publish("GREETINGS", &"Charles".to_string()).unwrap();
    /// ```
    pub fn publish<M>(&mut self, channel: &str, message: &M) -> Result<()>
        where M: Message
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
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// loop {
    /// # break;
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
    /// # use std::time::Duration;
    /// # use lcm::Lcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// let mut lcm = Lcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// let wait_dur = Duration::from_millis(100);
    /// loop {
    /// # break;
    ///     lcm.handle_timeout(Duration::from_millis(1000)).unwrap();
    /// }
    /// ```
    pub fn handle_timeout(&mut self, timeout: Duration) -> Result<()> {
        let result = unsafe { lcm_handle_timeout(self.lcm, (timeout.as_secs() * 1000) as i32 + (timeout.subsec_nanos() / 1000_000) as i32) };
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
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// # let mut lcm = Lcm::new().unwrap();
    /// let subscription = lcm.subscribe("POSITION", handler_function);
    /// lcm.subscription_set_queue_capacity(subscription, 30);
    /// ```
    pub fn subscription_set_queue_capacity(&self, subscription: LcmSubscription, num_messages: usize) {
        let handler = subscription.subscription;
        let num_messages = num_messages as _;
        unsafe { lcm_subscription_set_queue_capacity(handler, num_messages) };
    }



    extern "C" fn handler_callback<M>(rbuf: *const lcm_recv_buf_t,
                                      _: *const ::std::os::raw::c_char,
                                      user_data: *mut ::std::os::raw::c_void)
        where M: Message
    {
        trace!("Received data");
        let callback = user_data as *mut Box<FnMut(*const lcm_recv_buf_t)>;
        unsafe { (*(*callback))(rbuf); }
    }
}

impl Drop for Lcm {
    fn drop(&mut self) {
        trace!("Destroying Lcm instance");
        unsafe { lcm_destroy(self.lcm) };
    }
}

impl fmt::Debug for Lcm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lcm {{ lcm: {:?} }}", self.lcm)
    }
}

#[cfg(test)]
///
/// Tests
///
mod test {
    use super::*;

    #[test]
    fn initialized() {
        let _lcm = Lcm::new().unwrap();
    }

    #[test]
    fn test_subscribe() {
        let mut lcm = Lcm::new().unwrap();
        lcm.subscribe("channel", |_: String| {});
        assert_eq!(lcm.subscriptions.len(), 1);
    }

    #[test]
    fn test_unsubscribe() {
        let mut lcm = Lcm::new().unwrap();
        let sub = lcm.subscribe("channel", |_: String| {});
        lcm.unsubscribe(sub).unwrap();
        assert_eq!(lcm.subscriptions.len(), 0);
    }
}
