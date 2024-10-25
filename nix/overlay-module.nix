# This module takes the fenix and android-nixpkgs overlays and applies them to
# the nixpkgs provided by the flake.
({ self, inputs, ... }: {
  perSystem = { system, ... }: {
    _module.args.pkgs = import self.inputs.nixpkgs {
      inherit system;
      overlays = [
        self.inputs.fenix.overlays.default
        self.inputs.android-nixpkgs.overlays.default
        (prev: super: { n2c = inputs.nix2container.packages.${system}.nix2container; })
        (prev: super: { bashConfigs = import ./bash-configs.nix { pkgs = prev; }; })
        (prev: super: { buildBom = inputs.bombon.lib.${system}.buildBom; })
      ];
    };
  };
})
