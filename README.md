<!--
SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>

SPDX-License-Identifier: MIT
-->

# Zero Instrumentation Observability for Android™[^1] (AMOS WS 2024)

[![REUSE status](https://api.reuse.software/badge/github.com/amosproj/amos2024ws03-android-zero-instrumentation)](https://api.reuse.software/info/github.com/amosproj/amos2024ws03-android-zero-instrumentation)

<img src="Deliverables/sprint-01/team-logo.svg" width="224">

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

[^1]: Android is a trademark of Google LLC. The Android robot is reproduced or modified from work created and shared by Google and used according to terms described in the Creative Commons 3.0 Attribution License.

## License

This project adheres to the [reuse](https://reuse.software/) Software recommendations. 

When modifying a file please add yourself to the list of Copyright Holders.
You can do that with `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" FILE`.

When adding a new file you have to add yourself to the list of Copyright holders and set the license.
You should prefer the MIT license if possible.
The easiest way to set the license and copyright is to execute `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" --license "MIT" FILE`.

To check whether you have done everything correctly, execute `reuse lint` in the project root directory.