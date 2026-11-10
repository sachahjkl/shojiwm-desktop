# The log-on screen end to end, in VMs, through nixosModules.greeter:
#
#   - machine: greetd starts the greeter's Hyprland, which starts
#     greeter.qml; the password is typed; the session starts once that
#     Hyprland has left, and the session chosen is kept;
#   - fallback: Quickshell dies before any log-on; greetd then shows its
#     text log-on, which starts the default session.
{
  pkgs,
  module,
}: let
  # A session that only leaves a mark, and only when its arguments reach it
  # whole, as uwsm's do: greetd joins the command's words before its shell
  # splits them.
  session =
    (pkgs.writeTextDir "share/wayland-sessions/marker.desktop" ''
      [Desktop Entry]
      Name=Marker
      Exec=${pkgs.writeShellScript "marker" ''
        [ "$*" = "start -e marker.desktop" ] || exit 1
        touch /tmp/session-started
        exec sleep infinity
      ''} start -e marker.desktop
      DesktopNames=Marker
      Type=Application
    '')
    // {providedSessions = ["marker"];};

  node = {
    imports = [module];
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
    services.shojiwm-win7-greeter = {
      enable = true;
      user = "alice";
      defaultSession = "marker";
    };
    fonts.packages = [pkgs.inter];
  };
in
  pkgs.testers.runNixOSTest {
    name = "shojiwm-win7-greeter";
    nodes = {
      machine = node;
      fallback = node;
    };

    testScript = ''
      greeter_up = "pgrep -u greeter -f 'quickshell -p /etc/shojiwm-win7/greeter/greeter.qml'"

      # The log-on screen logs on.
      machine.wait_for_unit("greetd.service")
      machine.wait_until_succeeds(greeter_up, timeout=120)
      machine.sleep(10)
      machine.screenshot("greeter")
      machine.send_chars("secret\n")
      machine.wait_for_file("/tmp/session-started", timeout=60)
      machine.wait_until_fails("pgrep -u greeter -x Hyprland", timeout=30)
      # The state folder follows the stable path, so it outlives updates.
      machine.succeed("find /var/lib/greeter/.local/state -name last-session -exec grep -qx marker {} +")

      # A log-on screen that dies gives way to the text log-on.
      fallback.wait_for_unit("greetd.service")
      fallback.wait_until_succeeds(greeter_up, timeout=120)
      fallback.succeed("pkill -KILL -u greeter -x quickshell || pkill -KILL -u greeter -f 'quickshell -p'")
      fallback.wait_until_succeeds("pgrep -u greeter -x agreety", timeout=60)
      fallback.wait_until_tty_matches("1", "login:", timeout=30)
      fallback.send_chars("alice\n")
      fallback.wait_until_tty_matches("1", "assword", timeout=30)
      fallback.send_chars("secret\n")
      fallback.wait_for_file("/tmp/session-started", timeout=60)
    '';
  }
