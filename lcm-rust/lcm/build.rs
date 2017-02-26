extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=lcm");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .no_unstable_rust()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("lcm-bindings.rs"))
        .expect("Couldn't write bindings");
}
