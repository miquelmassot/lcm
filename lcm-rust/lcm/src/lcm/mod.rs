use std::ffi::CStr;
use std::panic::catch_unwind;
use ffi::*;

mod single_threaded;
pub use self::single_threaded::Lcm;

mod multi_threaded;
pub use self::multi_threaded::ThreadsafeLcm;


/// Represents an LCM subscription. Only useful for unsubscribing.
#[derive(Debug)]
pub struct LcmSubscription {
    subscription: *mut lcm_subscription_t,
}
unsafe impl Sync for LcmSubscription { }
unsafe impl Send for LcmSubscription { }

extern "C" fn handler_callback(rbuf: *const lcm_recv_buf_t,
                               chan: *const ::std::os::raw::c_char,
                               user_data: *mut ::std::os::raw::c_void)
{
    let res = catch_unwind(|| {
        trace!("Received data on channel {:?}", unsafe { CStr::from_ptr(chan) });
        let callback = user_data as *mut Box<FnMut(*const lcm_recv_buf_t)>;
        unsafe { (*(*callback))(rbuf); }
    });

    if res.is_err() {
        error!("Callback function panicked!");
    }
}

