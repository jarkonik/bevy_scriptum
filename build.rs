use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rustc-link-lib=z"); // TODO: if features Ruby
}
