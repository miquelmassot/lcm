#![allow(non_camel_case_types)]

use std::os::raw::{c_int, c_uint, c_char, c_void};

pub enum lcm_t { }
pub enum lcm_subscription_t { }

#[repr(C)]
pub struct lcm_recv_buf_t {
    pub data: *mut c_void,
    pub data_size: u32,
    pub recv_utime: i64,
    pub lcm: *mut lcm_t,
}

pub type lcm_msg_handler_t = Option<unsafe extern "C" fn(rbuf: *const lcm_recv_buf_t,
                                                         channel: *const c_char,
                                                         user_data: *mut c_void)>;

#[link(name = "lcm")]
extern "C" {
    pub fn lcm_create(provider: *const c_char) -> *mut lcm_t;
    pub fn lcm_destroy(lcm: *mut lcm_t);
    pub fn lcm_get_fileno(lcm: *mut lcm_t) -> c_int;

    pub fn lcm_subscribe(lcm: *mut lcm_t, channel: *const c_char, handler: lcm_msg_handler_t, user_data: *mut c_void) -> *mut lcm_subscription_t;
    pub fn lcm_unsubscribe(lcm: *mut lcm_t, handler: *mut lcm_subscription_t) -> c_int;

    pub fn lcm_publish(lcm: *mut lcm_t, channel: *const c_char, data: *const c_void, datalen: c_uint) -> c_int;

    pub fn lcm_handle(lcm: *mut lcm_t) -> c_int;
    pub fn lcm_handle_timeout(lcm: *mut lcm_t, timout_millis: c_int) -> c_int;

    pub fn lcm_subscription_set_queue_capacity(handler: *mut lcm_subscription_t, num_messages: c_int) -> c_int;
}
