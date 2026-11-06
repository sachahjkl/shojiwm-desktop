#!/usr/bin/env bash
# Produce the screenshots in assets/site from a real ShojiWM session.
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
#
# Override the Quickshell it uses with QUICKSHELL_FLAKE.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$repo/assets/site"

if [ "${1:-}" != "--inner" ]; then
  exec nix shell \
    "github:bea4dev/ShojiWM#default" \
    "nixpkgs#xorg-server" \
    "nixpkgs#xdotool" \
    "nixpkgs#imagemagick" \
    "${QUICKSHELL_FLAKE:-nixpkgs#quickshell}" \
    "nixpkgs#foot" \
    "nixpkgs#dbus" \
    "nixpkgs#copyq" \
    -c bash "${BASH_SOURCE[0]}" --inner
fi

if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
  # The notification area needs a session bus to find its items.
  exec dbus-run-session -- bash "${BASH_SOURCE[0]}" --inner
fi

say() { printf '==> %s\n' "$*"; }

work="$(mktemp -d /tmp/shojiwm-win7-shots.XXXXXX)"
display=":97"
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

export HOME="$work"
export XDG_CONFIG_HOME="$work/.config"
export XDG_STATE_HOME="$work/.local/state"
export XDG_DATA_HOME="$work/.local/share"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-$work/run}"
mkdir -p "$XDG_RUNTIME_DIR" "$XDG_DATA_HOME/fonts"

# $XDG_DATA_HOME/fonts is read both by fontconfig (Qt, for the shell) and by
# fontdb (cosmic-text, for the window titles).
for path in $fonts; do
  for file in "$path"/share/fonts/*/*.* "$path"/share/fonts/*/*/*.*; do
    [ -e "$file" ] && ln -sf "$file" "$XDG_DATA_HOME/fonts/"
  done
done

data_dirs="$foot_share"
for path in $icons; do data_dirs="$data_dirs:$path/share"; done
export XDG_DATA_DIRS="$data_dirs:${XDG_DATA_DIRS:-/usr/share}"

# ---------------------------------------------------------------------------
# The configuration under test
# ---------------------------------------------------------------------------

say "installing the configuration into $XDG_CONFIG_HOME/shojiwm"
mkdir -p "$XDG_CONFIG_HOME/shojiwm"
cp -R "$repo/src" "$repo/assets" "$repo/shell" "$repo/package.json" \
  "$repo/tsconfig.json" "$XDG_CONFIG_HOME/shojiwm/"
chmod -R u+w "$XDG_CONFIG_HOME/shojiwm"

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

say "starting Xvfb on $display"
Xvfb "$display" -screen 0 "${screen}x24" >"$work/xvfb.log" 2>&1 &
pids+=("$!")
export DISPLAY="$display"
sleep 2

say "starting ShojiWM (nested backend, software rendering)"
export LIBGL_ALWAYS_SOFTWARE=1
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

# Whether the shell is still up. It is the first thing to check when a
# picture comes out without a taskbar.
alive() {
  if pgrep -f "quickshell --path" >/dev/null; then
    say "shell: up ($1)"
  else
    say "shell: GONE ($1)"
  fi
}

shot() {
  local name="$1"
  mkdir -p "$out"
  import -window "$window" "$out/$name.png"
  magick "$out/$name.png" -strip -define png:compression-level=9 "$out/$name.png"
  say "wrote assets/site/$name.png"
}

# A client window. foot is small, Wayland native and needs no GPU.
term() {
  local title="$1" command="$2"
  foot --title "$title" --font "DejaVu Sans Mono:size=10" \
    sh -c "$command" >/dev/null 2>&1 &
  pids+=("$!")
  sleep 3
}

say "waiting for the shell"
sleep 8

term "Documents" "printf 'C:\\\\Users\\\\sacha> dir\\n\\n  Documents\\n  Pictures\\n  Music\\n\\n'; sleep 3600"
term "Notes" "printf 'The quick brown fox jumps over the lazy dog.\\n'; sleep 3600"
sleep 2

# An item for the notification area, if one can be had.
if command -v copyq >/dev/null 2>&1; then
  copyq >/dev/null 2>&1 &
  pids+=("$!")
  sleep 4
fi

geometry="$(xdotool getwindowgeometry --shell "$window")"
eval "$geometry"
bar_y=$((HEIGHT - 20))

# Move the front window clear of the one behind it, so both frames show.
xdotool mousemove --window "$window" 500 42
xdotool mousedown 1
xdotool mousemove --window "$window" 620 230
sleep 1
xdotool mouseup 1
sleep 2

shot "desktop"
# The two frames on their own, cut out of the picture above.
magick "$out/desktop.png" -crop "900x620+180+0" +repage \
  -strip -define png:compression-level=9 "$out/frames.png"
say "wrote assets/site/frames.png"

alive "after the first picture"

say "the Start menu"
quickshell -p "$XDG_CONFIG_HOME/shojiwm/shell" ipc call startmenu toggle || true
sleep 3
alive "menu open"
shot "start-menu"
# Closed with a click outside it rather than over IPC, which also tests the
# surface that catches that click.
xdotool mousemove --window "$window" $((WIDTH - 120)) 160
xdotool click 1
sleep 2
alive "menu closed"

say "the taskbar thumbnails"
xdotool mousemove --window "$window" 120 "$bar_y"
sleep 3
shot "taskbar-thumbnail"
alive "taskbar-thumbnail"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2

say "the jump list"
xdotool mousemove --window "$window" 120 "$bar_y"
sleep 1
xdotool click 3
sleep 2
shot "jump-list"
alive "jump-list"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 2

say "the notification area menu"
xdotool mousemove --window "$window" $((WIDTH - 120)) "$bar_y"
sleep 1
xdotool click 3
sleep 2
shot "tray-menu"
alive "tray-menu"
xdotool mousemove --window "$window" $((WIDTH / 2)) $((HEIGHT / 2))
sleep 1
xdotool click 1
sleep 1

say "Aero Snap, while dragging to an edge"
# Press on the caption bar of the front window, where the move above put
# it, and drag to the left edge.
xdotool mousemove --window "$window" 620 230
xdotool mousedown 1
xdotool mousemove --window "$window" 300 300
sleep 1
xdotool mousemove --window "$window" 2 400
sleep 2
shot "snap"
alive "snap"
xdotool mouseup 1
sleep 2

say "Alt+Tab"
xdotool windowfocus "$window"
xdotool keydown alt
xdotool key Tab
sleep 3
shot "alt-tab"
alive "alt-tab"
xdotool keyup alt
sleep 2

say "done. Pictures are in assets/site"
