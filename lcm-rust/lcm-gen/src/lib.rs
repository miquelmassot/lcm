extern crate glob;

use glob::glob;
use std::path::PathBuf;
use std::env;
use std::process::Command;

pub struct LcmGen {
    files: Vec<PathBuf>,
    out_dir: PathBuf
}

impl LcmGen {
    pub fn new() -> Self {
        LcmGen {
            files: Vec::new(),
            out_dir: env::var("OUT_DIR").unwrap().into()
        }
    }

    pub fn output_directory(&mut self, path: PathBuf) -> &Self {
        self.out_dir = path;
        self
    }

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

    pub fn add_file(&mut self, path: PathBuf) -> &Self {
        self.files.push(path);
        self
    }

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
