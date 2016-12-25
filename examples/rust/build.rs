extern crate lcm_gen;

use std::path::PathBuf;

fn main() {
    // lcm types are in a sibling directory: ../types/
    let mut lcm_source_dir : PathBuf = env!("CARGO_MANIFEST_DIR").into();
    lcm_source_dir.pop();
    lcm_source_dir.push("types");

    println!("cargo:rerun-if-changed={}", lcm_source_dir.display());

    lcm_gen::LcmGen::new()
        .add_directory(lcm_source_dir)
        .run();
}
