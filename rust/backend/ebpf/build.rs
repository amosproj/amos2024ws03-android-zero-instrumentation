// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

use std::env;

use which::which;
use xtask::AYA_BUILD_EBPF;

/// Building this crate has an undeclared dependency on the `bpf-linker` binary. This would be
/// better expressed by [artifact-dependencies][bindeps] but issues such as
/// https://github.com/rust-lang/cargo/issues/12385 make their use impractical for the time being.
///
/// This file implements an imperfect solution: it causes cargo to rebuild the crate whenever the
/// mtime of `which bpf-linker` changes. Note that possibility that a new bpf-linker is added to
/// $PATH ahead of the one used as the cache key still exists. Solving this in the general case
/// would require rebuild-if-changed-env=PATH *and* rebuild-if-changed={every-directory-in-PATH}
/// which would likely mean far too much cache invalidation.
///
/// [bindeps]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html?highlight=feature#artifact-dependencies
fn main() {
    println!("cargo:rerun-if-env-changed={}", AYA_BUILD_EBPF);

    let build_ebpf = env::var(AYA_BUILD_EBPF)
        .as_deref()
        .map(str::parse)
        .map(Result::unwrap)
        .unwrap_or_default();

    if build_ebpf {
        let bpf_linker = which("bpf-linker").unwrap();
        println!("cargo:rerun-if-changed={}", bpf_linker.to_str().unwrap());
    }
}
