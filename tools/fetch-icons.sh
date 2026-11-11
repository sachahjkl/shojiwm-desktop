#!/usr/bin/env bash
# Build the shell's colour icons from the Oxygen icon theme, into
# shell/assets/icons as 128 px PNG files.
#
# Oxygen (KDE, LGPL-3.0-or-later, https://invent.kde.org/frameworks/oxygen-icons)
# draws glossy, shaded icons of the same era as Windows 7. The folders are
# Oxygen's yellow folder, the colour Windows 7 gives them; the libraries
# (Documents, Pictures, Music, Videos, Downloads, Home) are that folder with
# an Oxygen emblem over its lower right corner.
#
#   tools/fetch-icons.sh
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$repo/shell/assets/icons"
oxygen="$(nix build --no-link --print-out-paths nixpkgs#kdePackages.oxygen-icons)/share/icons/oxygen/base"
magick="$(nix build --no-link --print-out-paths nixpkgs#imagemagick)/bin/magick"
rsvg="$(nix build --no-link --print-out-paths nixpkgs#librsvg)/bin/rsvg-convert"
mkdir -p "$out"

# The largest size Oxygen has of an icon, given as category/name.
largest() {
  for size in 256x256 128x128 64x64 48x48; do
    if [ -f "$oxygen/$size/$1.png" ]; then
      echo "$oxygen/$size/$1.png"
      return
    fi
  done
  echo "missing Oxygen icon: $1" >&2
  exit 1
}

# file name here -> Oxygen icon
plain=(
  "computer:devices/computer"
  "control-panel:categories/preferences-system"
  "calendar:mimetypes/x-office-calendar"
  "run:actions/system-run"
  "file:mimetypes/application-x-zerosize"
  "file-image:mimetypes/image-x-generic"
  "file-audio:mimetypes/audio-x-generic"
  "file-video:mimetypes/video-x-generic"
  "folder:places/folder-yellow"
  "network:places/network-workgroup"
  "network-wired:devices/network-wired"
  "network-wireless:devices/network-wireless"
  "sound:apps/preferences-desktop-sound"
  "sound-mixer:apps/multimedia-volume-control"
  "speaker:actions/speaker"
  "headphones:devices/audio-headphones"
  "headset:devices/audio-headset"
  "microphone:devices/audio-input-microphone"
  "line-in:devices/audio-input-line"
  "sound-card:devices/audio-card"
  "webcam:devices/camera-web"
  "display:devices/video-display"
)
for pair in "${plain[@]}"; do
  name="${pair%%:*}"
  "$magick" "$(largest "${pair#*:}")" -resize 128x128 -strip "$out/$name.png"
  echo "wrote shell/assets/icons/$name.png"
done

# The Recycle Bin is drawn here, not taken from Oxygen: a glass basket, as
# Windows 7 has, from assets/icons-src.
for name in recycle-bin recycle-bin-full; do
  "$rsvg" -w 128 -h 128 "$repo/assets/icons-src/$name.svg" -o "$out/$name.png"
  echo "wrote shell/assets/icons/$name.png"
done

# The open folder: the yellow folder, its front lowered a little.
"$magick" "$(largest places/folder-yellow)" -resize 128x128 \
  -distort Perspective '0,0 0,0  128,0 128,0  0,128 8,128  128,128 120,128' \
  -strip "$out/folder-open.png"
echo "wrote shell/assets/icons/folder-open.png"

# file name here -> emblem over the yellow folder
library=(
  "documents:mimetypes/x-office-document"
  "pictures:mimetypes/image-x-generic"
  "music:mimetypes/audio-x-generic"
  "videos:mimetypes/video-x-generic"
  "downloads:actions/arrow-down"
  "home:places/user-home"
)
for pair in "${library[@]}"; do
  name="${pair%%:*}"
  "$magick" "$(largest places/folder-yellow)" -resize 128x128 \
    \( "$(largest "${pair#*:}")" -resize 72x72 \) \
    -gravity southeast -geometry +6+4 -composite -strip "$out/$name.png"
  echo "wrote shell/assets/icons/$name.png"
done
