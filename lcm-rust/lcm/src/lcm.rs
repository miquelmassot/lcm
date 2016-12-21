use std::io::{Error, ErrorKind, Result};
use std::ffi::CString;
use libc;
use message::Message;

enum CLcm {}
pub struct Lcm(*mut CLcm);

#[link(name = "lcm")]
extern {
    fn lcm_create(provider: *const libc::c_char) -> *mut CLcm;
    fn lcm_destroy(lcm: *mut CLcm);

    fn lcm_publish(lcm: *mut CLcm, channel: *const libc::c_char, data: *const libc::c_void, datalen: usize) -> libc::c_int;
}

impl Lcm {
    pub fn new() -> Lcm {
        let lcm = unsafe { lcm_create(0 as *mut libc::c_char) };
        Lcm(lcm)
    }

    pub fn publish(&mut self, channel: &str, message: &Message) -> Result<()> {
        let channel = CString::new(channel).unwrap();
        let buffer = message.encode_with_hash()?;
        let datalen = buffer.len();
        unsafe {
            let result = lcm_publish(self.0, channel.as_ptr(), buffer.as_ptr() as *mut libc::c_void, datalen);
            match result {
                0 => Ok(()),
                _ => Err(Error::new(ErrorKind::Other, "LCM Error"))
            }
        }
    }
}

impl Drop for Lcm {
    fn drop(&mut self) {
        unsafe {
            lcm_destroy(self.0);
        }
    }
}