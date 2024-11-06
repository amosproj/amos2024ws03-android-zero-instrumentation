# SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
# SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
#
# SPDX-License-Identifier: MIT

{
  description = "A very basic flake";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
    };
    nix2container = {
      url = "github:nlewo/nix2container";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    bombon = {
      url = "github:nikstur/bombon";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, fenix, android-nixpkgs, nix2container, bombon, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./nix/overlay-module.nix
      ];
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = { config, pkgs, system, ... }:
        let
          packageGroups = {

            base = with pkgs; [
              git
              python3
              gcc
              gnugrep
              gnused
              gawk
              findutils
              iproute2
              cacert
              gcc.cc.lib
              glibc.out
              coreutils
              which
              bash
            ];

            sdkBase = with pkgs; [
              ncurses5
              openjdk
            ];

            sdkPkgs = with pkgs.androidSdkPackages; [
              cmdline-tools-latest
              ndk-28-0-12433566
              build-tools-35-0-0
              platform-tools
              platforms-android-35
              emulator
              system-images-android-33-default-x86-64
            ];

            rustPkgs = with pkgs.fenix; [
              (combine [
                latest.toolchain
                targets.x86_64-linux-android.latest.rust-std
                targets.aarch64-linux-android.latest.rust-std
              ])
            ];

            miscPkgs = with pkgs; [
              cargo-ndk
              protobuf
              bpf-linker
              cyclonedx-cli
              cargo-cyclonedx
              reuse
            ];

            combined =
              packageGroups.base ++
              packageGroups.sdkBase ++
              [ (pkgs.androidSdk (_: packageGroups.sdkPkgs)) ] ++
              packageGroups.rustPkgs ++
              packageGroups.miscPkgs;
          };

          builderBase = pkgs.n2c.buildImage (with pkgs; {
            name = "ghcr.io/fhilgers/ziofa-builder-base";
            tag = "latest";
            copyToRoot = [
              (buildEnv {
                name = "root";
                pathsToLink = [ "/bin" "/lib64" "/lib" "/share" "/etc" ];
                paths = packageGroups.combined;
              })
            ];
            layers =
              let
                mL = deps: layers: n2c.buildLayer { deps = deps; layers = layers; };
                baseLayer = mL packageGroups.base [ ];
                miscLayer = mL packageGroups.miscPkgs [ baseLayer ];
                rustLayer = mL packageGroups.rustPkgs [ baseLayer ];
                sdkBaseLayer = mL packageGroups.sdkBase [ baseLayer ];
                sdkLayers = map (sdkPkg: mL [ sdkPkg ] [ sdkBaseLayer baseLayer ]) packageGroups.sdkPkgs;
              in
              [
                baseLayer
                miscLayer
                rustLayer
                sdkBaseLayer
              ] ++ sdkLayers;
          });

          dockerHelpers = import ./nix/docker-helpers.nix { pkgs = pkgs; };
          inherit (dockerHelpers) mkWorker mkDoas mkDoasPerms mkWorkerPerms mkTmp mkHome mkEnv;

          builder = pkgs.n2c.buildImage {
            name = "ghcr.io/fhilgers/ziofa-builder";
            tag = "latest";
            fromImage = builderBase;
            copyToRoot = [
              mkTmp
              mkEnv
              mkHome
              mkWorker
              mkDoas

              # github actions
              # TODO: move them to another layer
              pkgs.nodejs
              pkgs.gnutar
              pkgs.zstd
              pkgs.gzip

              pkgs.bashInteractive
              pkgs.bashConfigs
            ];
            perms = [
              (mkWorkerPerms mkTmp "/tmp")
              (mkWorkerPerms mkHome "/home/worker")
              (mkDoasPerms mkDoas "/sbin/doas")
            ];
            config = {
              entrypoint = [ "/bin/bash" ];
              User = "worker";
              WorkingDir = "/home/worker";
              Env = [
                "PATH=/bin:/sbin:/usr/bin:/usr/sbin"
                "HOME=/home/worker"
                "LD_LIBRARY_PATH=/lib:/lib64"
                "ANDROID_HOME=/share/android-sdk" # deprecated but android-studio needs it
                "ANDROID_SDK_ROOT=/share/android-sdk"
                "ANDROID_NDK_HOME=/share/android-sdk/ndk"
              ];
            };
          };

          toolsDevShell = pkgs.mkShell {
            packages = packageGroups.combined;
          };

          generateSbom =
            let
              PATH = pkgs.lib.makeBinPath (with packageGroups; base ++ sdkBase ++ rustPkgs ++ miscPkgs);
            in
            pkgs.writeShellScriptBin "generate-sbom.sh" ''
              set -Euo pipefail
              ROOT="$(${pkgs.git}/bin/git rev-parse --show-toplevel)"
              export PATH="${PATH}:$PATH"
              (cd "$ROOT" && python utils/generate_sbom.py)
            '';

          rustCiPreamble = ''
            export PATH=${pkgs.lib.makeBinPath (with pkgs; [ protobuf clang cargo-ndk bpf-linker ] ++ packageGroups.rustPkgs)}:$PATH
            set -euo pipefail
          '';
          frontendCiPreamble =
            let
              minimalSdkPkgs = with pkgs.androidSdkPackages; [ cmdline-tools-latest build-tools-35-0-0 platforms-android-35 platform-tools ndk-28-0-12433566 ];
              minimalSdk = pkgs.androidSdk (_: minimalSdkPkgs);
            in
            ''
              ${rustCiPreamble}
              export PATH=${pkgs.lib.makeBinPath (with pkgs; [ openjdk21 minimalSdk ])}:$PATH;
              export ANDROID_SDK_ROOT=${minimalSdk}/share/android-sdk
              export ANDROID_HOME=$ANDROID_SDK_ROOT
              set -euo pipefail
            '';

          rustLint = pkgs.writeShellScriptBin "rust-lint" ''
            ${rustCiPreamble}
            (cd rust && cargo clippy)
          '';

          rustTest = pkgs.writeShellScriptBin "rust-test" ''
            ${rustCiPreamble}
            (cd rust && cargo test)
          '';
          
          rustBuildRelease = pkgs.writeShellScriptBin "rust-build-release" ''
            ${frontendCiPreamble}
            export AYA_BUILD_EBPF=true
            (
              cd rust
              cargo ndk --target x86_64 build --package example --release
              cargo ndk --target arm64-v8a build --package example --release
              cargo ndk --target x86_64 build --package client --features uniffi --release
              cargo ndk --target arm64-v8a build --package client --features uniffi --release
              mkdir -p /tmp/outputs/
              cp target/x86_64-linux-android/release/example /tmp/outputs/daemon-x86_64-linux-android
              cp target/x86_64-linux-android/release/libclient.so /tmp/outputs/libclient-x86_64-linux-android.so
              cp target/aarch64-linux-android/release/example /tmp/outputs/daemon-aarch64-linux-android
              cp target/aarch64-linux-android/release/libclient.so /tmp/outputs/libclient-aarch64-linux-android.so
            )
          '';

          reuseLint = pkgs.writeShellScriptBin "reuse-lint" ''
            ${pkgs.reuse}/bin/reuse lint
          '';

          gradleLint = pkgs.writeShellScriptBin "gradle-lint" ''
            ${frontendCiPreamble}
            (
              cd frontend
              ./gradlew ktfmtCheck
              ./gradlew lint
            )
          '';

          gradleTest = pkgs.writeShellScriptBin "gradle-build-test" ''
            ${frontendCiPreamble}
            (
              cd frontend
              ./gradlew check
            )
          '';

          gradleBuildRelease = pkgs.writeShellScriptBin "gradle-build-release" ''
            ${frontendCiPreamble}
            (
              cd frontend
              ./gradlew app:assembleRelease
              mkdir -p /tmp/outputs
              cp app/build/outputs/apk/release/app-*-release-unsigned.apk /tmp/outputs
            )
          '';


        in
        {
          devShells = {
            default = toolsDevShell;
          };
          packages = {
            dockerBuilderBase = builderBase;
            dockerBuilder = builder;
            toolsSbom = pkgs.buildBom toolsDevShell { };
            generateSbom = generateSbom;

            inherit rustLint rustTest reuseLint gradleLint gradleTest rustBuildRelease gradleBuildRelease;
          };
        };
    };
}
