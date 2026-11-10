# The Windows 7 log-on screen as the display manager: greetd starts a
# Hyprland of its own with the screens given here, and that Hyprland runs
# shell/greeter.qml. Once Quickshell has started the chosen session, the
# Hyprland leaves and greetd runs the session.
#
# If the log-on screen leaves without starting a session (Quickshell
# crashed, or Ctrl+Alt+Delete left a log-on screen that drew nothing),
# greetd shows its text log-on (agreety) instead, which starts the default
# session: a broken log-on screen never keeps you out.
#
# The shell is served from /etc/shojiwm-win7/greeter, a path that does not
# change with the package: Quickshell names its state folder after the
# path, and keeps the last session chosen there.
{package}: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.shojiwm-win7-greeter;
  shellPath = "/etc/shojiwm-win7/greeter/greeter.qml";
  sessions = "${config.services.displayManager.sessionData.desktops}/share/wayland-sessions";

  # In the greeter's Hyprland: the log-on screen, then out. The mark tells
  # greetd's command below that a session was started.
  greeterSession = pkgs.writeShellScript "shojiwm-win7-greeter" ''
    export SHOJIWM_GREETER_USER=${lib.escapeShellArg cfg.user}
    export SHOJIWM_GREETER_SESSIONS=${sessions}
    export SHOJIWM_GREETER_DEFAULT_SESSION=${lib.escapeShellArg cfg.defaultSession}
    export SHOJIWM_WIN7_PRIMARY_SCREEN=${lib.escapeShellArg cfg.mainScreen}
    if ${lib.getExe pkgs.quickshell} -p ${shellPath}; then
      touch "$XDG_RUNTIME_DIR/shojiwm-win7-greeter-launched"
    fi
    # The configuration is Lua: dispatch takes a Lua call.
    ${pkgs.hyprland}/bin/hyprctl dispatch 'hl.dsp.exit()'
  '';

  hyprlandConfig = pkgs.writeText "shojiwm-win7-greeter.lua" ''
    ${lib.concatMapStrings (monitor: ''
        hl.monitor({
            output   = ${builtins.toJSON monitor.output},
            mode     = ${builtins.toJSON monitor.mode},
            position = ${builtins.toJSON monitor.position},
            scale    = ${toString monitor.scale},
        })
      '')
      cfg.monitors}
    hl.on("hyprland.start", function()
        hl.exec_cmd("${greeterSession}")
    end)

    -- A way out of a log-on screen that draws nothing: greetd then shows
    -- its text log-on.
    hl.bind("CTRL + ALT + Delete", hl.dsp.exit())

    hl.config({
        misc = {
            disable_hyprland_logo   = true,
            force_default_wallpaper = 0,
        },
        animations = {
            enabled = false,
        },
        ecosystem = {
            no_update_news = true,
        },
        input = {
            kb_layout  = ${builtins.toJSON cfg.keyboard.layout},
            kb_variant = ${builtins.toJSON cfg.keyboard.variant},
            kb_options = ${builtins.toJSON cfg.keyboard.options},
        },
    })
  '';

  # greetd's command: the log-on screen, else the text log-on into the
  # default session.
  greeter = pkgs.writeShellScript "shojiwm-win7-greeter-or-text" ''
    mark="$XDG_RUNTIME_DIR/shojiwm-win7-greeter-launched"
    rm -f "$mark"
    # Hyprland writes its banner and log to its output: to the journal
    # (journalctl -t shojiwm-win7-greeter), not over the log-on screen's
    # terminal before it draws.
    ${pkgs.systemd}/bin/systemd-cat -t shojiwm-win7-greeter -- \
      ${pkgs.hyprland}/bin/start-hyprland -- --config ${hyprlandConfig}
    if [ -e "$mark" ]; then
      exit 0
    fi
    session=$(sed -n 's/^Exec=//p' ${sessions}/${lib.escapeShellArg cfg.defaultSession}.desktop | head -n 1)
    exec ${pkgs.greetd}/bin/agreety --cmd "''${session:-$SHELL}"
  '';
in {
  options.services.shojiwm-win7-greeter = {
    enable = lib.mkEnableOption "the Windows 7 log-on screen of shojiwm-win7, through greetd";

    user = lib.mkOption {
      type = lib.types.str;
      description = "The account the log-on screen logs on.";
    };

    defaultSession = lib.mkOption {
      type = lib.types.str;
      default =
        if config.services.displayManager.defaultSession != null
        then config.services.displayManager.defaultSession
        else "";
      defaultText = lib.literalExpression "config.services.displayManager.defaultSession";
      example = "shojiwm-uwsm";
      description = ''
        The session chosen the first time, and the one the text log-on
        starts, by its .desktop file name without the extension.
      '';
    };

    mainScreen = lib.mkOption {
      type = lib.types.str;
      default = "";
      example = "HDMI-A-1";
      description = "Connector of the screen with the log-on box; the others stay black. Empty: the first.";
    };

    monitors = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule {
        options = {
          output = lib.mkOption {type = lib.types.str;};
          mode = lib.mkOption {
            type = lib.types.str;
            default = "preferred";
          };
          position = lib.mkOption {
            type = lib.types.str;
            default = "auto";
          };
          scale = lib.mkOption {
            type = lib.types.number;
            default = 1;
          };
        };
      });
      default = [];
      description = "The screens, as Hyprland's monitor rules. Empty: Hyprland's defaults.";
    };

    keyboard = {
      layout = lib.mkOption {
        type = lib.types.str;
        default = config.services.xserver.xkb.layout;
        defaultText = lib.literalExpression "config.services.xserver.xkb.layout";
      };
      variant = lib.mkOption {
        type = lib.types.str;
        default = config.services.xserver.xkb.variant;
        defaultText = lib.literalExpression "config.services.xserver.xkb.variant";
      };
      options = lib.mkOption {
        type = lib.types.str;
        default = config.services.xserver.xkb.options;
        defaultText = lib.literalExpression "config.services.xserver.xkb.options";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.etc."shojiwm-win7/greeter".source = "${package}/shell";

    services.greetd = {
      enable = true;
      settings.default_session.command = "${greeter}";
    };

    # The greeter keeps the last session chosen in its state folder
    # (~/.local/state), and Quickshell its cache.
    users.users.greeter = {
      home = "/var/lib/greeter";
      createHome = true;
    };
  };
}
