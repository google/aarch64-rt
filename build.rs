// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    // Write linker script to out directory, and add that to the search path. We can't actually make
    // the linker use it, only a binary can do that.
    let image_ld = include_bytes!("image.ld");
    File::create(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("image.ld"))
        .unwrap()
        .write_all(image_ld)
        .unwrap();

    println!("cargo::rustc-link-search={}", env::var("OUT_DIR").unwrap());
    println!("cargo::rerun-if-changed=image.ld");
}
