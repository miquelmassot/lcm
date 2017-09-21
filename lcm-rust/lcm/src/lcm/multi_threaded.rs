use std::{fmt, ptr, slice};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::CString;
use std::io::{Error, ErrorKind, Result};
use std::sync::Mutex;
use std::time::Duration;
use super::{LcmSubscription, handler_callback};
use message::Message;
use ffi::*;

/// A threadsafe version of the LCM instance. Because this can be used on
/// multiple threads, all callbacks must also be threadsafe.
pub struct ThreadsafeLcm {
    lcm: *mut lcm_t,

    // This is a solid type. I'm proud of what I've done here.
    subscriptions:
      Mutex<HashMap<*mut lcm_subscription_t, Box<Box<FnMut(*const lcm_recv_buf_t) + Sync + Send>>>>,
}
unsafe impl Sync for ThreadsafeLcm { }
unsafe impl Send for ThreadsafeLcm { }

impl ThreadsafeLcm {
    /// Creates a new `ThreadsafeLcm` instance.
    ///
    /// ```
    /// use lcm::ThreadsafeLcm;
    /// let mut lcm = ThreadsafeLcm::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        trace!("Creating LCM instance");
        let lcm = unsafe { lcm_create(ptr::null()) };
        match lcm.is_null() {
            true => Err(Error::new(ErrorKind::Other, "Failed to initialize LCM.")),
            false => {
                Ok(ThreadsafeLcm {
                    lcm: lcm,
                    subscriptions: Mutex::new(HashMap::new()),
                })
            }
        }
    }

    pub fn get_fileno(&self) -> ::std::os::raw::c_int {
        unsafe { lcm_get_fileno(self.lcm) }
    }

    /// Subscribes a callback to a particular topic.
    ///
    /// If the callback function panics, then it will poison the inner mutex
    /// which will then cause all other subscription operations to panic.
    ///
    /// ```
    /// # use lcm::ThreadsafeLcm;
    /// let mut lcm = ThreadsafeLcm::new().unwrap();
    /// lcm.subscribe("GREETINGS", |name: String| println!("Hello, {}!", name) );
    /// ```
    pub fn subscribe<M, F>(&self, channel: &str, mut callback: F) -> LcmSubscription
        where M: Message,
              F: FnMut(M) + Sync + Send + 'static
    {
        trace!("Subscribing handler to channel {}", channel);

        let channel = CString::new(channel).unwrap();

        // This is a double box for a reason
        let handler = {
            let handler: Box<FnMut(*const lcm_recv_buf_t) + Sync + Send> =
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
                          Some(handler_callback),
                          &*handler as *const _  as *mut _)
        };

        let mut subs = self.subscriptions.lock().unwrap();
        assert!(!subs.contains_key(&subscription));
        subs.insert(subscription, handler);

        LcmSubscription { subscription }
    }

    /// Unsubscribes a message handler.
    ///
    /// ```
    /// # use lcm::ThreadsafeLcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// # let mut lcm = ThreadsafeLcm::new().unwrap();
    /// let subscription = lcm.subscribe("GREETINGS", handler_function);
    /// // ...
    /// lcm.unsubscribe(subscription);
    /// ```
    pub fn unsubscribe(&self, subscription: LcmSubscription) -> Result<()> {
        trace!("Unsubscribing handler {:?}", subscription.subscription);
        let result = unsafe { lcm_unsubscribe(self.lcm, subscription.subscription) };

        match result {
            0 => {
                self.subscriptions.lock().unwrap().remove(&subscription.subscription);
                Ok(())
            },
            _ => Err(Error::new(ErrorKind::Other, "LCM: Failed to unsubscribe")),
        }
    }

    /// Publishes a message on the specified channel.
    ///
    /// ```
    /// # use lcm::ThreadsafeLcm;
    /// let mut lcm = ThreadsafeLcm::new().unwrap();
    /// lcm.publish("GREETINGS", &"Charles".to_string()).unwrap();
    /// ```
    pub fn publish<M>(&self, channel: &str, message: &M) -> Result<()>
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
    /// # use lcm::ThreadsafeLcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// let mut lcm = ThreadsafeLcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// loop {
    /// # break;
    ///     lcm.handle().unwrap();
    /// }
    /// ```
    pub fn handle(&self) -> Result<()> {
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
    /// # use lcm::ThreadsafeLcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// let mut lcm = ThreadsafeLcm::new().unwrap();
    /// lcm.subscribe("POSITION", handler_function);
    /// let wait_dur = Duration::from_millis(100);
    /// loop {
    /// # break;
    ///     lcm.handle_timeout(Duration::from_millis(1000)).unwrap();
    /// }
    /// ```
    pub fn handle_timeout(&self, timeout: Duration) -> Result<()> {
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
    /// # use lcm::ThreadsafeLcm;
    /// # let handler_function = |name: String| println!("Hello, {}!", name);
    /// # let mut lcm = ThreadsafeLcm::new().unwrap();
    /// let subscription = lcm.subscribe("POSITION", handler_function);
    /// lcm.subscription_set_queue_capacity(subscription, 30);
    /// ```
    pub fn subscription_set_queue_capacity(&self, subscription: LcmSubscription, num_messages: usize) {
        let handler = subscription.subscription;
        let num_messages = num_messages as _;
        unsafe { lcm_subscription_set_queue_capacity(handler, num_messages) };
    }



}

impl Drop for ThreadsafeLcm {
    fn drop(&mut self) {
        trace!("Destroying Lcm instance");
        unsafe { lcm_destroy(self.lcm) };
    }
}

impl fmt::Debug for ThreadsafeLcm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ThreadsafeLcm {{ lcm: {:?} }}", self.lcm)
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
        let _lcm = ThreadsafeLcm::new().unwrap();
    }

    #[test]
    fn test_subscribe() {
        let mut lcm = ThreadsafeLcm::new().unwrap();
        lcm.subscribe("channel", |_: String| {});
        let subs_len = lcm.subscriptions.lock().unwrap().len();
        assert_eq!(subs_len, 1);
    }

    #[test]
    fn test_unsubscribe() {
        let mut lcm = ThreadsafeLcm::new().unwrap();
        let sub = lcm.subscribe("channel", |_: String| {});
        lcm.unsubscribe(sub).unwrap();
        let subs_len = lcm.subscriptions.lock().unwrap().len();
        assert_eq!(subs_len, 0);
    }
}
