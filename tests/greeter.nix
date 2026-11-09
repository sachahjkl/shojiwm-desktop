# The log-on screen end to end, in a VM: greetd starts Hyprland with the
# greeter's Lua configuration, Hyprland starts greeter.qml, a password is
# typed, and the session starts once the greeter's Hyprland has left. The
# nixconfig module of the author starts it the same way.
{
  pkgs,
  win7,
}: let
  inherit (pkgs) lib;

  # A session that only leaves a mark.
  session =
    (pkgs.writeTextDir "share/wayland-sessions/marker.desktop" ''
      [Desktop Entry]
      Name=Marker
      Exec=${pkgs.writeShellScript "marker" "touch /tmp/session-started; exec sleep infinity"}
      DesktopNames=Marker
      Type=Application
    '')
    // {providedSessions = ["marker"];};
in
  pkgs.testers.runNixOSTest {
    name = "shojiwm-win7-greeter";
    nodes.machine = {config, ...}: let
      greeterSession = pkgs.writeShellScript "shojiwm-win7-greeter" ''
        export SHOJIWM_GREETER_USER=alice
        export SHOJIWM_GREETER_SESSIONS=${config.services.displayManager.sessionData.desktops}/share/wayland-sessions
        export SHOJIWM_GREETER_DEFAULT_SESSION=marker
        ${lib.getExe pkgs.quickshell} -p ${win7}/shell/greeter.qml > /tmp/qs.log 2>&1
        ${pkgs.hyprland}/bin/hyprctl dispatch 'hl.dsp.exit()'
      '';
      greeterHyprland = pkgs.writeText "greeter.lua" ''
        hl.on("hyprland.start", function()
            hl.exec_cmd("${greeterSession}")
        end)
        hl.config({
            misc = { disable_hyprland_logo = true, force_default_wallpaper = 0 },
            animations = { enabled = false },
            ecosystem = { no_update_news = true },
        })
      '';
    in {
      virtualisation = {
        memorySize = 3072;
        cores = 2;
        qemu.options = ["-vga none" "-device virtio-gpu-pci"];
      };
      hardware.graphics.enable = true;
      users.users.alice = {
        isNormalUser = true;
        password = "secret";
        description = "Alice Liddell";
      };
      programs.hyprland.enable = true;
      services.displayManager.sessionPackages = [session];
      services.greetd = {
        enable = true;
        settings.default_session.command = "${pkgs.hyprland}/bin/start-hyprland -- --config ${greeterHyprland}";
      };
      users.users.greeter = {
        home = "/var/lib/greeter";
        createHome = true;
      };
      fonts.packages = [pkgs.inter];
    };

    testScript = ''
      try:
          machine.wait_for_unit("greetd.service")
          machine.wait_until_succeeds("grep -q 'Configuration Loaded' /tmp/qs.log", timeout=120)
          machine.sleep(3)
          machine.screenshot("greeter")
          machine.send_chars("secret\n")
          machine.wait_for_file("/tmp/session-started", timeout=60)
          machine.wait_until_fails("pgrep -u greeter -x Hyprland", timeout=30)
          machine.succeed("find /var/lib/greeter/.local/state -name last-session -exec grep -qx marker {} +")
      finally:
          print(machine.execute("grep -v '^$' /tmp/qs.log | tail -40")[1])
          print(machine.execute("journalctl -b --no-pager -o cat | grep -i 'greetd\\|hyprland' | tail -20")[1])
    '';
  }
