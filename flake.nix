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
  };

  outputs = inputs@{ self, nixpkgs, fenix, android-nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./nix/overlay-module.nix
      ];
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = { config, pkgs, system, ... }:
        let
          androidSdk = {
            sdk = pkgs.androidSdk (sdkPkgs: with sdkPkgs; [
              cmdline-tools-latest
              ndk-28-0-12433566
              build-tools-35-0-0
              platform-tools
              platforms-android-35
              emulator
              system-images-android-35-default-x86-64
            ]);
          };
          rust = {
            # https://rust-lang.github.io/rustup-components-history/
            # We need nightly for aya to build ebpf programs
            nightlyToolchain = pkgs.fenix.combine (with pkgs.fenix; [
              latest.toolchain
              targets.x86_64-linux-android.latest.rust-std
            ]);
          };
        in
        {
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rust.nightlyToolchain
              protobuf
              bpf-linker
              androidSdk.sdk
              cargo-ndk
              python3
            ];
          };
        };
    };
}
