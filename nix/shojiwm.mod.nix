# The compositor, from the fork's own package expression, built with this
# flake's nixpkgs: the one the shell and the portal are built with. The fork's
# NixOS module does the same with the system's nixpkgs, so that the compositor
# loads the system's graphics driver.
{inputs, ...}: {
  perSystem = {pkgs, ...}: {
    packages.shojiwm = pkgs.callPackage "${inputs.shojiwm}/nix/package.nix" {
      libgbm = pkgs.libgbm or pkgs.mesa;
      xwayland = pkgs.xwayland or (pkgs.xorg.xwayland or null);
    };
  };
}
