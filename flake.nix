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
  };

  outputs = { self, nixpkgs, fenix, android-nixpkgs }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };
      fenixPkgs = fenix.packages.${system};
      mkAndroidSdk = android-nixpkgs.sdk.${system};

      androidSdk = {
        sdk = mkAndroidSdk (sdkPkgs: with sdkPkgs; [
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
        nightlyToolchain = fenixPkgs.combine [
          fenixPkgs.latest.toolchain
          fenixPkgs.targets.x86_64-linux-android.latest.rust-std
        ];
      };

    in
    {
      devShells.${system}.default = pkgs.mkShell {
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
}
