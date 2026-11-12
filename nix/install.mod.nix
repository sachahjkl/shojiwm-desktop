# `nix run .#install`: the configuration into ~/.config/shojiwm.
{
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    desktopConfig = config.packages.config;
  in {
    packages.install = pkgs.writeShellApplication {
      name = "shojiwm-desktop-install";
      runtimeInputs = [pkgs.coreutils];
      text = ''
        config_dir="''${XDG_CONFIG_HOME:-$HOME/.config}/shojiwm"
        mkdir -p "$config_dir"

        # node_modules/shoji_wm is created by ShojiWM itself; leave it.
        rm -rf "$config_dir/src" "$config_dir/assets" "$config_dir/shell"
        cp -R ${desktopConfig}/src ${desktopConfig}/assets ${desktopConfig}/shell "$config_dir/"
        cp ${desktopConfig}/package.json ${desktopConfig}/tsconfig.json "$config_dir/"
        chmod -R u+w "$config_dir/src" "$config_dir/assets" "$config_dir/shell"
        chmod u+w "$config_dir/package.json" "$config_dir/tsconfig.json"

        echo "installed the Windows 7 desktop in $config_dir"
        echo "log out and start the ShojiWM session, or press Super+Shift+R"
      '';
    };

    apps.install = {
      type = "app";
      program = "${config.packages.install}/bin/shojiwm-desktop-install";
    };
  };
}
