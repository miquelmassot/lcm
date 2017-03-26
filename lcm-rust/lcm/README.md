# LCM Bindings for Rust

[![crates.io](http://meritbadge.herokuapp.com/lcm)](https://crates.io/crates/lcm)

This crate provides Rust bindings for [LCM](http://lcm-proj.github.io).

To generate Rust code for your LCM message definitions, you must use [this fork] of `lcm-gen`.
See also the [lcm_gen](https://crates.io/crates/lcm_gen) crate to integrate this into a Cargo build.
While the fork of `lcm-gen` is required for generating Rust code, it is not required to use this crate, since it only wraps the official C library.
