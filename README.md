<!--
SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>

SPDX-License-Identifier: MIT
-->

# Zero Instrumentation Observability for Android™[^1] (AMOS WS 2024/25)

![Kotlin Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Famosproj%2Famos2024ws03-android-zero-instrumentation%2Fmaster%2Ffrontend%2Fgradle%2Flibs.versions.toml&query=%24.versions.kotlin&style=for-the-badge&label=Kotlin&color=pink)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Nix](https://img.shields.io/badge/NIX-5277C3.svg?style=for-the-badge&logo=NixOS&logoColor=white)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/amosproj/amos2024ws03-android-zero-instrumentation/docker-build.yml?style=for-the-badge)
[![REUSE status](https://api.reuse.software/badge/github.com/amosproj/amos2024ws03-android-zero-instrumentation)](https://api.reuse.software/info/github.com/amosproj/amos2024ws03-android-zero-instrumentation)
[![forthebadge](https://forthebadge.com/images/badges/works-on-my-machine.svg)](https://forthebadge.com)

<p align="center">
  <img src="Deliverables/sprint-01/team-logo.svg" width="224">
</p>

## Product Mission

**ZIOFA** (Zero Instrumentation Observability for Android) aims to implement observability use cases relevant to performance specified by our industry partner using eBPF. Examples include tracing long-running blocking calls, leaking JNI indirect references or signals like SIGKILL sent to processes, all without instrumenting the observed application itself.  
The eBPF programs are loaded and unloaded using a **backend daemon** running as root that will collect metrics and send them to a client.  For displaying these metrics to the user, we are implementing an **on-device UI** that can display visualizations for these use cases and allow for configuration of the enabled use cases, but **using a decoupled Client SDK** so that future work may easily make the data accessible the external processing.

## Prerequisites

The easiest way to get all dependencies is to use the provided development shell in `flake.nix`. For that you will need [Nix](https://nixos.org/download/).
Additionally you can use a tool like [`direnv`](https://github.com/direnv/direnv) to automatically load the environment for this repository.

The shell will setup:

- The rust nightly toolchain (nightly is required currently for ebpf because of the unstable [`build-std`](https://doc.rust-lang.org/cargo/reference/unstable.html#build-std) feature)
- The [`bpf-linker`](https://github.com/aya-rs/bpf-linker/)
- The Android SDK and NDK
- The [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk) package for compiling for rust for android
- The [`protobuf`](https://protobuf.dev/) programs for generating grpc server and client code

## Build & Deploy
### Emulator Setup

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

### Frontend
The simplest way to build and test everything is the following command inside of `frontend/` (this will take a while):
```
./gradlew build
```

The apks for each build configuration are located in `frontend/app/build/outputs/apk/`.

To install the application using the real backend on your device or emulator, run either `./gradlew installRealDebug` or `adb install path/to/real/app.apk`

There are other flavors available, for example `installMockDebug`, for a frontend using fake data instead of the real backend.
This is mainly interesting for development purposes.

In order to view the full list of tasks configured in gradle, run
```
./gradlew tasks
```

### Backend
#### All in one deploy
If you'd like to build and run the daemon all in one command, you can use
```
cargo xtask daemon
```

This will ask you for root privileges to run the built executable.

By running
```
cargo xtask daemon --android
```
the executable won't start on your device but instead on an adb reachable android device or emulator by pushing it to `/data/local/tmp/backend-daemon` on the device and running it with root there.

#### Do it youself deploy
To just build the daemon, run the following command inside of `rust/` for your desired architecture:
```
AYA_BUILD_EBPF=true cargo ndk -t x86_64 build --package backend-daemon
AYA_BUILD_EBPF=true cargo ndk -t arm64-v8a build --package backend-daemon
```
You can then proceed to copy the executable (`rust/target/debug/backend-daemon`) to wherever you like and run it. You need root privileges in order to run it.

## Usage
The app can be used like any other android app. Just open it from the device launcher.

### Demo video
[20250128-2116-demo-video.mp4](https://github.com/user-attachments/assets/61f5fd46-1878-4e9c-9112-542d884e0976)

### Visualize

This screen offers visualizations for different kinds of events. 
You can select the package you want to inspect, the kind of metric that interests you and time intervals.

![Visualization](https://github.com/user-attachments/assets/f81a3513-cf5b-483a-9c68-0af355159144)


### Configuration

The configuration screen allows you to select options per process.

![image](https://github.com/user-attachments/assets/75909616-3819-447f-bb50-52f25de9b99c)

### Reset

This empties the configuration and allows for a clean restart.

## Technical Design
To view more information on our technical design, please check our [Wiki](https://github.com/amosproj/amos2024ws03-android-zero-instrumentation/wiki/Technical-Design)

## Documentation

To generate the html documentation run the following:

```
asciidoctor -r asciidoctor-diagram Documentation/asciidoc/main.adoc -o Documentation/build/doc/index.html
```

## License

This project adheres to the [reuse](https://reuse.software/) Software recommendations. 

When modifying a file please add yourself to the list of Copyright Holders.
You can do that with `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" FILE`.

When adding a new file you have to add yourself to the list of Copyright holders and set the license.
You should prefer the MIT license if possible.
The easiest way to set the license and copyright is to execute `reuse annotate --copyright="YOUR NAME <YOUR EMAIL>" --license "MIT" FILE`.

To check whether you have done everything correctly, execute `reuse lint` in the project root directory.

[^1]: Android is a trademark of Google LLC. The Android robot is reproduced or modified from work created and shared by Google and used according to terms described in the Creative Commons 3.0 Attribution License.
