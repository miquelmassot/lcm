extern crate lcm_gen;

use lcm_gen::LcmGen;
use std::path::PathBuf;

fn main() {
    // lcm types are in a sibling directory: ../types/
    let mut lcm_source_dir : PathBuf = env!("CARGO_MANIFEST_DIR").into();
    lcm_source_dir.pop();
    lcm_source_dir.push("types");

    let mut dest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dest.push("src");

    let mut gen = LcmGen::new();
    gen.output_directory(&dest);
    gen.add_directory(lcm_source_dir);
    gen.run();
}
