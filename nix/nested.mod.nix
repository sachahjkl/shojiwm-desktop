# `nix run github:sachahjkl/shojiwm-desktop`: the whole desktop in a window
# of the current Wayland or X11 session, with the packaged configuration.
#
# ShojiWM opens its nested (winit) backend when it is not given --tty. The
# session gets folders of its own for its configuration and state, and a
# session bus of its own, so it changes nothing in ~/.config, ~/.local/state
# or the running desktop: dconf, the notification area and the shell's
# settings all stay inside it.
{
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    inherit (config.packages) icon-theme portal shell shojiwm;
    desktopConfig = config.packages.config;
  in {
    packages.nested = pkgs.writeShellApplication {
      name = "shojiwm-desktop-nested";
      runtimeInputs = [pkgs.coreutils pkgs.dbus shojiwm shell];
      text = ''
        if [ -z "''${WAYLAND_DISPLAY:-}" ] && [ -z "''${DISPLAY:-}" ]; then
          echo "no WAYLAND_DISPLAY or DISPLAY: start this from a graphical session" >&2
          exit 1
        fi

        work="$(mktemp -d "''${TMPDIR:-/tmp}/shojiwm-desktop.XXXXXX")"
        trap 'rm -rf "$work"' EXIT
        export XDG_CONFIG_HOME="$work/config"
        export XDG_STATE_HOME="$work/state"

        # The configuration as `install` writes it. ShojiWM's init-config
        # then adds node_modules/shoji_wm; it keeps src, which has its
        # index.tsx.
        config_dir="$XDG_CONFIG_HOME/shojiwm"
        mkdir -p "$config_dir" "$XDG_STATE_HOME"
        cp -R ${desktopConfig}/src ${desktopConfig}/assets ${desktopConfig}/shell \
          ${desktopConfig}/package.json ${desktopConfig}/tsconfig.json "$config_dir/"
        chmod -R u+w "$config_dir"
        shojiwm-init-config >/dev/null

        # What a running ShojiWM desktop puts in its session names that
        # desktop's sockets, screens and keys, not this one's.
        unset SHOJIWM_SOCKET SHOJIWM_DESKTOP_SOCKET SHOJIWM_DESKTOP_KEYS \
          SHOJIWM_DESKTOP_OUTPUTS SHOJIWM_DESKTOP_PRIMARY_SCREEN SHOJIWM_SHELL_DIR \
          SHOJI_CONFIG SHOJI_RUNTIME_DIR SHOJI_DECORATION_RUNTIME
        # The variables of the session around it stay that session's.
        export SHOJI_PUBLISH_ACTIVATION_ENV=off

        # The Aero icon theme with the Oxygen icons it inherits, and the
        # GlobalShortcuts portal's D-Bus service for the bus below.
        export XDG_DATA_DIRS="${icon-theme}/share:${pkgs.kdePackages.oxygen-icons}/share:${portal}/share:''${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

        # The bus's own configuration, for a system with no /etc/dbus-1.
        bus_options=()
        if [ ! -e /etc/dbus-1/session.conf ]; then
          bus_options=(--config-file=${pkgs.dbus}/share/dbus-1/session.conf)
        fi
        unset DBUS_SESSION_BUS_ADDRESS
        # Under X11, winit opens Xlib and its extensions at run time (dlopen): they are not
        # linked, so they must be on the library path.
        export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [libX11 libxcb libXcursor libXrandr libXi libxkbcommon])}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        dbus-run-session "''${bus_options[@]}" -- shoji_wm "$@"
      '';
      meta = {
        description = "Run the shojiwm-desktop session in a window of the current session";
        mainProgram = "shojiwm-desktop-nested";
      };
    };

    apps.default = {
      type = "app";
      program = "${config.packages.nested}/bin/shojiwm-desktop-nested";
    };
  };
}
