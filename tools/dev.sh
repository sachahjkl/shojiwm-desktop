#!/usr/bin/env bash
# Run ShojiWM nested in a window of the current Wayland session, on this
# checkout.
#
# The configuration paths are fixed to ~/.config/shojiwm (src/index.tsx,
# shell/Apps.qml). The script points src, assets and shell there at this
# checkout, then starts the compositor nested. When the compositor stops, the
# script restores the links that the NixOS module declares, with
# systemd-tmpfiles.
#
#   tools/dev.sh            # start the nested session
#   shoji_wm -r             # reload src/ (from another terminal)
#
# Quickshell reloads the shell itself when a QML file changes.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/shojiwm"
log="${XDG_STATE_HOME:-$HOME/.local/state}/shoji_wm/logs/latest.log"

if [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${DISPLAY:-}" ]; then
  echo "no WAYLAND_DISPLAY or DISPLAY: start this script from a graphical session" >&2
  exit 1
fi

if [ ! -e "$config_dir/node_modules/shoji_wm" ]; then
  echo "$config_dir/node_modules/shoji_wm is missing: run shojiwm-init-config" >&2
  exit 1
fi

restore() {
  if systemd-tmpfiles --user --cat-config 2>/dev/null | grep -q "$config_dir/src"; then
    systemd-tmpfiles --user --create >/dev/null 2>&1 || true
    echo "restored the links of the NixOS module in $config_dir"
  else
    echo "no tmpfiles rule for $config_dir: the links still point at $repo"
  fi
}

for name in src assets shell; do
  if [ -e "$config_dir/$name" ] && [ ! -L "$config_dir/$name" ]; then
    echo "$config_dir/$name is not a link: move it away first" >&2
    exit 1
  fi
  ln -sfn "$repo/$name" "$config_dir/$name"
done
echo "linked src, assets and shell of $config_dir to $repo"
trap restore EXIT

echo "starting ShojiWM nested; log: $log"
shoji_wm "$@"
