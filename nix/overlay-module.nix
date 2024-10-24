# This module takes the fenix and android-nixpkgs overlays and applies them to
# the nixpkgs provided by the flake.
({ self, inputs, ... }: {
  perSystem = { system, ... }: {
    _module.args.pkgs = import self.inputs.nixpkgs {
      inherit system;
      overlays = [
        self.inputs.fenix.overlays.default
        self.inputs.android-nixpkgs.overlays.default
      ];
    };
  };
})
