#!/usr/bin/env bash
# Produce the screenshots in assets/site from a real ShojiWM session. assets/site is the branch
# site-shots, a worktree (README.md, "Screenshots"): the pictures stay out of main's history.
#
# The session runs on a machine with no display: an X server with no screen
# (Xvfb) holds the window of ShojiWM's nested backend, Mesa renders in
# software, and ImageMagick captures that window. The keys and the pointer are
# driven with xdotool, so every picture is the real configuration doing the
# real thing.
#
# The pictures are published, so the session shows sample data only, never the machine's: a
# made-up account, a private system bus with mock NetworkManager, BlueZ and logind
# (python-dbusmock), a private PipeWire with null devices, a made-up host name and network
# cards (bubblewrap, for the shell), and stub commands for what the shell asks the system
# (systemctl, df, lsusb, lpstat, timedatectl). The stubs also keep the session from powering
# off or logging off the machine it runs on.
#
# Each step waits for a condition with a bound, never for a fixed time: a picture is taken
# once the screen has changed after the action and then stayed still (settle, below).
#
# Everything but Nix is fetched by the script.
#
#   tools/screenshots.sh            # all of them, Aero, into assets/site
#   THEME=all tools/screenshots.sh  # Aero, then Kernel (into assets/site/kernel): one session
#                                   # at a time
#   KEEP=1 tools/screenshots.sh     # keep the session running at the end (THEME=all: the
#                                   # Kernel one)
#   THEME=kernel OUT=/tmp/k UNTIL=desktop tools/screenshots.sh
#                                   # another theme, into another folder, up to one picture
#   SIDE=top tools/screenshots.sh   # the taskbar at the top (default: bottom)
#
# Each shot is written twice: AVIF (smaller, for browsers that read it) and WebP (for the
# others). The close-ups stay lossless WebP. The end of a run prints the size of both sets.
#
# The shell is this flake's (crates/shell). The compositor can be overridden
# with SHOJIWM_FLAKE (a local clone: SHOJIWM_FLAKE=path:/src/ShojiWM#default).
# The default is this flake's packages.shojiwm: the integration branch of the
# fork pinned in flake.lock, which carries what this configuration uses before
# upstream has it (labels' textShadow).
#
# BUILD says how the shell and the compositor are built:
#
#   BUILD=cargo (default)  the shell with `cargo build` (dev profile, its own
#                          target dir), in this flake's `shell` dev shell: a
#                          change rebuilds in seconds. The compositor is
#                          SHOJIWM_FLAKE's, as with nix, unless SHOJIWM_SRC
#                          names a ShojiWM checkout: then `cargo build
#                          --release -p shoji_wm` there, in its dev shell,
#                          into ~/.cache/shojiwm-target, so only the crates
#                          that changed rebuild.
#   BUILD=nix              both as Nix packages, in release: every change
#                          rebuilds all of it (15 to 20 minutes). SHOJIWM_SRC
#                          is not read.
#
#   SHOJIWM_SRC=~/src/ShojiWM tools/screenshots.sh
#                                   # a compositor being worked on
#   BUILD=nix tools/screenshots.sh  # the pictures that are published
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
theme="${THEME:-aero}"
# The site takes the Kernel captures from assets/site/kernel.
case "$theme" in
  aero) out="${OUT:-$repo/assets/site}" ;;
  kernel) out="${OUT:-$repo/assets/site/kernel}" ;;
  all) ;;
  *)
    printf 'THEME is aero, kernel or all, not %s\n' "$theme" >&2
    exit 1
    ;;
esac

# The total size of the shots of some folders (not the close-ups), as AVIF and as WebP, so
# that the two encodings can be compared.
sizes() {
  local avif=0 webp=0 count=0 dir file
  for dir in "$@"; do
    for file in "$dir"/*.avif; do
      [ -e "$file" ] || continue
      avif=$((avif + $(stat -c %s "$file")))
      count=$((count + 1))
    done
    for file in "$dir"/*.webp; do
      [ -e "$file" ] || continue
      webp=$((webp + $(stat -c %s "$file")))
    done
  done
  printf '==> sizes: %s shots, AVIF %s KiB, WebP %s KiB\n' "$count" $((avif / 1024)) $((webp / 1024))
}

if [ "$theme" = all ]; then
  base="${OUT:-$repo/assets/site}"
  # One session after the other: each takes room in /tmp, which is a small tmpfs. Only the
  # last one may stay up, or the two would run side by side.
  KEEP='' THEME=aero OUT="$base" bash "${BASH_SOURCE[0]}"
  THEME=kernel OUT="$base/kernel" bash "${BASH_SOURCE[0]}"
  sizes "$base" "$base/kernel"
  exit 0
fi

if [ "${1:-}" != "--inner" ]; then
  shojiwm_flake="${SHOJIWM_FLAKE:-path:$repo#shojiwm}"
  # The compositor first and the shell last in PATH, as they always were.
  compositor=("$shojiwm_flake")
  shell=("path:$repo#shell")
  case "${BUILD:-cargo}" in
    nix) ;;
    cargo)
      # What a dev shell gives the build, read back from a file: a dev shell
      # may write to stdout itself (ShojiWM's greets).
      env_file="$(mktemp)"

      # The shell: the binary, and the libraries the Nix wrapper puts on its
      # path (the dev shell's LD_LIBRARY_PATH is the same list). The
      # programs the wrapper adds (dconf, gio, setsid, xdg-open) are in the
      # nix shell below.
      printf '==> %s\n' "building the shell (cargo, dev profile)"
      nix develop "$repo#shell" -c bash -c \
        'cargo build --manifest-path "$1/crates/shell/Cargo.toml" &&
         printf "%s\n" "$LD_LIBRARY_PATH" >"$2"' \
        bash "$repo" "$env_file"
      SHOTS_SHELL_BIN="$repo/crates/shell/target/debug/shojiwm-shell"
      SHOTS_SHELL_LIBS="$(cat "$env_file")"
      export SHOTS_SHELL_BIN SHOTS_SHELL_LIBS
      shell=("nixpkgs#util-linux" "nixpkgs#xdg-utils")

      # The compositor, from a checkout. Its wrapper (nix/package.nix) is
      # remade below, from what its dev shell holds: the runtime's files
      # are the checkout's own (packages/shoji_wm, tools/), and
      # xwayland-satellite is built in.
      if [ -n "${SHOJIWM_SRC:-}" ]; then
        SHOTS_WM_SRC="$(cd "$SHOJIWM_SRC" && pwd)"
        target="$HOME/.cache/shojiwm-target"
        printf '==> %s\n' "building ShojiWM in $SHOTS_WM_SRC (cargo, release, into $target)"
        nix develop "$SHOTS_WM_SRC" -c bash -c \
          'cd "$1" && CARGO_TARGET_DIR="$2" cargo build --release -p shoji_wm &&
           printf "%s\n" "$LD_LIBRARY_PATH" "$GBM_BACKENDS_PATH" \
             "$LIBGL_DRIVERS_PATH" "$__EGL_VENDOR_LIBRARY_DIRS" >"$3"' \
          bash "$SHOTS_WM_SRC" "$target" "$env_file"
        {
          read -r SHOTS_WM_LIBS
          read -r SHOTS_WM_GBM
          read -r SHOTS_WM_DRI
          read -r SHOTS_WM_EGL
        } <"$env_file"
        SHOTS_WM_BIN="$target/release/shoji_wm"
        export SHOTS_WM_SRC SHOTS_WM_BIN SHOTS_WM_LIBS SHOTS_WM_GBM SHOTS_WM_DRI SHOTS_WM_EGL
        # The wrapper puts Xwayland and dbus on PATH; dbus is below.
        compositor=("nixpkgs#xwayland")
      fi
      rm -f "$env_file"
      ;;
    *)
      printf 'BUILD is cargo or nix, not %s\n' "$BUILD" >&2
      exit 1
      ;;
  esac

  # The caller's buses, sound server and display must not reach the session: it has its own.
  # papier is the flake's own input (the desktop's clipboard history): --inputs-from takes
  # the version its lock pins.
  exec nix shell --inputs-from "path:$repo" \
    "${compositor[@]}" \
    "nixpkgs#xorg-server" \
    "nixpkgs#xdotool" \
    "nixpkgs#imagemagick" \
    "nixpkgs#libavif" \
    "nixpkgs#foot" \
    "nixpkgs#dbus" \
    "papier" \
    "nixpkgs#dconf" \
    "nixpkgs#glib" \
    "nixpkgs#pipewire" \
    "nixpkgs#bubblewrap" \
    "${shell[@]}" \
    -c env -u DBUS_SESSION_BUS_ADDRESS -u DBUS_SYSTEM_BUS_ADDRESS -u WAYLAND_DISPLAY \
    -u DISPLAY -u PIPEWIRE_REMOTE -u PIPEWIRE_RUNTIME_DIR -u XDG_SESSION_ID \
    -u SHOJIWM_SOCKET -u SHOJIWM_DESKTOP_KEYS -u SHOJIWM_DESKTOP_SOCKET \
    bash "${BASH_SOURCE[0]}" --inner
fi

if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
  # The notification area needs a session bus to find its items.
  # The bus's own configuration, for a machine with no /etc/dbus-1.
  dbus_conf="$(dirname "$(command -v dbus-daemon)")/../share/dbus-1/session.conf"
  if [ -e /etc/dbus-1/session.conf ] || [ ! -e "$dbus_conf" ]; then
    exec dbus-run-session -- bash "${BASH_SOURCE[0]}" --inner
  fi
  exec dbus-run-session --config-file="$dbus_conf" -- bash "${BASH_SOURCE[0]}" --inner
fi

say() { printf '==> %s\n' "$*"; }

work="$(mktemp -d /tmp/shojiwm-desktop-shots.XXXXXX)"
screen="1600x1000"

# The time now, in milliseconds, into now_ms: no process, so a wait can look often.
now_ms=0
tick() {
  local t="${EPOCHREALTIME/[.,]/}"
  now_ms=$((t / 1000))
}

# wait_for WHAT SECONDS COMMAND...: runs COMMAND every 100 ms until it succeeds. Gives up after
# SECONDS and fails.
wait_for() {
  local what="$1" seconds="$2" deadline
  shift 2
  tick
  deadline=$((now_ms + seconds * 1000))
  until "$@" >/dev/null 2>&1; do
    tick
    if [ "$now_ms" -ge "$deadline" ]; then
      say "$what: not ready after ${seconds}s"
      return 1
    fi
    sleep 0.1
  done
}

# ---------------------------------------------------------------------------
# The libraries the nested backend needs, and the fonts and icons the session
# draws with. Both are found by directory, not by PATH.
# ---------------------------------------------------------------------------

say "collecting libraries, fonts, icons and the GTK themes"
libs="$(nix build --no-link --print-out-paths \
  "nixpkgs#xorg.libX11" \
  "nixpkgs#xorg.libXcursor" \
  "nixpkgs#xorg.libXrandr" \
  "nixpkgs#xorg.libXi" \
  "nixpkgs#libglvnd")"
for path in $libs; do
  [ -d "$path/lib" ] && LD_LIBRARY_PATH="$path/lib:${LD_LIBRARY_PATH:-}"
done
export LD_LIBRARY_PATH

fonts="$(nix build --no-link --print-out-paths "nixpkgs#inter" "nixpkgs#dejavu_fonts")"
icons="$(nix build --no-link --print-out-paths \
  "nixpkgs#adwaita-icon-theme" "nixpkgs#hicolor-icon-theme")"
foot_share="$(nix build --no-link --print-out-paths "nixpkgs#foot")/share"
# The GTK programs shown in the desktop's look: the widget factories are in the dev outputs,
# the schemas they read in the main ones.
gtk_theme="$(nix build --no-link --print-out-paths "path:$repo#gtk-theme")"
gtk3_dev="$(nix build --no-link --print-out-paths "nixpkgs#gtk3.dev")"
gtk4_dev="$(nix build --no-link --print-out-paths "nixpkgs#gtk4.dev")"
# GTK 3 stops when it can load no icon at all: the icon themes, and librsvg's loaders for
# their SVG pictures.
pixbuf_loaders="$(nix build --no-link --print-out-paths "nixpkgs#librsvg" | head -1)/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
gtk_icons=""
for path in $icons; do gtk_icons="$gtk_icons:$path/share"; done
# GTK 4's factory has a video, and stops when GStreamer has no playbin3.
gst_plugins="$(nix build --no-link --print-out-paths "nixpkgs#gst_all_1.gst-plugins-base" | head -1)/lib/gstreamer-1.0"
gtk_schemas=""
for path in $(nix build --no-link --print-out-paths "nixpkgs#gtk3" "nixpkgs#gtk4"); do
  for dir in "$path"/share/gsettings-schemas/*; do
    [ -d "$dir" ] && gtk_schemas="$gtk_schemas:$dir"
  done
done
# Python with the mock services, and with Pillow for the close-ups.
python_env="$(nix build --no-link --print-out-paths --impure --expr \
  'let p = builtins.getFlake "nixpkgs"; pkgs = p.legacyPackages.${builtins.currentSystem};
   in pkgs.python3.withPackages (ps: [ ps.pillow ps.python-dbusmock ps.pygobject3 ])')"
py="$python_env/bin/python3"
tzdata="$(nix build --no-link --print-out-paths "nixpkgs#tzdata")"

# A real locale, so the clock and the calendar read as they would on a
# desktop (the C locale writes the date as "7 11 2026"), and foot opens with
# no warning.
LOCALE_ARCHIVE="$(nix build --no-link --print-out-paths "nixpkgs#glibcLocales")/lib/locale/locale-archive"
export LOCALE_ARCHIVE
export LANG=en_GB.UTF-8
# A zone of the sample data (timedatectl below), not the machine's.
export TZDIR="$tzdata/share/zoneinfo" TZ=Europe/London
export HOME="$work"
# A made-up account, so the pictures show no one's name or picture: the
# shell reads the account's full name (getent) and AccountsService's
# picture by $USER, and this one has neither.
export USER=alex LOGNAME=alex
export XDG_CONFIG_HOME="$work/.config"
export XDG_STATE_HOME="$work/.local/state"
export XDG_DATA_HOME="$work/.local/share"
# A runtime folder of its own and no Wayland display: run from a desktop
# session, the nested backend would open on that session instead of Xvfb,
# and the shell's IPC would reach the desktop's own shell.
export XDG_RUNTIME_DIR="$work/run"
unset WAYLAND_DISPLAY
mkdir -p "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME/fonts"
# A dconf profile of its own: only this session's database, not the system's defaults (a
# desktop's own configuration may set a dark scheme there).
printf 'user-db:user\n' >"$work/dconf-profile"
export DCONF_PROFILE="$work/dconf-profile"
# No FUSE mount of gvfs in the runtime folder: the session could not remove it at the end.
export GVFS_DISABLE_FUSE=1
# The system bus of the session (below): never the machine's, whose services know its real
# networks, devices and power.
export DBUS_SYSTEM_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/system_bus_socket"
# The bus started before these were set: the services it starts (dconf above all) must
# write into this session's home, not into the desktop's own settings.
dbus-update-activation-environment HOME USER LOGNAME XDG_CONFIG_HOME XDG_STATE_HOME \
  XDG_DATA_HOME XDG_RUNTIME_DIR LANG LOCALE_ARCHIVE GVFS_DISABLE_FUSE DCONF_PROFILE \
  DBUS_SYSTEM_BUS_ADDRESS TZ TZDIR
chmod 700 "$XDG_RUNTIME_DIR"

# $XDG_DATA_HOME/fonts is read both by fontconfig (Qt, for the shell) and by
# fontdb (cosmic-text, for the window titles).
for path in $fonts; do
  for file in "$path"/share/fonts/*/*.* "$path"/share/fonts/*/*/*.*; do
    [ -e "$file" ] && ln -sf "$file" "$XDG_DATA_HOME/fonts/"
  done
done

data_dirs="$foot_share"
for path in $icons; do data_dirs="$data_dirs:$path/share"; done
# Only these: the caller's would list the programs installed on its machine.
export XDG_DATA_DIRS="$data_dirs"

# ---------------------------------------------------------------------------
# The configuration under test
# ---------------------------------------------------------------------------

say "installing the configuration into $XDG_CONFIG_HOME/shojiwm"
mkdir -p "$XDG_CONFIG_HOME/shojiwm"
cp -R "$repo/src" "$repo/assets" "$repo/shell" "$repo/package.json" \
  "$repo/tsconfig.json" "$XDG_CONFIG_HOME/shojiwm/"
chmod -R u+w "$XDG_CONFIG_HOME/shojiwm"

# The programs pinned by default (Thunar, Chromium, kitty) are not installed
# here. Pin two that are, so the taskbar shows one pinned program with
# windows open and one with none, and the Start menu has a pinned list. The
# two terminals stay two windows (TERMINAL_TABS=true: one, as tabs), for the
# detail crops that know where every edge is. The
# shell reads the rest of its settings from their defaults. taskbarCompact is
# written out for the step that switches it with sed.
cat >"$XDG_CONFIG_HOME/shojiwm/settings.json" <<JSON
{
  "theme": "$theme",
  "taskbarSide": "${SIDE:-bottom}",
  "colorSchemeMode": "light",
  "lockAfterMinutes": 0,
  "startPinned": ["foot.desktop", "footclient.desktop"],
  "taskbarPinned": ["foot.desktop", "footclient.desktop"],
  "taskbarCompact": true,
  "terminalTabs": ${TERMINAL_TABS:-false},
  "trayHidden": ["papier", "gui_toolkit"]
}
JSON

# Two dozen pictures in ~/Pictures, for the Control Panel's background chooser
# to show (and to need its scroll bar for).
mkdir -p "$HOME/Pictures"
for i in $(seq -w 1 24); do
  hue=$((10#$i * 15))
  magick -size 640x400 "gradient:hsl($hue,60%,35%)-hsl($((hue + 40)),70%,60%)" \
    "$HOME/Pictures/sample-$i.png"
done

# A few things on the desktop: a shortcut, a folder, a picture, a text file.
mkdir -p "$HOME/Desktop/Projects"
cp "$HOME/Pictures/sample-07.png" "$HOME/Desktop/Holiday.png"
printf 'Remember the milk.\n' >"$HOME/Desktop/Shopping list.txt"
cat >"$HOME/Desktop/foot.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Terminal
Exec=foot
Icon=utilities-terminal
DESKTOP

# Nothing here opens files through GTK, so no program records them. Write the
# list a desktop keeps of them, naming foot, for the jump list to read.
mkdir -p "$XDG_DATA_HOME"
for name in notes.txt todo.md shojiwm.log; do
  printf 'sample\n' >"$work/$name"
done
{
  printf '<?xml version="1.0" encoding="UTF-8"?>\n'
  printf '<xbel version="1.0" xmlns:bookmark="http://www.freedesktop.org/standards/desktop-bookmarks">\n'
  minute=10
  for name in notes.txt todo.md shojiwm.log; do
    printf '  <bookmark href="file://%s/%s" added="2026-11-07T10:%s:00Z" modified="2026-11-07T10:%s:00Z" visited="2026-11-07T10:%s:00Z">\n' \
      "$work" "$name" "$minute" "$minute" "$minute"
    printf '    <info><metadata owner="http://freedesktop.org"><bookmark:applications>\n'
    printf '      <bookmark:application name="foot" exec="&apos;foot %%u&apos;" modified="2026-11-07T10:%s:00Z" count="1"/>\n' "$minute"
    printf '    </bookmark:applications></metadata></info>\n'
    printf '  </bookmark>\n'
    minute=$((minute + 5))
  done
  printf '</xbel>\n'
} >"$XDG_DATA_HOME/recently-used.xbel"

# The notification history the bell's panel reads at start: three sample notifications, two
# of them not seen yet.
now_s="$(date +%s)"
mkdir -p "$XDG_STATE_HOME/shojiwm-desktop"
cat >"$XDG_STATE_HOME/shojiwm-desktop/notifications.json" <<JSON
{"unread": 2, "entries": [
  {"app": "Calendar", "icon": "x-office-calendar", "summary": "Team meeting in 15 minutes", "body": "Room 2, with Sam and Robin.", "time": $(((now_s - 120) * 1000))},
  {"app": "Downloads", "icon": "folder-download", "summary": "Download complete", "body": "holiday-photos.zip (48 MB) is in your Downloads folder.", "time": $(((now_s - 1500) * 1000))},
  {"app": "Software Updates", "icon": "system-software-update", "summary": "Updates installed", "body": "Restart the computer to finish installing them.", "time": $(((now_s - 86400 * 2) * 1000))}
]}
JSON

# The system's autostart folder the Control Panel's Startup page lists (only the shell reads
# it, see its wrapper): two sample programs instead of what the machine has installed.
mkdir -p "$work/xdg/autostart"
cat >"$work/xdg/autostart/papier.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Papier
Comment=The clipboard's history
Exec=papier --hidden
Icon=papier
DESKTOP
cat >"$work/xdg/autostart/foot-server.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Foot Server
Comment=A terminal server, so new terminals open at once
Exec=foot --server
Icon=utilities-terminal
DESKTOP

# ---------------------------------------------------------------------------
# What the shell asks the system, answered with sample data
# ---------------------------------------------------------------------------

mkdir -p "$work/bin"
export PATH="$work/bin:$PATH"

# A stub: bin/NAME, from the text on standard input.
stub() {
  cat >"$work/bin/$1"
  chmod +x "$work/bin/$1"
}

# The Action Center's checks read one failed service and one disk almost full. Nothing in
# the session may power off, restart or suspend this machine; the rest goes to systemctl.
stub systemctl <<SH
#!/bin/sh
case " \$* " in
  *" --failed "*)
    case " \$* " in *" --user "*) echo "backup.service loaded failed failed Nightly backup" ;; esac
    exit 0 ;;
  *" poweroff "*|*" reboot "*|*" halt "*|*" kexec "*|*" suspend "*|*" hibernate "*|*" hybrid-sleep "*|*" suspend-then-hibernate "*)
    exit 0 ;;
esac
PATH="\${PATH#"$work/bin:"}" exec systemctl "\$@"
SH
# Log off would end this machine's own session.
stub loginctl <<'SH'
#!/bin/sh
exit 0
SH
stub df <<'SH'
#!/bin/sh
printf 'Filesystem     1024-blocks      Used Available Capacity Mounted on\n'
printf '/dev/sda2        487652352 199937464 263010040      44%% /\n'
printf '/dev/sdb1        976284672 917707592  58576880      94%% /data\n'
SH
# Devices and Printers: three USB devices and two printers (`lsusb`, `lpstat -p` and `-d`).
stub lsusb <<'SH'
#!/bin/sh
printf 'Bus 001 Device 002: ID 1d6b:0101 USB Keyboard\n'
printf 'Bus 001 Device 003: ID 1d6b:0102 Optical Mouse\n'
printf 'Bus 002 Device 002: ID 1d6b:0103 HD Webcam\n'
SH
stub lpstat <<'SH'
#!/bin/sh
case "$1" in
  -p)
    printf 'printer Office_Printer is idle.  enabled since Mon 16 Nov 2026 09:00:00\n'
    printf 'printer Photo_Printer is idle.  enabled since Mon 16 Nov 2026 09:00:00\n' ;;
  -d) printf 'system default destination: Office_Printer\n' ;;
esac
SH
# Date and Time: the zone of TZ above, and a short list to choose from.
stub timedatectl <<'SH'
#!/bin/sh
case "$1" in
  show)
    printf 'Timezone=Europe/London\nLocalRTC=no\nCanNTP=yes\nNTP=yes\nNTPSynchronized=yes\n' ;;
  list-timezones)
    printf '%s\n' America/New_York America/Los_Angeles Asia/Tokyo Australia/Sydney \
      Europe/Berlin Europe/London Europe/Paris UTC ;;
esac
SH

# ---------------------------------------------------------------------------
# The session
# ---------------------------------------------------------------------------

pids=()
cleanup() {
  if [ -n "${KEEP:-}" ] && [ "$KEEP" != hold ]; then
    say "KEEP is set; the session stays up. Work directory: $work"
    return
  fi
  for pid in "${pids[@]:-}"; do
    [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
  done
  # What the session started on its own (the portals, which D-Bus activation detaches)
  # outlives the processes above: anything with this session's runtime folder is its own.
  for pid in $(pgrep -u "$(id -u)"); do
    # This script has the folder too.
    [ "$pid" = "$$" ] && continue
    if tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null | grep -qx "XDG_RUNTIME_DIR=$work/run"; then
      kill "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$work"
}
trap cleanup EXIT

say "starting the private system bus and its mock services"
cat >"$work/system-bus.conf" <<XML
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <listen>unix:path=$XDG_RUNTIME_DIR/system_bus_socket</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow user="*"/>
    <allow own="*"/>
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
  </policy>
</busconfig>
XML
dbus-daemon --config-file="$work/system-bus.conf" --nofork --nopidfile \
  >"$work/system-bus.log" 2>&1 &
pids+=("$!")
wait_for "the system bus" 10 test -S "$XDG_RUNTIME_DIR/system_bus_socket"
for template in networkmanager bluez5 logind; do
  "$py" -m dbusmock --system --template "$template" >"$work/mock-$template.log" 2>&1 &
  pids+=("$!")
done

# A call to a mock: mock_call BUS-NAME OBJECT METHOD ARGS... (GVariant text, typed by the
# method's signature). A failure leaves that part of the sample data out.
mock_call() {
  local name="$1" object="$2" method="$3"
  shift 3
  gdbus call --system --dest "$name" --object-path "$object" --method "$method" "$@" \
    >/dev/null 2>>"$work/mock-calls.log"
}

# The network: Wi-Fi on "Home Network" with three others in reach, and a cable not plugged.
nm=/org/freedesktop/NetworkManager
if wait_for "NetworkManager (mock)" 60 gdbus introspect --system --dest org.freedesktop.NetworkManager --object-path /org/freedesktop; then
  nm_mock() { mock_call org.freedesktop.NetworkManager /org/freedesktop "org.freedesktop.DBus.Mock.$1" "${@:2}"; }
  nm_mock AddEthernetDevice "'eth0'" "'eth0'" 20 || true
  nm_mock AddWiFiDevice "'wlan0'" "'wlan0'" 100 || true
  wifi="$nm/Devices/wlan0"
  # name, SSID, address, mode (infrastructure), frequency, rate, strength, security (WPA-PSK
  # or none)
  nm_mock AddAccessPoint "'$wifi'" "'home'" "'Home Network'" "'02:00:00:00:00:01'" 2 5180 866000 86 256 || true
  nm_mock AddAccessPoint "'$wifi'" "'cafe'" "'Café Wi-Fi'" "'02:00:00:00:00:02'" 2 2437 144000 61 0 || true
  nm_mock AddAccessPoint "'$wifi'" "'office'" "'Office Guest'" "'02:00:00:00:00:03'" 2 2462 144000 44 256 || true
  nm_mock AddAccessPoint "'$wifi'" "'upstairs'" "'Upstairs'" "'02:00:00:00:00:04'" 2 5240 433000 23 256 || true
  nm_mock AddWiFiConnection "'$wifi'" "'home'" "'Home Network'" "'wpa-psk'" || true
  nm_mock AddActiveConnection "['$wifi']" "'$nm/Settings/home'" "'$nm/AccessPoint/home'" "'home'" 2 || true
  # The mock does not say which access point a device is on; the shell reads it here.
  # The template's wireless device has no such property: it is added on the device itself.
  mock_call org.freedesktop.NetworkManager "$wifi" org.freedesktop.DBus.Mock.AddProperty \
    "'org.freedesktop.NetworkManager.Device.Wireless'" "'ActiveAccessPoint'" \
    "<objectpath '$nm/AccessPoint/home'>" || true
  # Nor that Wi-Fi is on and the machine online (full connectivity, globally connected): the
  # panel would say "Not connected" and "Wireless is turned off" over the list.
  nm_root() { mock_call org.freedesktop.NetworkManager "$nm" org.freedesktop.DBus.Mock.SetProperty "'$nm'" "'org.freedesktop.NetworkManager'" "$@"; }
  nm_root "'WirelessEnabled'" "<true>" || true
  nm_root "'WirelessHardwareEnabled'" "<true>" || true
  nm_root "'Connectivity'" "<uint32 4>" || true
  nm_root "'State'" "<uint32 70>" || true
fi

# Bluetooth: headphones and a mouse connected (with their batteries), a keyboard paired, and a
# phone in reach. While there is an adapter, the notification area has one more icon (32 px:
# the icon, its room and the gap), so the places left of it move by that much.
tray_shift=0
bluez_device() { printf '/org/bluez/hci0/dev_%s' "${1//:/_}"; }
if wait_for "BlueZ (mock)" 60 gdbus introspect --system --dest org.bluez --object-path /; then
  bz_mock() { mock_call org.bluez / "org.bluez.Mock.$1" "${@:2}"; }
  if bz_mock AddAdapter "'hci0'" "'alex-pc'"; then
    tray_shift=32
  fi
  # address, name, icon, paired, connected, battery
  while read -r address name icon paired connected battery; do
    name="${name//_/ }"
    bz_mock AddDevice "'hci0'" "'$address'" "'$name'" || continue
    path="$(bluez_device "$address")"
    mock_call org.bluez "$path" org.freedesktop.DBus.Properties.Set \
      "'org.bluez.Device1'" "'Icon'" "<'$icon'>" || true
    if [ "$paired" = yes ]; then
      bz_mock PairDevice "'hci0'" "'$address'" || true
    fi
    if [ "$connected" = yes ]; then
      bz_mock ConnectDevice "'hci0'" "'$address'" || true
    fi
    if [ "$battery" != - ]; then
      mock_call org.bluez "$path" org.freedesktop.DBus.Mock.AddProperties \
        "'org.bluez.Battery1'" "{'Percentage': <byte $battery>}" || true
    fi
  done <<'DEVICES'
02:00:00:00:10:01 Headphones audio-headphones yes yes 80
02:00:00:00:10:02 Mouse input-mouse yes yes 55
02:00:00:00:10:03 Keyboard input-keyboard yes no -
02:00:00:00:10:04 Phone phone no no -
DEVICES
fi
# logind: the power menu and the Shut Down dialog ask it what the machine can do. The mock
# says yes to all and does nothing.
wait_for "logind (mock)" 60 gdbus introspect --system --dest org.freedesktop.login1 \
  --object-path /org/freedesktop/login1 || true

say "starting PipeWire with sample devices"
# Speakers (the default), headphones and a microphone, as null devices: no sound card of the
# machine shows. No session manager either: the configuration sets the defaults itself.
mkdir -p "$XDG_CONFIG_HOME/pipewire/pipewire.conf.d"
cat >"$XDG_CONFIG_HOME/pipewire/pipewire.conf.d/50-sample-devices.conf" <<'CONF'
context.objects = [
  { factory = adapter
    args = { factory.name = support.null-audio-sink node.name = "speakers"
             node.description = "Speakers" media.class = "Audio/Sink"
             audio.position = [ FL FR ] monitor.channel-volumes = true object.linger = true } }
  { factory = adapter
    args = { factory.name = support.null-audio-sink node.name = "headphones"
             node.description = "Headphones" media.class = "Audio/Sink"
             audio.position = [ FL FR ] monitor.channel-volumes = true object.linger = true } }
  { factory = adapter
    args = { factory.name = support.null-audio-sink node.name = "microphone"
             node.description = "Microphone" media.class = "Audio/Source"
             audio.position = [ MONO ] object.linger = true } }
  { factory = metadata
    args = { metadata.name = default
             metadata.values = [
               { key = default.audio.sink value = { name = "speakers" } }
               { key = default.configured.audio.sink value = { name = "speakers" } }
               { key = default.audio.source value = { name = "microphone" } }
               { key = default.configured.audio.source value = { name = "microphone" } }
             ] } }
]
CONF
pipewire >"$work/pipewire.log" 2>&1 &
pids+=("$!")
if wait_for "PipeWire" 15 test -S "$XDG_RUNTIME_DIR/pipewire-0"; then
  # A program that plays, for the Sound window's Playback page. Nothing links it, so it waits
  # on its silence and makes no sound.
  pw-cat --playback --raw --format s16 --rate 48000 --channels 2 \
    -P '{ application.name = "Music Player" media.name = "Evening Playlist" }' - \
    </dev/zero >"$work/pw-cat.log" 2>&1 &
  pids+=("$!")
fi

say "starting Xvfb"
# Xvfb takes a free display itself and writes its number (-displayfd): several
# sessions can run side by side without taking the same one. It answers once it is ready.
exec {display_fd}<> <(:)
Xvfb -displayfd "$display_fd" -screen 0 "${screen}x24" >"$work/xvfb.log" 2>&1 &
pids+=("$!")
read -r -t 30 -u "$display_fd" display_number
exec {display_fd}>&-
display=":$display_number"
export DISPLAY="$display"
say "Xvfb is on $display"

say "starting ShojiWM (nested backend, software rendering)"
export LIBGL_ALWAYS_SOFTWARE=1
# The compositor starts shojiwm-shell; wgpu draws it with Vulkan in software
# (lavapipe).
{
  # The shell runs in a namespace of its own when bubblewrap can make one: a made-up host
  # name (the System and Network pages show it) and made-up network cards, among them a
  # wireless one, so the network panel lists the sample networks (the shell looks for a
  # wireless card in sysfs). Its system autostart folder is the sample one above.
  mkdir -p "$work/net/lo" "$work/net/eth0" "$work/net/wlan0/wireless"
  sandbox=""
  if bwrap --dev-bind / / --unshare-uts --hostname alex-pc --bind "$work/net" /sys/class/net \
    true 2>/dev/null; then
    sandbox="bwrap --dev-bind / / --unshare-uts --hostname alex-pc --bind $work/net /sys/class/net --"
  else
    say "warning: no bubblewrap namespace; the network panel shows no Wi-Fi list, and the" \
      "System and Network pages are not taken (they would show this machine's name)"
  fi
  # Its errors, and the backtrace of a panic, go to $work/shell.log.
  if [ -n "${SHOTS_SHELL_BIN:-}" ]; then
    # The cargo build, with the libraries the Nix wrapper adds.
    shell_bin="$SHOTS_SHELL_BIN"
    shell_libs="export LD_LIBRARY_PATH=\"$SHOTS_SHELL_LIBS\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}\""
  else
    shell_bin="$(command -v shojiwm-shell)"
    shell_libs=""
  fi
  stub shojiwm-shell <<SH
#!/bin/sh
$shell_libs
export XDG_CONFIG_DIRS="$work/xdg"
RUST_BACKTRACE=1 exec $sandbox "$shell_bin" "\$@" 2>>"$work/shell.log"
SH
  if [ -n "${SHOTS_WM_BIN:-}" ]; then
    # The compositor's cargo build, with what nix/package.nix's wrapper sets.
    stub shoji_wm <<SH
#!/bin/sh
export SHOJI_RUNTIME_DIR="$SHOTS_WM_SRC"
export SHOJI_DECORATION_RUNTIME="$SHOTS_WM_SRC/tools/decoration-runtime.ts"
export LD_LIBRARY_PATH="$SHOTS_WM_LIBS\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
export GBM_BACKENDS_PATH="\${GBM_BACKENDS_PATH:+\$GBM_BACKENDS_PATH:}$SHOTS_WM_GBM"
export LIBGL_DRIVERS_PATH="\${LIBGL_DRIVERS_PATH:+\$LIBGL_DRIVERS_PATH:}$SHOTS_WM_DRI"
export __EGL_VENDOR_LIBRARY_DIRS="\${__EGL_VENDOR_LIBRARY_DIRS:+\$__EGL_VENDOR_LIBRARY_DIRS:}$SHOTS_WM_EGL"
exec "$SHOTS_WM_BIN" "\$@"
SH
  fi
  mesa="$(nix build --no-link --print-out-paths nixpkgs#mesa | head -1)"
  export VK_ICD_FILENAMES="$(echo "$mesa"/share/vulkan/icd.d/lvp_icd.*.json)"
}
shoji_wm >"$work/shoji.log" 2>&1 &
pids+=("$!")

log="$XDG_STATE_HOME/shoji_wm/logs/latest.log"
if ! wait_for "ShojiWM" 30 grep -q "control socket listening" "$log"; then
  say "ShojiWM did not start; see $work/shoji.log"
  tail -20 "$work/shoji.log" || true
  exit 1
fi

WAYLAND_DISPLAY="$(sed -n 's/.*shojiwm\.\(wayland-[0-9]*\)\.sock.*/\1/p' "$log" | head -1)"
export WAYLAND_DISPLAY
say "ShojiWM is on $WAYLAND_DISPLAY"

window="$(xdotool search --onlyvisible --class . | tail -1)"
# With no window manager nothing holds the X input focus, so the keys sent
# below would go nowhere.
xdotool windowfocus "$window"
geometry="$(xdotool getwindowgeometry --shell "$window")"
eval "$geometry"
say "nested window $window (${WIDTH}x${HEIGHT})"

# The bar: 40 px in Aero, 32 in Kernel, at the bottom unless SIDE=top. bar_y is its middle;
# under a bar at the top, the windows open lower by its height (bar_above).
if [ "$theme" = kernel ]; then bar_h=32; else bar_h=40; fi
if [ "${SIDE:-bottom}" = top ]; then
  bar_y=$((bar_h / 2))
  bar_above=$bar_h
else
  bar_y=$((HEIGHT - bar_h / 2))
  bar_above=0
fi
# A height `d` above Aero's bottom bar (40 px), as it is from this bar: the popups of the bar
# open away from it.
from_bar() {
  if [ "$bar_above" -gt 0 ]; then echo $((bar_h + $1 - 40)); else echo $((HEIGHT - ($1 - 40 + bar_h))); fi
}

# ---------------------------------------------------------------------------
# Waiting for the screen to settle
# ---------------------------------------------------------------------------

# The desktop's gadgets move by themselves (the clock's seconds hand, the processor meter
# every 1.5 s): settle looks at the screen without them. Both stand at the top right
# (desktop/gadgets.rs), the meter 24 px and the clock 52 px from the right edge.
mask="rectangle $((WIDTH - 236)),$((bar_above + 16)) $((WIDTH - 12)),$((bar_above + 316))"
# How many pixels of the half-size look may change for the screen to count as still: a text
# caret blinks, nothing else should move.
still_pixels=15
# The longest a step waits for the screen, in milliseconds.
settle_limit=8000

# Captures the nested window into frame.ppm, and its look (half size, the gadgets masked) into
# look.ppm.
capture() {
  import -silent -window "$window" "$work/frame.ppm"
  magick "$work/frame.ppm" -fill black -draw "$mask" -sample 50% "$work/look.ppm"
}

# How many pixels of the half-size look differ between two looks.
changed_pixels() {
  magick compare -metric AE -fuzz 2% "$1" "$2" null: 2>&1 >/dev/null |
    awk '{ printf "%d", $1 }' || true
}

# Whether two looks differ by more than a blinking caret, or by more than $3 pixels.
differs() {
  local pixels
  pixels="$(changed_pixels "$1" "$2")"
  [ "${pixels:-0}" -gt "${3:-$still_pixels}" ]
}

# The least change of the half-size look that counts as what a step opened: a panel, a menu or
# a dialog is thousands of pixels, a tooltip or a hover a few hundred. A smaller change (a
# tooltip of the button under the pointer, a surface shown before it is painted) does not end
# the wait for the real one.
opened_pixels=3000

# The screen just before the last action (a key, a click, a move, a command to the shell): what
# settle waits to see change. The last settled picture is no reference: a step often closes the
# panel before it opens the next, and that close alone would count as the change.
mark() {
  [ -n "${window:-}" ] || return 0
  capture
  cp "$work/look.ppm" "$work/settled.ppm"
}

# Every action of the steps marks the screen first. A search for the window does not.
xdotool() {
  case "$1" in
  key | keydown | keyup | type | click | mousedown | mouseup | mousemove) mark ;;
  esac
  command xdotool "$@"
}

# settle [QUIET] [CHANGE] [LIMIT]: waits until the screen has changed from the screen before the
# last action (mark; CHANGE=1, the default; 0 for an action that may show nothing) and has then stayed
# still for QUIET ms (default 1500: the shell's fades last 90 to 120 ms and a panel of the
# notification area waits 300 ms for a double click, but a new surface shows first, nearly
# empty, and is painted up to a second later in software on a loaded machine; an action that
# closes one panel and opens another also changes the screen twice). After LIMIT ms (default
# settle_limit)
# it goes on with a warning. The last capture is in frame.ppm.
settle() {
  local quiet="${1:-1500}" change="${2:-1}" limit="${3:-$settle_limit}"
  local start run_start=-1 changed=0 least
  # CHANGE: 0 none, 1 what a step opens (opened_pixels), else that many pixels (a hover).
  case "$change" in
  0 | 1) least=$opened_pixels ;;
  *) least=$change ;;
  esac
  [ -e "$work/settled.ppm" ] || changed=1
  tick
  start=$now_ms
  while :; do
    capture
    tick
    if [ "$changed" = 0 ] && differs "$work/look.ppm" "$work/settled.ppm" "$least"; then
      changed=1
    fi
    # A run: captures like its first one. It is long enough once it has lasted QUIET.
    if [ "$run_start" -ge 0 ] && ! differs "$work/look.ppm" "$work/run.ppm"; then
      if { [ "$changed" = 1 ] || [ "$change" = 0 ]; } && [ $((now_ms - run_start)) -ge "$quiet" ]; then
        break
      fi
    else
      cp "$work/look.ppm" "$work/run.ppm"
      run_start=$now_ms
    fi
    if [ $((now_ms - start)) -ge "$limit" ]; then
      if [ "$changed" = 0 ] && [ "$change" = 1 ]; then
        say "warning: the screen did not change within ${limit} ms"
      else
        say "warning: the screen did not settle within ${limit} ms"
      fi
      break
    fi
    sleep 0.05
  done
  cp "$work/look.ppm" "$work/settled.ppm"
}

# Waits for the screen (settle's QUIET, CHANGE and LIMIT), then writes it as $out/NAME.avif and
# $out/NAME.webp. The lossless capture stays in $work, where the crops below are cut from.
#
# AVIF: 4:4:4, as chroma at half resolution makes coloured text bleed; quality 65, where the
# 12 px text of the menus stays clean at about two thirds of the size of WebP at 88; speed 6,
# a fifth of a second a picture.
shot() {
  local name="$1"
  shift
  settle "$@"
  mkdir -p "$out"
  magick "$work/frame.ppm" "$work/$name-full.png"
  magick "$work/$name-full.png" -strip -quality 88 "$out/$name.webp"
  avifenc -q 65 -s 6 -y 444 -d 8 -j all "$work/$name-full.png" "$out/$name.avif" >/dev/null
  say "wrote $out/$name.avif and .webp"
  if [ "$name" = "${UNTIL:-}" ]; then
    say "$UNTIL: the last picture asked for"
    sizes "$out"
    # KEEP=hold: the whole session stays up, its buses too (they end with this script), while
    # $work/hold exists, an hour at most: remove the file to end it.
    if [ "${KEEP:-}" = hold ]; then
      touch "$work/hold"
      say "holding the session until $work/hold is removed (an hour at most)"
      wait_for "the end of the hold" 3600 test ! -e "$work/hold" || true
    fi
    exit 0
  fi
}

# A piece of a capture, blown up with no smoothing so every pixel shows. Lossless WebP: lossy
# compression would blur the pixels that the crop is there to show.
#   detail <source shot> <name> <x> <y> <w> <h> [zoom]
detail() {
  local src="$1" name="$2" x="$3" y="$4" w="$5" h="$6" zoom="${7:-8}"
  mkdir -p "$out/details"
  magick "$work/$src-full.png" -crop "${w}x${h}+${x}+${y}" +repage \
    -filter point -resize "${zoom}00%" \
    -strip -define webp:lossless=true "$out/details/$name.webp"
  say "wrote assets/site/details/$name.webp"
}

# A command to the shell.
#
# What a command opens does not need the pointer, which a step may have left on a button: its
# tooltip comes and goes, and would count as the change. The pointer goes to the empty
# wallpaper under the gadgets first, and the screen settles before the mark.
shell_call() {
  if [ -n "${window:-}" ]; then
    command xdotool mousemove --window "$window" $((WIDTH - 130)) $((bar_above + 420))
    settle 400 0 3000
  fi
  mark
  shojiwm-shell msg "$@" || true
}

# Whether the shell is still up. It is the first thing to check when a
# picture comes out without a taskbar. The shell runs from bin/ (Nix) or
# target/debug/ (cargo).
alive() {
  if pgrep -f '/shojiwm-shell$' >/dev/null; then
    say "shell: up ($1)"
  else
    say "shell: GONE ($1)"
  fi
}

# The pointer to x y, then the screen settled (settle's QUIET, default 200: a hover's veil).
# The nested window may not draw the pointer, so a move over nothing changes no pixel: no
# change is required.
point() {
  xdotool mousemove --window "$window" "$1" "$2"
  settle "${3:-200}" 0
}

# Rests the pointer on x y the way a hand comes: first beside it, then on it, so that what is
# there sees the pointer come in rather than land on it.
rest() {
  xdotool mousemove --window "$window" $(($1 - 6)) $(($2 - 3))
  settle 200 0
  xdotool mousemove --window "$window" "$1" "$2"
}

# Before a panel that opens with a click: a press outside an open popup closes it and goes no
# further, so a panel left open (the calendar, the notifications, a slow one shown late)
# would eat the click meant for the next. A click on the empty wallpaper closes whatever is
# open; with nothing open it changes nothing.
close_panels() {
  command xdotool mousemove --window "$window" $((WIDTH - 130)) $((bar_above + 420))
  command xdotool click 1
  settle 400 0 3000
}

# A client window. foot is small, Wayland native and needs no GPU.
# The size is fixed so the detail crops know where every edge is: the frame
# is the client plus 16 px across and 38 px down.
term() {
  local title="$1" size="$2" command="$3"
  foot --title "$title" --font "DejaVu Sans Mono:size=10" \
    --window-size-pixels="$size" sh -c "$command" >/dev/null 2>&1 &
  pids+=("$!")
  settle 500 1 15000
}

say "waiting for the shell"
# The Control Panel's target is among the last the shell installs.
wait_for "the shell" 60 shojiwm-shell msg controlpanel close || true
settle 800 1 20000

term "Documents" 600x360 "printf 'C:\\\\Users\\\\sacha> dir\\n\\n  Documents\\n  Pictures\\n  Music\\n\\n'; sleep 3600"
term "Notes" 754x531 "printf 'The quick brown fox jumps over the lazy dog.\\n'; sleep 3600"

# An item for the notification area, if one can be had.
# Papier, the desktop's clipboard history, hidden in the notification area: the steps of the
# hidden icons and of the tray menu need its item. Its history is the session's, empty. Wait
# until the shell (the StatusNotifierWatcher) has its item, then for the arrow, a small change.
tray_items() {
  gdbus call --session --dest org.kde.StatusNotifierWatcher --object-path /StatusNotifierWatcher \
    --method org.freedesktop.DBus.Properties.Get org.kde.StatusNotifierWatcher \
    RegisteredStatusNotifierItems | grep -o "'[^']*'" | wc -l
}
if command -v papier >/dev/null 2>&1; then
  # The sample autostart folder may have started it already (then this one hands over to it and
  # leaves): it is the session's only item, so one item is it.
  papier --hidden >"$work/papier.log" 2>&1 &
  pids+=("$!")
  papier_item() { [ "$(tray_items 2>/dev/null || echo 0)" -ge 1 ]; }
  # Up to a minute: it starts slowly in software on a loaded machine.
  if wait_for "Papier's notification-area item" 60 papier_item; then
    settle 500 40 5000
  else
    say "papier: $(tail -5 "$work/papier.log" | tr '\n' ' ')"

  fi
fi

# The places of the notification area, from the right: the show-desktop strip (15 px), the
# clock (74) and, 8 px apart, the shell's icons of 24 px each (status/mod.rs): volume,
# network, Bluetooth while there is an adapter, notifications, Action Center; then the tray
# with its arrow.
network_x=$((WIDTH - 149))
bell_x=$((WIDTH - 181 - tray_shift))
arrow_x=$((WIDTH - 245 - tray_shift))
clock_x=$((WIDTH - 55))

# Move the front window clear of the one behind it, so both frames show and
# its bottom edge stays above the taskbar.
xdotool mousemove --window "$window" 500 $((25 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 575 $((118 + bar_above))
settle 300
# The release may change nothing on the screen.
xdotool mouseup 1
shot "desktop" 500 0
# The two frames on their own, cut out of the picture above.
magick "$work/desktop-full.png" -crop "900x620+180+0" +repage "$work/frames-full.png"
magick "$work/frames-full.png" -strip -quality 88 "$out/frames.webp"
avifenc -q 65 -s 6 -y 444 -d 8 -j all "$work/frames-full.png" "$out/frames.avif" >/dev/null
say "wrote $out/frames.avif and .webp"

alive "after the first picture"

say "the Start menu"
shell_call startmenu toggle
shot "start-menu"
alive "menu open"

say "a place under the pointer, in the user's tile"
# The Start menu, 490 x 476 in both looks (measured on 16 Nov 2026), 2 px from the bar: under it
# in Aero, over it in Kernel. Its places are counted from its bottom, where its list ends.
menu_h=476
if [ "$bar_above" -gt 0 ]; then glass_top=$((bar_h + 2)); else glass_top=$((HEIGHT - bar_h - 2 - menu_h)); fi
menu_left=2
menu_bottom=$((glass_top + menu_h))
# Documents, in the column of places.
xdotool mousemove --window "$window" 340 $((menu_bottom - 394))
shot "start-menu-place" 900 60

say "All Programs, with one folder open"
xdotool mousemove --window "$window" $((menu_left + 60)) $((menu_bottom - 58))
xdotool click 1
settle
# The list's first folder.
xdotool mousemove --window "$window" $((menu_left + 60)) $((glass_top + 23))
xdotool click 1
shot "start-menu-all"
alive "all programs"

say "the power menu"
# The arrow beside Shut down.
xdotool mousemove --window "$window" $((menu_left + 408)) $((menu_bottom - 24))
xdotool click 1
shot "start-menu-power" 900
alive "power menu"
xdotool click 1
settle

say "a search that finds nothing"
# The menu opened again, with the keyboard in its search field: the clicks above leave it on
# the lists, with the power menu open.
shell_call startmenu close
settle 400 0 3000
shell_call startmenu open
settle
# Typed through the session's French layout: no letter that AZERTY moves (a, q, w, z, m), no
# punctuation.
xdotool type --delay 60 "nothing here"
shot "start-menu-run"
alive "search"
xdotool key Escape
settle 300 0
# Closed with a click outside it rather than over IPC, which also tests the
# surface that catches that click.
xdotool mousemove --window "$window" $((WIDTH - 120)) 160
xdotool click 1
settle 600 0
# Closed for sure: a menu left open would take the next steps' clicks.
shell_call startmenu close
settle 400 0 3000
alive "menu closed"

say "the taskbar thumbnails"
# The cards come 350 ms after the pointer: the screen must stay still longer than that.
rest 78 "$bar_y"
shot "taskbar-thumbnail" 900
alive "taskbar-thumbnail"
# Resting on the first card: its window stands out (a gold glow), the pointer going up from
# the button into the cards.
xdotool mousemove --window "$window" 100 "$(from_bar 100)"
settle 200 0
xdotool mousemove --window "$window" 102 "$(from_bar 103)"
shot "thumbnail-highlight" 900 60
alive "thumbnail-highlight"
point $((WIDTH / 2)) $((HEIGHT / 2)) 500

say "the jump list"
close_panels
point 78 "$bar_y" 900
xdotool click 3
shot "jump-list"
alive "jump-list"
# A menu: a press outside closes it (the bar takes no keys: Escape would go to
# the window with the keyboard). On the empty desktop, under the gadgets.
xdotool mousemove --window "$window" 1150 700
xdotool click 1
settle

say "the desktop's menu"
close_panels
# Pick an icon, then right click on the empty desktop, under the gadgets, and open View.
xdotool mousemove --window "$window" 40 $((140 + bar_above))
xdotool click 1
settle
xdotool mousemove --window "$window" 1150 560
xdotool click 3
settle
rest 1190 584
settle 300 0
xdotool click 1
shot "desktop-menu"
alive "desktop-menu"
xdotool mousemove --window "$window" 1200 600
xdotool click 1
settle

say "the hidden notification-area icons"
close_panels
# A new icon is hidden, as on Windows 7: the arrow left of the shell's icons opens the
# popover that holds it.
point "$arrow_x" "$bar_y"
xdotool click 1
shot "tray-overflow"
alive "tray-overflow"

say "the notification area menu"
# A right click on the icon in the popover; the pointer then rests on the
# menu, as a hand would.
# The icon stands about 50 px above the bar in both looks (49 in Aero, 53 in Kernel).
if [ "$bar_above" -gt 0 ]; then icon_y=$((bar_h + 51)); else icon_y=$((HEIGHT - bar_h - 51)); fi
xdotool mousemove --window "$window" $((arrow_x - 6)) $((icon_y + 6))
settle 200 0
xdotool mousemove --window "$window" $((arrow_x + 2)) "$icon_y"
settle 300 0
xdotool click 3
settle
xdotool mousemove --window "$window" $((arrow_x + 2)) "$(from_bar 110)"
shot "tray-menu" 900 60
alive "tray-menu"
point $((WIDTH / 2)) $((HEIGHT / 2)) 500
xdotool click 1
settle


say "the network panel"
close_panels
# A click waits 300 ms for a double click, then opens the panel.
point "$network_x" "$bar_y"
xdotool click 1
shot "tray-network"
alive "tray-network"
close_panels

say "the volume panel, its outputs, and the Sound window"
shell_call volume open
shot "tray-volume"
shell_call volume openDevices
shot "tray-volume-devices"
shell_call volume close
settle
shell_call sound open playback
shot "sound-playback"
shell_call sound open outputs
shot "sound-outputs"
shell_call sound close
settle
alive "sound"

if [ "$tray_shift" -gt 0 ]; then
  say "the Bluetooth panel"
  shell_call bluetooth open
  shot "tray-bluetooth"
  shell_call bluetooth close
  settle
fi

say "the notifications"
close_panels
point "$bell_x" "$bar_y"
xdotool click 1
shot "tray-notifications"
close_panels

say "the Action Center"
shell_call actioncenter open
shot "tray-action-center"
shell_call actioncenter close
settle
alive "action center"

say "the calendar"
close_panels
point "$clock_x" "$bar_y"
xdotool click 1
shot "calendar"
alive "calendar"
xdotool click 1
settle
point $((WIDTH / 2)) $((HEIGHT / 2)) 500

say "the launcher, Super+Shift+D"
close_panels
xdotool windowfocus "$window"
shell_call launcher toggle
shot "launcher"
alive "launcher"
# (not a letter pair of the pictures' file names, which the search would list: the session's
# home folder is the work folder; and no letter that the session's AZERTY layout moves)
xdotool type --delay 80 "printer"
shot "launcher-search"
# Escape closes the launcher, typed text or not.
xdotool key Escape
settle

say "the Run dialog"
shell_call run open
shot "run-dialog"
shell_call run close
settle

say "the administration menu, Super+X"
shell_call adminmenu toggle
shot "admin-menu"
shell_call adminmenu close
settle
alive "dialogs"

say "Aero Peek"
# The windows turn to outlines 500 ms after the pointer comes.
xdotool mousemove --window "$window" $((WIDTH - 4)) "$bar_y"
shot "peek" 900
alive "peek"
point $((WIDTH / 2)) $((HEIGHT / 2)) 500

say "Aero Snap, while dragging to an edge"
# Press on the caption bar of the front window, where the move above put
# it, and drag to the left edge.
xdotool mousemove --window "$window" 575 $((118 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 300 300
settle 300
xdotool mousemove --window "$window" 2 400
shot "snap" 900
alive "snap"
xdotool mouseup 1
settle

say "window groups: a caption dropped on another's"
# The snapped window's caption onto the caption of the one behind it: a ghost
# tab while it is held there, then the two as tabs of one frame.
xdotool mousemove --window "$window" 320 $((15 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 500 $((40 + bar_above))
settle 300
xdotool mousemove --window "$window" 760 $((59 + bar_above))
shot "group-ghost" 900
xdotool mouseup 1
shot "group-tabs"
alive "group"

# The close button of the frame with the keyboard, in a capture: the largest patch of strong
# red (both looks draw it red), as "x0 y0 x1 y1".
red_box() {
  magick "$1" -fx '(r > 0.43 && r - g > 0.18 && r - b > 0.18) ? 1 : 0' -colorspace gray \
    -threshold 50% -define connected-components:verbose=true \
    -define connected-components:area-threshold=150 -connected-components 4 null: |
    awk '$NF ~ /(255|65535|100%)/ && $(NF - 1) + 0 > best { best = $(NF - 1) + 0; box = $2 }
      END {
        if (box == "") exit 1
        split(box, p, /[x+]/)
        print p[3], p[4], p[3] + p[1] - 1, p[4] + p[2] - 1
      }'
}

if box="$(red_box "$work/group-tabs-full.png")"; then
  read -r x0 y0 x1 y1 <<<"$box"
  # The frame's right edge and the middle of its caption, from the close button; the first
  # tab starts at the frame's left (the group has the size of the window it was dropped on:
  # 600 px of client) and is 220 px wide. groups.ts, themes/*/metrics.ts.
  if [ "$theme" = kernel ]; then
    right=$((x1 + 2))
    caption_y=$(((y0 + y1) / 2))
    tab_x=$((right - 604 + 9 + 110))
  else
    right=$((x1 + 9))
    caption_y=$((y0 + 13))
    tab_x=$((right - 616 + 11 + 110))
  fi

  say "the preview of a tab not shown"
  # The tab first, the window dropped last: its window is the one shown.
  rest "$tab_x" "$caption_y"
  shot "tab-preview" 900
  point $((WIDTH / 2)) $((HEIGHT / 2)) 500

  say "closing a window with tabs"
  point $(((x0 + x1) / 2)) $(((y0 + y1) / 2))
  xdotool click 1
  shot "close-tabs"
  shell_call closegroup close
  settle
  point $((WIDTH / 2)) $((HEIGHT / 2)) 500
  alive "close tabs"
else
  say "warning: no close button found in group-tabs; no tab-preview or close-tabs"
fi

say "Alt+Tab"
xdotool windowfocus "$window"
xdotool keydown alt
xdotool key Tab
shot "alt-tab"
alive "alt-tab"
xdotool keyup alt
settle

say "the Control Panel"
shell_call controlpanel open
shot "control-panel"
alive "control panel"

say "the Control Panel's own scroll bar"
shell_call controlpanel openPage background
shot "control-panel-scroll"

say "the other pages of the Control Panel"
# System and Network show the computer's name: only with the made-up one.
for page in appearance account taskbar notifications network sound display devices defaults \
  startup clock ease about; do
  if [ -z "$sandbox" ] && { [ "$page" = network ] || [ "$page" = about ]; }; then
    continue
  fi
  shell_call controlpanel openPage "$page"
  shot "control-panel-$page"
done
alive "control panel pages"
shell_call controlpanel open
settle

say "the taskbar with labels"
settings="$XDG_CONFIG_HOME/shojiwm/settings.json"
sed -i 's/"taskbarCompact": true/"taskbarCompact": false/' "$settings"
shot "taskbar-labels"
sed -i 's/"taskbarCompact": false/"taskbarCompact": true/' "$settings"
settle

say "the Control Panel, dark"
shell_call controlpanel openPage appearance
settle
shell_call appearance setDark true
shot "control-panel-dark"
alive "dark"
shell_call controlpanel close
settle
shell_call startmenu toggle
shot "start-menu-dark"
shell_call startmenu toggle
settle
shell_call appearance setDark false
settle
shell_call controlpanel openPage lockscreen
shot "control-panel-lock"
shell_call controlpanel close
settle

say "GTK programs in the desktop's look"
# The look's own GTK theme (packages.gtk-theme), light as the session is, drawn with Cairo:
# GTK 4's GL renderer on a software GPU is slow and gains nothing here. No session bus: GTK 3
# waited on it (portals, accessibility, its application name) past the step's bound, and the
# factories need nothing from it.
if [ "$theme" = kernel ]; then gtk_name=Kernel; else gtk_name=Aero; fi
for program in "$gtk3_dev/bin/gtk3-widget-factory" "$gtk4_dev/bin/gtk4-widget-factory"; do
  env -u DBUS_SESSION_BUS_ADDRESS GTK_THEME="$gtk_name" GDK_BACKEND=wayland GSK_RENDERER=cairo \
    GDK_PIXBUF_MODULE_FILE="$pixbuf_loaders" GST_PLUGIN_SYSTEM_PATH_1_0="$gst_plugins" \
    GTK_USE_PORTAL=0 GDK_DEBUG=no-portals NO_AT_BRIDGE=1 GTK_A11Y=none \
    XDG_DATA_DIRS="$gtk_theme/share$gtk_schemas$gtk_icons:$XDG_DATA_DIRS" \
    "$program" >>"$work/gtk.log" 2>&1 &
  gtk_pid=$!
  pids+=("$gtk_pid")
  shot "$(basename "$program")" 800 1 20000
  kill "$gtk_pid" 2>/dev/null || true
  settle
done
alive "gtk"

say "Share your screen"
# What the screen-cast portal hands the picker: this screen and the two terminals. The
# windows' identifiers are made up, so their cards show the program's icon.
share_request='{"app_id":"foot","prefer":"output","sources":[
  {"id":"winit","kind":"output","name":"winit","description":"Nested screen","width":1280,"height":800},
  {"id":"window-1","kind":"window","app_id":"foot","title":"Documents"},
  {"id":"window-2","kind":"window","app_id":"foot","title":"Notes"}]}'
printf '%s\n' "$share_request" | shojiwm-shell pick-source >/dev/null 2>&1 &
pick_pid=$!
pids+=("$pick_pid")
shot "share-screen"
# Ctrl+Tab switches to the other tab; Escape cancels, and the picker ends.
xdotool key ctrl+Tab
shot "share-screen-windows"
xdotool key Escape
settle
# The dialog closes with its caller, if Escape did not reach it. In the namespace, the caller
# is the child of bwrap, which a kill of bwrap leaves running.
if ! wait_for "the screen picker" 5 sh -c "! kill -0 $pick_pid"; then
  pkill -P "$pick_pid" 2>/dev/null || true
  kill "$pick_pid" 2>/dev/null || true
  settle
fi
alive "share"

say "Shut Down"
shell_call shutdown openOn ""
shot "shutdown-dialog"
shell_call shutdown close
settle

say "the lock screen"
# Last: the test session has no password to unlock it with.
shell_call lock lock
shot "lock-screen" 800
alive "locked"
# The pointer on the round arrow, right of the field under the name.
xdotool mousemove --window "$window" $((WIDTH / 2 + 127)) $((HEIGHT / 2 + 87))
shot "lock-screen-hover"
xdotool type --delay 60 "not my password"
settle
xdotool key Return
# The check may hold the screen still a moment before the answer shows.
shot "lock-screen-wrong" 2000 1 12000
alive "wrong password"

say "the lock screen, dark"
xdotool key Return
settle 300 0
shell_call appearance setDark true
shot "lock-screen-dark"
alive "dark lock"

# ---------------------------------------------------------------------------
# The same pictures, cut up and blown up, so the finish can be checked
# ---------------------------------------------------------------------------

# The site shows the close-ups of Aero only.
if [ "$theme" = aero ]; then
  say "details"
  # The window parts: the script finds each frame by its close button.
  "$py" -I "$repo/tools/details.py" "$work/desktop-full.png" "$out/details" \
    front=770x569 back=616x398

  # The shell parts, at fixed places.
  detail start-menu start-menu-top-left 0 $((glass_top - 10)) 28 28
  detail start-menu start-menu-search 8 $((menu_bottom - 40)) 170 28 6
  detail start-menu start-menu-shutdown 305 $((menu_bottom - 40)) 142 30 6
  detail start-menu start-menu-divider 294 300 24 24
  detail taskbar-thumbnail taskbar-orb 0 756 56 44 7
  detail taskbar-thumbnail taskbar-button 40 758 190 42 5
  detail taskbar-thumbnail taskbar-right $((WIDTH - 190)) 756 190 44 5
  detail taskbar-thumbnail thumbnail-corner 2 616 40 40
  detail alt-tab alt-tab-corner 448 316 48 48
  detail jump-list jump-list-corner 40 688 60 40 7
  detail tray-menu tray-menu-corner $((WIDTH - 335 - tray_shift)) 640 60 40 7
fi

sizes "$out"
say "done. Pictures are in $out"
