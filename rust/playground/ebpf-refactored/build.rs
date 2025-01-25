use std::env;

pub fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")
        .expect("`CARGO_CFG_TARGET_ARCH` should be set in a buildscript");

    if target_arch == "bpf" {
        let bitcode_path = env::var("DEP_RELOCATION_HELPERS_BITCODE_PATH")
            .expect("`DEP_RELOCATION_HELPERS_BITCODE_PATH` should be set when importing `relocation-helpers`");

        println!("cargo::rustc-link-arg={bitcode_path}");
        println!("cargo::rustc-link-arg=--btf");
    }
}
