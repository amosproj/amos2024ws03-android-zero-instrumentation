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
  };

  outputs = { self, nixpkgs, fenix }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };
      fenixPkgs = fenix.packages.${system};

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
        ];
      };
    };
}
