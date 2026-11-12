{
  description = "A complete Wayland desktop environment, built on ShojiWM";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2605";

    flake-parts = {
      url = "https://flakehub.com/f/hercules-ci/flake-parts/0.1";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    git-hooks = {
      url = "https://flakehub.com/f/cachix/git-hooks.nix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The compositor: the author's fork, whose integration branch carries what
    # this config uses before upstream has it (labels' textShadow; see its
    # FORK.md). Its source is also the TypeScript package the config is
    # checked against, and its NixOS module is the base of ours.
    shojiwm = {
      url = "github:sachahjkl/ShojiWM/integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The toolkit whose changelog package tools/desktoptool uses (-collection:gt=<it>/src). The
    # development shell gives its path in $GUI_TOOLKIT_SRC.
    gui-toolkit = {
      url = "github:sachahjkl/gui_toolkit/v0.191.1";
      flake = false;
    };
  };

  # Each part of the flake is a flake-parts module in nix/*.mod.nix.
  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} ({lib, ...}: let
      # The release version: tools/desktoptool reads and writes the first
      # `version = "x";` of this file, beside Cargo.toml's and Cargo.lock's.
      version = "0.2.0";
    in {
      systems = ["x86_64-linux" "aarch64-linux"];

      _module.args.version = version;

      imports = lib.filter (path: lib.hasSuffix ".mod.nix" (toString path)) (lib.filesystem.listFilesRecursive ./nix);
    });
}
