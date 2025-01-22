use std::{env, path::PathBuf};

pub fn main() {
    println!("cargo::rerun-if-changed=src/c/relocation_helper.c");
    println!("cargo::rerun-if-changed=src/c/relocation_helper.h");
    
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")
        .expect("`CARGO_CFG_TARGET_ARCH` should be set in a buildscript");
    
    if target_arch == "bpf" {
        build_helpers_bpf();
    } else {
        build_helpers_not_bpf();
    }
    
    generate_bindings();
}

fn build_helpers_bpf() {
    let bitcode_file = cc::Build::new()
        .compiler("clang")
        .no_default_flags(true)
        .file("src/c/relocation_helpers.c")
        .flag("-g")
        .flag("-emit-llvm")
        .compile_intermediates()
        .into_iter()
        .next()
        .expect("bitcode file should be compiled");

    println!("cargo::rustc-link-arg={}", bitcode_file.display());
    println!("cargo::rustc-link-arg=--btf");
}

fn build_helpers_not_bpf() {
    cc::Build::default()
        .compiler("clang")
        .no_default_flags(true)
        .file("src/c/relocation_helpers.c")
        .flag("-flto=thin")
        .compile("relocation_helpers");

    println!("cargo::rustc-link-arg=-lc");
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        .use_core()
        .header("src/c/relocation_helpers.h")
        .generate()
        .expect("generating bindings should not fail");

    let out_dir = env::var("OUT_DIR").expect("`OUT_DIR` should be set in a buildscript");

    let out_file_path = PathBuf::from(out_dir).join("relocation_helpers.rs");

    bindings
        .write_to_file(out_file_path)
        .expect("writing bindings should not fail");
}
