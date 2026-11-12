# The Windows 7 log-on screen as the display manager (greetd): see
# nix/greeter.nix and services.shojiwm-desktop-greeter.
{
  self,
  withSystem,
  ...
}: {
  flake.nixosModules.greeter = {
    # The key lets nixosModules.default import this module while a system
    # imports it too.
    key = "shojiwm-desktop#nixosModules.greeter";
    imports = [
      ({
        config,
        lib,
        pkgs,
        ...
      }: let
        packages = withSystem pkgs.stdenv.hostPlatform.system (perSystem: perSystem.config.packages);
      in
        import ./greeter.nix {
          package = packages.default;
          inherit (packages) shell;
        } {inherit config lib pkgs;})
    ];
  };

  perSystem = {pkgs, ...}: {
    # The log-on screen in a VM: greetd, Hyprland, shojiwm-shell greeter, a password,
    # and the session it starts.
    checks.greeter = import ../tests/greeter.nix {
      inherit pkgs;
      module = self.nixosModules.greeter;
    };
  };
}
