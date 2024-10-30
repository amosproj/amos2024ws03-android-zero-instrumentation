<!--
SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>

SPDX-License-Identifier: MIT
-->

# Zero Instrumentation Observability for Android (AMOS WS 2024)

[![REUSE status](https://api.reuse.software/badge/github.com/amosproj/amos2024ws03-android-zero-instrumentation)](https://api.reuse.software/info/github.com/amosproj/amos2024ws03-android-zero-instrumentation)


## Building

### [Nix](https://nixos.org/download/)

The easiest way to get all dependencies is to use the provided development shell in `flake.nix`.
You can use a tool like [`direnv`](https://github.com/direnv/direnv) to automatically load the environment for this repository.

The shell will setup:

- The rust nightly toolchain (nightly is required currently for ebpf because of the unstable [`build-std`](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std) feature)
- The [`bpf-linker`](https://github.com/aya-rs/bpf-linker/)
- The Android SDK and NDK
- The [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk) package for compiling for rust for android
- The [`protobuf`](https://protobuf.dev/) programs for generating grpc server and client code

### [OCI Container](https://opencontainers.org/)

We use the same input that go into the development shell for building a layered docker image.
This image is used for the ci, so it supports building the project as well, however you graphical applications like the android emulator require custom setups depending on your OS.

The container image is built via nix and can be automatically uploaded to the registry via `nix run .#dockerBuilder.copyToRegistry`.
It is currently hosted in the github registry under `ghcr.io/fhilgers/ziofa-builder`.

For a working emulator inside the docker image, you will need some form of X11 forwarding.
Installing nix and using the development shell is the recommended approach for development.
