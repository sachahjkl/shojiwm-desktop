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
# Everything but Nix is fetched by the script.
#
#   tools/screenshots.sh            # all of them
#   KEEP=1 tools/screenshots.sh     # keep the session running at the end
#   THEME=kernel OUT=/tmp/k UNTIL=desktop tools/screenshots.sh
#                                   # another theme, into another folder, up to one picture
#   SIDE=top tools/screenshots.sh   # the taskbar at the top (default: bottom)
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
out="${OUT:-$repo/assets/site}"

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

  exec nix shell \
    "${compositor[@]}" \
    "nixpkgs#xorg-server" \
    "nixpkgs#xdotool" \
    "nixpkgs#imagemagick" \
    "nixpkgs#foot" \
    "nixpkgs#dbus" \
    "nixpkgs#copyq" \
    "nixpkgs#dconf" \
    "nixpkgs#glib" \
    "${shell[@]}" \
    -c env -u DBUS_SESSION_BUS_ADDRESS -u WAYLAND_DISPLAY -u DISPLAY \
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

# ---------------------------------------------------------------------------
# The libraries the nested backend needs, and the fonts and icons the session
# draws with. Both are found by directory, not by PATH.
# ---------------------------------------------------------------------------

say "collecting libraries, fonts and icons"
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

# A real locale, so the clock and the calendar read as they would on a
# desktop (the C locale writes the date as "7 11 2026"), and foot opens with
# no warning.
LOCALE_ARCHIVE="$(nix build --no-link --print-out-paths "nixpkgs#glibcLocales")/lib/locale/locale-archive"
export LOCALE_ARCHIVE
export LANG=en_GB.UTF-8
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
# The bus started before these were set: the services it starts (dconf above all) must
# write into this session's home, not into the desktop's own settings.
dbus-update-activation-environment HOME USER LOGNAME XDG_CONFIG_HOME XDG_STATE_HOME \
  XDG_DATA_HOME XDG_RUNTIME_DIR LANG LOCALE_ARCHIVE GVFS_DISABLE_FUSE DCONF_PROFILE
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
# shell reads the rest of its settings from their defaults. taskbarCompact is
# written out for the step that switches it with sed.
cat >"$XDG_CONFIG_HOME/shojiwm/settings.json" <<JSON
{
  "theme": "${THEME:-aero}",
  "taskbarSide": "${SIDE:-bottom}",
  "colorSchemeMode": "light",
  "lockAfterMinutes": 0,
  "startPinned": ["foot.desktop", "footclient.desktop"],
  "taskbarPinned": ["foot.desktop", "footclient.desktop"],
  "taskbarCompact": true,
  "trayHidden": ["CopyQ_copyq"]
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

# ---------------------------------------------------------------------------
# The session
# ---------------------------------------------------------------------------

pids=()
cleanup() {
  if [ -n "${KEEP:-}" ]; then
    say "KEEP is set; the session stays up. Work directory: $work"
    return
  fi
  for pid in "${pids[@]:-}"; do
    [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
  done
  rm -rf "$work"
}
trap cleanup EXIT

# Xvfb takes a free display itself and writes its number (-displayfd): several
# sessions can run side by side without taking the same one.
say "starting Xvfb"
exec {display_fd}<> <(:)
Xvfb -displayfd "$display_fd" -screen 0 "${screen}x24" >"$work/xvfb.log" 2>&1 &
pids+=("$!")
read -r -t 30 -u "$display_fd" display_number
exec {display_fd}>&-
display=":$display_number"
export DISPLAY="$display"
say "Xvfb is on $display"
sleep 1

say "starting ShojiWM (nested backend, software rendering)"
export LIBGL_ALWAYS_SOFTWARE=1
# The compositor starts shojiwm-shell; wgpu draws it with Vulkan in software
# (lavapipe).
{
  # Its errors, and the backtrace of a panic, go to $work/shell.log.
  mkdir -p "$work/bin"
  if [ -n "${SHOTS_SHELL_BIN:-}" ]; then
    # The cargo build, with the libraries the Nix wrapper adds.
    cat >"$work/bin/shojiwm-shell" <<SH
#!/bin/sh
export LD_LIBRARY_PATH="$SHOTS_SHELL_LIBS\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
RUST_BACKTRACE=1 exec "$SHOTS_SHELL_BIN" "\$@" 2>>"$work/shell.log"
SH
  else
    printf '#!/bin/sh\nRUST_BACKTRACE=1 exec %s "$@" 2>>"%s"\n' \
      "$(command -v shojiwm-shell)" "$work/shell.log" >"$work/bin/shojiwm-shell"
  fi
  chmod +x "$work/bin/shojiwm-shell"
  if [ -n "${SHOTS_WM_BIN:-}" ]; then
    # The compositor's cargo build, with what nix/package.nix's wrapper sets.
    cat >"$work/bin/shoji_wm" <<SH
#!/bin/sh
export SHOJI_RUNTIME_DIR="$SHOTS_WM_SRC"
export SHOJI_DECORATION_RUNTIME="$SHOTS_WM_SRC/tools/decoration-runtime.ts"
export LD_LIBRARY_PATH="$SHOTS_WM_LIBS\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
export GBM_BACKENDS_PATH="\${GBM_BACKENDS_PATH:+\$GBM_BACKENDS_PATH:}$SHOTS_WM_GBM"
export LIBGL_DRIVERS_PATH="\${LIBGL_DRIVERS_PATH:+\$LIBGL_DRIVERS_PATH:}$SHOTS_WM_DRI"
export __EGL_VENDOR_LIBRARY_DIRS="\${__EGL_VENDOR_LIBRARY_DIRS:+\$__EGL_VENDOR_LIBRARY_DIRS:}$SHOTS_WM_EGL"
exec "$SHOTS_WM_BIN" "\$@"
SH
    chmod +x "$work/bin/shoji_wm"
  fi
  export PATH="$work/bin:$PATH"
  mesa="$(nix build --no-link --print-out-paths nixpkgs#mesa | head -1)"
  export VK_ICD_FILENAMES="$(echo "$mesa"/share/vulkan/icd.d/lvp_icd.*.json)"
}
shoji_wm >"$work/shoji.log" 2>&1 &
pids+=("$!")

log="$XDG_STATE_HOME/shoji_wm/logs/latest.log"
for _ in $(seq 1 40); do
  [ -f "$log" ] && grep -q "control socket listening" "$log" && break
  sleep 0.5
done
if ! grep -q "control socket listening" "$log" 2>/dev/null; then
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
say "nested window $window ($(xdotool getwindowgeometry "$window" | tr '\n' ' '))"

# A command to the shell.
shell_call() {
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

# The site shows WebP at quality 88: about a third of the PNG's size. The lossless capture stays
# in $work, where the crops below are cut from.
shot() {
  local name="$1"
  mkdir -p "$out"
  import -window "$window" "$work/$name-full.png"
  magick "$work/$name-full.png" -strip -quality 88 "$out/$name.webp"
  say "wrote $out/$name.webp"
  if [ "$name" = "${UNTIL:-}" ]; then
    say "$UNTIL: the last picture asked for"
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

# A client window. foot is small, Wayland native and needs no GPU.
# The size is fixed so the detail crops know where every edge is: the frame
# is the client plus 16 px across and 38 px down.
term() {
  local title="$1" size="$2" command="$3"
  foot --title "$title" --font "DejaVu Sans Mono:size=10" \
    --window-size-pixels="$size" sh -c "$command" >/dev/null 2>&1 &
  pids+=("$!")
  sleep 3
}

say "waiting for the shell"
sleep 8

term "Documents" 600x360 "printf 'C:\\\\Users\\\\sacha> dir\\n\\n  Documents\\n  Pictures\\n  Music\\n\\n'; sleep 3600"
term "Notes" 754x531 "printf 'The quick brown fox jumps over the lazy dog.\\n'; sleep 3600"
sleep 2

# An item for the notification area, if one can be had.
if command -v copyq >/dev/null 2>&1; then
  copyq >/dev/null 2>&1 &
  pids+=("$!")
  sleep 4
fi

geometry="$(xdotool getwindowgeometry --shell "$window")"
eval "$geometry"
# The bar: 40 px in Aero, 32 in Kernel, at the bottom unless SIDE=top. bar_y is its middle;
# under a bar at the top, the windows open lower by its height (bar_above).
if [ "${THEME:-aero}" = kernel ]; then bar_h=32; else bar_h=40; fi
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

# Move the front window clear of the one behind it, so both frames show and
# its bottom edge stays above the taskbar.
xdotool mousemove --window "$window" 500 $((25 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 575 $((118 + bar_above))
sleep 1
xdotool mouseup 1
sleep 2

shot "desktop"
# The two frames on their own, cut out of the picture above.
magick "$work/desktop-full.png" -crop "900x620+180+0" +repage \
  -strip -quality 88 "$out/frames.webp"
say "wrote assets/site/frames.webp"

alive "after the first picture"

say "the Start menu"
shell_call startmenu toggle
sleep 3
alive "menu open"
shot "start-menu"

say "a place under the pointer, in the user's tile"
# The Start menu's glass, 560 px high, 2 px from the bar: under it in Aero, over it in Kernel.
if [ "$bar_above" -gt 0 ]; then glass_top=$((bar_h + 2)); else glass_top=$((HEIGHT - bar_h - 2 - 560)); fi
xdotool mousemove --window "$window" 380 $((glass_top + 92))
sleep 2
shot "start-menu-place"

# The menu stands on the taskbar at the left: 490 x 560, 2 px from both.
menu_left=2
menu_bottom=$((glass_top + 560))

say "All Programs, with one folder open"
xdotool mousemove --window "$window" $((menu_left + 60)) $((menu_bottom - 55))
xdotool click 1
sleep 1
xdotool mousemove --window "$window" $((menu_left + 60)) $((menu_bottom - 545))
xdotool click 1
sleep 2
shot "start-menu-all"
alive "all programs"

say "the power menu"
xdotool mousemove --window "$window" $((menu_left + 469)) $((menu_bottom - 23))
xdotool click 1
sleep 2
shot "start-menu-power"
alive "power menu"
xdotool click 1
sleep 1

say "a search that finds nothing"
xdotool type --delay 60 "xyzzy --help"
sleep 2
shot "start-menu-run"
alive "search"
xdotool key Escape
sleep 1
# Closed with a click outside it rather than over IPC, which also tests the
# surface that catches that click.
xdotool mousemove --window "$window" $((WIDTH - 120)) 160
xdotool click 1
sleep 2
alive "menu closed"

say "the taskbar thumbnails"
# Two moves: the button must see the pointer come in, not land on it.
xdotool mousemove --window "$window" 72 "$bar_y"
sleep 0.3
xdotool mousemove --window "$window" 78 "$bar_y"
sleep 3
shot "taskbar-thumbnail"
alive "taskbar-thumbnail"
# Resting on the first card: its window stands out (a gold glow), the pointer going up from
# the button into the cards.
xdotool mousemove --window "$window" 100 "$(from_bar 100)"
sleep 0.3
xdotool mousemove --window "$window" 102 "$(from_bar 103)"
sleep 2
shot "thumbnail-highlight"
alive "thumbnail-highlight"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2

say "the jump list"
xdotool mousemove --window "$window" 78 "$bar_y"
sleep 1
xdotool click 3
sleep 2
shot "jump-list"
alive "jump-list"
# A menu: a press outside closes it (the bar takes no keys: Escape would go to
# the window with the keyboard). On the empty desktop, under the gadgets.
xdotool mousemove --window "$window" 1150 700
xdotool click 1
sleep 2

say "the desktop's menu"
# Pick an icon, then right click on the empty desktop, under the gadgets, and open View.
xdotool mousemove --window "$window" 40 $((140 + bar_above))
xdotool click 1
sleep 1
xdotool mousemove --window "$window" 1150 560
xdotool click 3
sleep 2
xdotool mousemove --window "$window" 1180 578
sleep 0.3
xdotool mousemove --window "$window" 1190 584
sleep 1
xdotool click 1
sleep 2
shot "desktop-menu"
alive "desktop-menu"
xdotool mousemove --window "$window" 1200 600
xdotool click 1
sleep 2

say "the hidden notification-area icons"
# A new icon is hidden, as on Windows 7: the arrow left of the clock opens
# the popover that holds it.
xdotool mousemove --window "$window" $((WIDTH - 245)) "$bar_y"
sleep 1
xdotool click 1
sleep 2
shot "tray-overflow"
alive "tray-overflow"

say "the notification area menu"
# A right click on the icon in the popover; the pointer then rests on the
# menu, as a hand would.
xdotool mousemove --window "$window" $((WIDTH - 251)) "$(from_bar 73)"
sleep 0.3
xdotool mousemove --window "$window" $((WIDTH - 243)) "$(from_bar 81)"
sleep 0.5
xdotool click 3
sleep 0.5
xdotool mousemove --window "$window" $((WIDTH - 243)) "$(from_bar 110)"
sleep 2
shot "tray-menu"
alive "tray-menu"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2
xdotool click 1
sleep 1

say "the calendar"
xdotool mousemove --window "$window" $((WIDTH - 55)) "$bar_y"
sleep 1
xdotool click 1
sleep 2
shot "calendar"
alive "calendar"
xdotool click 1
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2

say "the launcher, Super+Shift+D"
xdotool windowfocus "$window"
shell_call launcher toggle
sleep 4
shot "launcher"
alive "launcher"
xdotool type --delay 80 "fi"
sleep 2
shot "launcher-search"
# Escape closes the launcher, typed text or not.
xdotool key Escape
sleep 2

say "Aero Peek"
xdotool mousemove --window "$window" $((WIDTH - 4)) "$bar_y"
sleep 3
shot "peek"
alive "peek"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2

say "Aero Snap, while dragging to an edge"
# Press on the caption bar of the front window, where the move above put
# it, and drag to the left edge.
xdotool mousemove --window "$window" 575 $((118 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 300 300
sleep 1
xdotool mousemove --window "$window" 2 400
sleep 2
shot "snap"
alive "snap"
xdotool mouseup 1
sleep 2

say "window groups: a caption dropped on another's"
# The snapped window's caption onto the caption of the one behind it: a ghost
# tab while it is held there, then the two as tabs of one frame.
xdotool mousemove --window "$window" 320 $((15 + bar_above))
xdotool mousedown 1
xdotool mousemove --window "$window" 500 $((40 + bar_above))
sleep 0.5
xdotool mousemove --window "$window" 760 $((59 + bar_above))
sleep 2
shot "group-ghost"
xdotool mouseup 1
sleep 2
shot "group-tabs"
alive "group"

say "Alt+Tab"
xdotool windowfocus "$window"
xdotool keydown alt
xdotool key Tab
sleep 3
shot "alt-tab"
alive "alt-tab"
xdotool keyup alt
sleep 2

say "the Control Panel"
qs() { shell_call "$@"; }
qs controlpanel open
sleep 4
shot "control-panel"
alive "control panel"

say "the Control Panel's own scroll bar"
qs controlpanel openPage background
sleep 3
shot "control-panel-scroll"
qs controlpanel open
sleep 1

say "the taskbar with labels"
settings="$XDG_CONFIG_HOME/shojiwm/settings.json"
sed -i 's/"taskbarCompact": true/"taskbarCompact": false/' "$settings"
sleep 3
shot "taskbar-labels"
sed -i 's/"taskbarCompact": false/"taskbarCompact": true/' "$settings"
sleep 2

say "the Control Panel, dark"
qs controlpanel openPage appearance
sleep 2
qs appearance setDark true
sleep 2
shot "control-panel-dark"
alive "dark"
qs controlpanel close
sleep 1
qs startmenu toggle
sleep 3
shot "start-menu-dark"
qs startmenu toggle
sleep 1
qs appearance setDark false
sleep 1
qs controlpanel openPage lockscreen
sleep 2
shot "control-panel-lock"
qs controlpanel close
sleep 1

say "the lock screen"
# Last: the test session has no password to unlock it with.
qs lock lock
sleep 4
shot "lock-screen"
alive "locked"
# The pointer on the round arrow, right of the field under the name.
xdotool mousemove --window "$window" $((WIDTH / 2 + 127)) $((HEIGHT / 2 + 87))
sleep 1
shot "lock-screen-hover"
xdotool type --delay 60 "not my password"
xdotool key Return
sleep 5
shot "lock-screen-wrong"
alive "wrong password"

say "the lock screen, dark"
xdotool key Return
sleep 1
qs appearance setDark true
sleep 3
shot "lock-screen-dark"
alive "dark lock"

# ---------------------------------------------------------------------------
# The same pictures, cut up and blown up, so the finish can be checked
# ---------------------------------------------------------------------------

say "details"
# Pillow, for the script that finds the windows in a capture and cuts them up.
python_env="$(nix build --no-link --print-out-paths --impure --expr \
  'let p = builtins.getFlake "nixpkgs"; pkgs = p.legacyPackages.x86_64-linux;
   in pkgs.python3.withPackages (ps: [ ps.pillow ])')"
py="$python_env/bin/python3"

# The window parts: the script finds each frame by its close button.
"$py" -I "$repo/tools/details.py" "$work/desktop-full.png" "$out/details" \
  front=770x569 back=616x398

# The shell parts, at fixed places.
detail start-menu start-menu-top-left 0 188 28 28
detail start-menu start-menu-search 8 718 170 28 6
detail start-menu start-menu-shutdown 350 718 142 30 6
detail start-menu start-menu-divider 294 300 24 24
detail taskbar-thumbnail taskbar-orb 0 756 56 44 7
detail taskbar-thumbnail taskbar-button 40 758 190 42 5
detail taskbar-thumbnail taskbar-right $((WIDTH - 190)) 756 190 44 5
detail taskbar-thumbnail thumbnail-corner 2 616 40 40
detail alt-tab alt-tab-corner 448 316 48 48
detail jump-list jump-list-corner 40 688 60 40 7
detail tray-menu tray-menu-corner $((WIDTH - 335)) 640 60 40 7

say "done. Pictures are in assets/site"
