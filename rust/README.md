<!--
SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>

SPDX-License-Identifier: MIT
-->

# example

## Prerequisites

1. stable rust toolchains: `rustup toolchain install stable`
1. nightly rust toolchains: `rustup toolchain install nightly --component rust-src`
1. (if cross-compiling) rustup target: `rustup target add ${ARCH}-unknown-linux-musl`
1. (if cross-compiling) LLVM: (e.g.) `brew install llvm` (on macOS)
1. (if cross-compiling) C toolchain: (e.g.) [`brew install filosottile/musl-cross/musl-cross`](https://github.com/FiloSottile/homebrew-musl-cross) (on macOS)
1. bpf-linker: `cargo install bpf-linker` (`--no-default-features` on macOS)
1. android rust toolchain: `rustup target add x86_64-linux_android`
1. [Android NDK](https://developer.android.com/ndk)
1. cargo ndk: `cargo install cargo-ndk`

## Build & Run

Use `cargo build`, `cargo check`, etc. as normal.

- Run the server via `cargo xtask daemon`
- Run the client via `cargo xtask client`
- To run both on android, pass `--android`, e.g. `cargo xtask daemon --android`, this will push the executable via `adb push` to `/data/local/tmp` and execute it via `su`

## Cross-compiling on macOS

Cross compilation should work on both Intel and Apple Silicon Macs.

```shell
AYA_BUILD_EBPF=true CC=${ARCH}-linux-musl-gcc cargo build --package example --release \
  --target=${ARCH}-unknown-linux-musl \
  --config=target.${ARCH}-unknown-linux-musl.linker=\"${ARCH}-linux-musl-gcc\"
```
The cross-compiled program `target/${ARCH}-unknown-linux-musl/release/example` can be
copied to a Linux server or VM and run there.
