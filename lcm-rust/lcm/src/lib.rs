//! From the [LCM Homepage](http://lcm-proj.github.io/):
//! >
//! LCM is a set of libraries and tools for message passing and data marshalling,
//! targeted at real-time systems where high-bandwidth and low latency are critical.
//! It provides a publish/subscribe message passing model
//! and automatic marshalling/unmarshalling code generation
//! with bindings for applications in a variety of programming languages.
//!
//! This crate provides Rust support for LCM.
//! See also the `lcm-gen` crate, for running `lcmgen` at build time.

extern crate byteorder;
#[macro_use]
extern crate log;

mod ffi;

mod lcm;
pub use lcm::{Lcm, ThreadsafeLcm, LcmSubscription};

mod message;
pub use message::Message;
