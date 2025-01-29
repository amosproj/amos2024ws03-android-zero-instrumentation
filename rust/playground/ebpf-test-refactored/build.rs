// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{env, fs, path::PathBuf, process::Command};

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let ebpf_dir = "../ebpf-refactored";
    println!("cargo::rerun-if-changed={ebpf_dir}");

    let mut cmd = Command::new("cargo");
    cmd.env("CARGO_ENCODED_RUSTFLAGS", "-Cdebuginfo=2");
    cmd.current_dir(ebpf_dir);
    cmd.args([
        "build",
        "-Zbuild-std=core",
        "--bin=ebpf-refactored",
        "--target=bpfel-unknown-none",
        "--release",
        "--features=bounds-check",
    ]);

    let ebpf_target_dir = out_dir.join("backend/ebpf");
    cmd.arg("--target-dir").arg(&ebpf_target_dir);

    let output = cmd
        .output()
        .expect("Failed to run `cargo build` for ebpf-refactored");

    if !output.status.success() {
        panic!("{:?}", output);
    }

    fs::copy(
        ebpf_target_dir.join("bpfel-unknown-none/release/ebpf-refactored"),
        out_dir.join("ebpf.o"),
    )
    .expect("Failed to copy ebpf-refactored to ebpf.o");
}
