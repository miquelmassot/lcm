extern crate lcm_gen;

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut lcm_source_dir = PathBuf::from(manifest_dir);
    lcm_source_dir.pop();
    lcm_source_dir.push("types");

    println!("cargo:rerun-if-changed={}", lcm_source_dir.display());

    lcm_gen::LcmGen::new()
        .add_directory(lcm_source_dir)
        .run();
}
