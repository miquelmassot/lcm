//! Crate docs
//!
//! Example:
//!
//! ```no_run
//! // build.rs
//! use std::env;
//! use std::path::PathBuf;
//!
//! fn main() {
//!     // lcm types are in a sibling directory: ../types/
//!     let mut lcm_source_dir : PathBuf = env!("CARGO_MANIFEST_DIR").into();
//!     lcm_source_dir.pop();
//!     lcm_source_dir.push("types");
//!
//!     println!("cargo:rerun-if-changed={}", lcm_source_dir.display());
//!
//!     lcm_gen::LcmGen::new()
//!         .add_directory(lcm_source_dir)
//!         .run();
//! }
//! ```

extern crate glob;

use glob::glob;
use std::path::PathBuf;
use std::env;
use std::process::Command;

/// This struct is used to configure options for, and then run, the `lcm-gen` command.
pub struct LcmGen {
    files: Vec<PathBuf>,
    out_dir: PathBuf
}

impl LcmGen {
    /// Constructs a new `lcm-gen` command.
    pub fn new() -> Self {
        LcmGen {
            files: Vec::new(),
            out_dir: env::var("OUT_DIR").unwrap().into()
        }
    }

    /// Sets the output directory. The default is `env::var("OUT_DIR")`.
    pub fn output_directory(&mut self, path: PathBuf) -> &Self {
        self.out_dir = path;
        self
    }

    /// Adds a file to the list of arguments to pass to `lcm-gen`
    pub fn add_file(&mut self, path: PathBuf) -> &Self {
        self.files.push(path);
        self
    }

    /// Recursively adds all the `.lcm` files from a directory.
    pub fn add_directory(&mut self, path: PathBuf) -> &Self {
        let pattern = format!("{}/**/*.lcm", path.display());
        let paths =
            glob(pattern.as_str()).unwrap()
            .filter_map(Result::ok);
        for path in paths {
            self.files.push(path);
        }
        self
    }

    /// Runs `lcm-gen --rust --rast-path={}` on each `.lcm` file that was added.
    pub fn run(&self) {
        let mut cmd = Command::new("lcm-gen");
        for path in &self.files {
            cmd.arg(path);
        }
        cmd
            .arg("--rust")
            .arg(format!("--rust-path={}", self.out_dir.display()));

        let status = cmd.status().unwrap();
        assert!(status.success());
    }
}
