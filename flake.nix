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
  };

  outputs = inputs@{ self, nixpkgs, fenix, android-nixpkgs, nix2container, flake-parts, ... }:
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
              system-images-android-35-default-x86-64
            ];

            rustPkgs = with pkgs.fenix; [
              (combine [ latest.toolchain targets.x86_64-linux-android.latest.rust-std ])
            ];

            miscPkgs = with pkgs; [
              cargo-ndk
              protobuf
              bpf-linker
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

        in
        {
          devShells = {
            default = pkgs.mkShell { packages = packageGroups.combined; };
          };
          packages = {
            dockerBuilderBase = builderBase;
            dockerBuilder = builder;
          };
        };
    };
}
