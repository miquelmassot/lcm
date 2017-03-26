# Generate LCM messages at build time

[![crates.io](http://meritbadge.herokuapp.com/lcm_gen)](https://crates.io/crates/lcm_gen)

This crate lets you invoke `lcm-gen` as part of a Cargo build script.
It requires that you have `lcm-gen` on your PATH, and that it supports emitting Rust code with the `--rust` and `--rust-path` options.
