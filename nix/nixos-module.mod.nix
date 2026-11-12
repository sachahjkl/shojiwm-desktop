# nixosModules.default: the whole desktop session on NixOS. ShojiWM (its own
# module, programs.shojiwm), the configuration in each user's
# ~/.config/shojiwm, the shell, the GlobalShortcuts portal, the Aero icon
# theme, and a "ShojiWM (UWSM)" session. The log-on screen is
# services.shojiwm-desktop-greeter (nix/greeter.nix), imported here.
{
  inputs,
  self,
  withSystem,
  ...
}: {
  flake.nixosModules.default = {
    key = "shojiwm-desktop#nixosModules.default";
    imports = [
      inputs.shojiwm.nixosModules.default
      self.nixosModules.greeter
      ({
        config,
        lib,
        pkgs,
        ...
      }: let
        cfg = config.services.shojiwm-desktop;
        packages = withSystem pkgs.stdenv.hostPlatform.system (perSystem: perSystem.config.packages);

        # A checkout is linked as is: its files change without a rebuild.
        configSource =
          if cfg.checkout == null
          then packages.config
          else cfg.checkout;

        # nm-applet's autostart entry is only replaced where nm-applet is
        # installed: elsewhere the copy would start a program that is not there.
        # By name: comparing packages would evaluate every store path.
        hasNetworkManagerApplet =
          config.programs.nm-applet.enable
          || lib.any (package: lib.getName package == lib.getName pkgs.networkmanagerapplet) config.environment.systemPackages;
      in {
        options.services.shojiwm-desktop = {
          enable = lib.mkEnableOption "the shojiwm-desktop session: ShojiWM with the desktop's configuration, shell and portal";

          users = lib.mkOption {
            type = lib.types.listOf lib.types.str;
            default = [];
            example = ["alice"];
            description = ''
              The accounts whose ~/.config/shojiwm holds the desktop's
              configuration. Their src, assets and shell link to it; ShojiWM's
              init-config writes the rest at each activation.
            '';
          };

          # A string, not a path: a path would be copied to the store, and the
          # checkout would then change only with a rebuild.
          checkout = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            example = "/home/alice/Projects/shojiwm-desktop";
            description = ''
              Path of a shojiwm-desktop checkout. If set, ~/.config/shojiwm
              links to this checkout instead of the packaged configuration.
            '';
          };

          # The programs the desktop opens for its roles: by default the author's
          # (inspecteur, classeur, papier, cliche). Each package is installed;
          # null installs none and keeps only the command.
          programs = let
            role = description: package: command:
              lib.mkOption {
                inherit description;
                default = {};
                type = lib.types.submodule {
                  options = {
                    package = lib.mkOption {
                      type = lib.types.nullOr lib.types.package;
                      default = package;
                      defaultText = lib.literalMD "the author's program, where it is built (x86_64-linux)";
                      description = "The program's package, installed with the session; null for none.";
                    };
                    command = lib.mkOption {
                      type = lib.types.str;
                      default = command;
                      description = "The command the desktop runs for this role.";
                    };
                  };
                };
              };
            ours = name: inputs.${name}.packages.${pkgs.stdenv.hostPlatform.system}.default or null;
          in {
            taskManager = role "The task manager: Ctrl+Shift+Escape, the taskbar's and the clock's menus." (ours "inspecteur") "uwsm app -- inspecteur";
            systemMonitor = role "What the CPU Meter gadget's button opens." null "uwsm app -- inspecteur --section performance";
            fileManager = role "The file manager: the desktop's folders, Computer, the keys that open a folder." (ours "classeur") "uwsm app -- classeur";
            clipboard = role "The clipboard history: Super+V." (ours "papier") "papier toggle";
            screenCapture = role ''
              The screen capture. An empty command: the program binds Print
              Screen itself (through the GlobalShortcuts portal), as cliche
              does, and the compositor leaves the key to it.
            '' (ours "cliche") "";
          };

          fileManagerEntry = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = "classeur.desktop";
            description = "The desktop entry that opens folders (inode/directory) in this session only; null leaves the system's choice.";
          };

          environment = lib.mkOption {
            type = lib.types.attrsOf lib.types.str;
            default = {};
            example = {
              SHOJIWM_DESKTOP_TERMINAL = "uwsm app -- kitty";
              SHOJIWM_DESKTOP_OUTPUTS = "DP-2,2560x1440@180,0x0,1.25";
            };
            description = ''
              The session's environment, written to uwsm's env file for
              ShojiWM: SHOJIWM_DESKTOP_* (see "Changing it" in the README),
              XKB_DEFAULT_*, and any other variable. An empty value is left
              out. The XKB variables default to services.xserver.xkb, and
              DCONF_PROFILE to the session's profile.
            '';
          };
        };

        config = lib.mkIf cfg.enable {
          programs = {
            shojiwm = {
              enable = true;
              # Creates package.json and the shoji_wm link in ~/.config/shojiwm.
              initConfig = {
                enable = true;
                inherit (cfg) users;
              };
            };

            # The "ShojiWM (UWSM)" session below.
            uwsm.enable = true;

            # The session's dconf profile: the same user database as the other
            # sessions', so the settings stay shared, and before the system
            # ones a database that chooses the Aero icon theme, the desktop's
            # icons, for Thunar and the other GTK programs. A value written to
            # the user database would still come first, in every session.
            dconf = {
              enable = true;
              profiles.shojiwm = {
                enableUserDb = true;
                databases =
                  [{settings."org/gnome/desktop/interface".icon-theme = "Aero";}]
                  ++ config.programs.dconf.profiles.user.databases or [];
              };
            };
          };

          services = {
            shojiwm-desktop.environment = lib.mapAttrs (_: lib.mkDefault) {
              # ShojiWM sets no keymap; libxkbcommon then reads these.
              XKB_DEFAULT_LAYOUT = config.services.xserver.xkb.layout;
              XKB_DEFAULT_VARIANT = config.services.xserver.xkb.variant;
              XKB_DEFAULT_OPTIONS = config.services.xserver.xkb.options;
              # The dconf profile below: the Aero icon theme in this session.
              DCONF_PROFILE = "shojiwm";
              SHOJIWM_DESKTOP_TASK_MANAGER = cfg.programs.taskManager.command;
              SHOJIWM_DESKTOP_SYSTEM_MONITOR = cfg.programs.systemMonitor.command;
              SHOJIWM_DESKTOP_FILE_MANAGER = cfg.programs.fileManager.command;
              SHOJIWM_DESKTOP_CLIPBOARD = cfg.programs.clipboard.command;
              # "none": the compositor binds no Print Screen key (src/index.tsx).
              SHOJIWM_DESKTOP_SCREENSHOT =
                if cfg.programs.screenCapture.command == ""
                then "none"
                else cfg.programs.screenCapture.command;
            };

            # One account: it is the one the log-on screen logs on.
            shojiwm-desktop-greeter.user = lib.mkIf (lib.length cfg.users == 1) (lib.mkDefault (lib.head cfg.users));

            # The "ShojiWM (UWSM)" session. uwsm stops graphical-session.target
            # when the compositor exits or crashes. The plain "ShojiWM" session
            # leaves it active after a crash, and uwsm then refuses to start
            # another compositor. ShojiWM puts WAYLAND_DISPLAY into the systemd
            # activation environment itself, which is what uwsm waits for.
            #
            # X-NIXOS-SYSTEMD-AWARE stops the NixOS session wrapper from
            # starting nixos-fake-graphical-session.target before uwsm, which
            # then refuses to start. programs.uwsm.waylandCompositors writes no
            # DesktopNames, so the session file is written here.
            displayManager.sessionPackages = [
              ((pkgs.writeTextDir "share/wayland-sessions/shojiwm-uwsm.desktop" ''
                  [Desktop Entry]
                  Name=ShojiWM (UWSM)
                  Comment=ShojiWM managed by UWSM
                  Exec=${lib.getExe config.programs.uwsm.package} start -e -D ShojiWM shojiwm.desktop
                  DesktopNames=ShojiWM;X-NIXOS-SYSTEMD-AWARE
                  Type=Application
                '')
                // {providedSessions = ["shojiwm-uwsm"];})
            ];
          };

          environment = {
            etc =
              {
                # The session's environment, through uwsm's env file for
                # ShojiWM (XDG_CURRENT_DESKTOP lowercased): uwsm exports it to
                # the systemd user manager, which starts the compositor.
                # environment.sessionVariables does not reach it: the manager
                # already has __NIXOS_SET_ENVIRONMENT_DONE, so the login
                # profile skips /etc/set-environment.
                # Folders open in the chosen file manager in this session only.
                # GIO and xdg-mime read <desktop>-mimeapps.list
                # (XDG_CURRENT_DESKTOP lowercased) before mimeapps.list; the
                # other sessions keep theirs.
                "xdg/shojiwm-mimeapps.list" = lib.mkIf (cfg.fileManagerEntry != null) {
                  text = ''
                    [Default Applications]
                    inode/directory=${cfg.fileManagerEntry}
                  '';
                };
                "xdg/uwsm/env-shojiwm".text = lib.concatStrings (lib.mapAttrsToList
                  (name: value: "export ${name}=${lib.escapeShellArg value}\n")
                  (lib.filterAttrs (_: value: value != "") cfg.environment));
              }
              // lib.optionalAttrs hasNetworkManagerApplet {
                # The shell has its own network icon and panel: nm-applet stays
                # out of this session, in the others it still starts.
                # /etc/xdg comes first in XDG_CONFIG_DIRS, so this copy of its
                # autostart entry hides the package's.
                "xdg/autostart/nm-applet.desktop".source = pkgs.runCommand "nm-applet-not-in-shojiwm.desktop" {} ''
                  sed 's/^NotShowIn=\(.*\)$/NotShowIn=\1ShojiWM;/' \
                    ${pkgs.networkmanagerapplet}/etc/xdg/autostart/nm-applet.desktop > $out
                  grep -q '^NotShowIn=.*ShojiWM;' $out
                '';
              };

            # What the desktop itself needs at run time.
            systemPackages =
              [
                # The shell: taskbar, Start menu, desktop, panels, lock screen.
                packages.shell
                packages.icon-theme
                # The Aero icon theme inherits Oxygen.
                pkgs.kdePackages.oxygen-icons
                # The text face of the configuration and the shell.
                pkgs.inter
                # Super+V and the shell's clipboard.
                pkgs.wl-clipboard
              ]
              ++ lib.filter (package: package != null) (map (role: role.package) (lib.attrValues cfg.programs));
          };

          # The GlobalShortcuts portal of shojiwm-desktop: programs such as
          # Hex bind their keys through it. ShojiWM's own portal has only
          # ScreenCast. The name is the desktop's in lowercase:
          # xdg-desktop-portal reads shojiwm-portals.conf, where ShojiWM's
          # module puts its own choices.
          xdg.portal = {
            extraPortals = [packages.portal];
            config.shojiwm."org.freedesktop.impl.portal.GlobalShortcuts" = ["shojiwm-desktop"];
          };

          systemd = {
            # A drop-in on the unit the package installs, as a plain file:
            # NixOS's service options would write it with a PATH of its own
            # (coreutils, grep, sed, systemd), which the portal then lives
            # with; GIO, finding no browser on it, offered no app for a link.
            #
            # xdg-desktop-portal runs only inside a graphical session. It reads
            # the desktop (XDG_CURRENT_DESKTOP) once, as it starts; woken by
            # D-Bus between two sessions it had none, and kept no
            # GlobalShortcuts for the next one. A call before fails; the next
            # one, in the session, starts it right.
            packages = [
              (pkgs.writeTextDir "lib/systemd/user/xdg-desktop-portal.service.d/session.conf" ''
                [Unit]
                Requisite=graphical-session.target
                After=graphical-session.target
              '')
            ];

            user = {
              # xdg-desktop-portal reads the desktop's portals.conf at its start,
              # by XDG_CURRENT_DESKTOP. Between two sessions (the user manager
              # lives on: another login keeps it), a call over D-Bus starts it
              # without that variable, and it then offers no GlobalShortcuts.
              # Once the session's variables are in, before
              # graphical-session.target (the programs that bind keys start
              # after it), it is started again if it runs.
              services.portal-for-shojiwm = {
                description = "xdg-desktop-portal again with the ShojiWM session's variables";
                wantedBy = ["wayland-session@shojiwm.desktop.target"];
                after = ["wayland-wm@shojiwm.desktop.service"];
                before = ["graphical-session.target" "wayland-session@shojiwm.desktop.target"];
                serviceConfig = {
                  Type = "oneshot";
                  ExecStart = "${pkgs.systemd}/bin/systemctl --user try-restart xdg-desktop-portal.service";
                };
              };

              tmpfiles.users = lib.genAttrs cfg.users (user: {
                rules = map (name: "L+ ${config.users.users.${user}.home}/.config/shojiwm/${name} - - - - ${configSource}/${name}") [
                  "src"
                  "assets"
                  "shell"
                ];
              });
            };
          };
        };
      })
    ];
  };
}
