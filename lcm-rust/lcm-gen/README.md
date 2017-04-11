# Generate LCM messages at build time

[![crates.io](http://meritbadge.herokuapp.com/lcm_gen)](https://crates.io/crates/lcm_gen)
[![Build Status](https://travis-ci.org/adeschamps/lcm.svg?branch=rust)](https://travis-ci.org/adeschamps/lcm)

This crate lets you invoke `lcm-gen` as part of a Cargo build script.
It requires that you have `lcm-gen` on your PATH, and that it supports emitting Rust code with the `--rust` and `--rust-path` options.
