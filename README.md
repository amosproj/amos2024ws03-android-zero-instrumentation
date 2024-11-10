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

## Emulator Setup

As we need a modified version of Android, we cannot use the standard system images that come with the default Android SDK.
To make development easier, a custom Android SDK is loaded into your environment using the nix development shell.

> [!NOTE]
> The Android SDK shipped with the shell is built of the standard parts with everything necessary for building Android Apps.
> The only "custom' part is the system image, which was build externally and is currently hosted on S3.
> The image can be also downloaded manually from [sdk-repo-linux-system-images.zip](https://ftrace-emu.nbg1.your-objectstorage.com/emulator_car64_x86_64/sdk-repo-linux-system-images.zip) with its manifest stored in [package.xml](https://ftrace-emu.nbg1.your-objectstorage.com/emulator_car64_x86_64/package.xml).
> You can also download and unzip the system image to `$ANDROID_SDK_ROOT/system-images/android-VanillaIceCream/android-automotive/x86_64` manually and copy the `package.xml` to that directory as well.

This sdk includes the automotive system image, which is built with a custom kernel having `CONFIG_FTRACE_SYSCALLS=y` set.
To create an emulator using that image, you can use the `avdmanager` tool also provided with the SDK:

```
avdmanager create avd -n YOUR_AVD_NAME  -k 'system-images;android-VanillaIceCream;android-automotive;x86_64' --device automotive_1080p_landscape
```

This can be started with the `emulator` tool like this:


```
emulator @YOUR_AVD_NAME
```

## License

This project adheres to the [reuse](https://reuse.software/) Software recommendations. 

When modifying a file please add yourself to the list of Copyright Holders.
You can do that with `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" FILE`.

When adding a new file you have to add yourself to the list of Copyright holders and set the license.
You should prefer the MIT license if possible.
The easiest way to set the license and copyright is to execute `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" --license "MIT" FILE`.

To check whether you have done everything correctly, execute `reuse lint` in the project root directory.
