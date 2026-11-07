#!/usr/bin/env bash
# Fetch the colour icons the shell uses from Iconify, into shell/assets/icons.
#
# They come from Icons8's "Flat Color Icons" set (MIT licence,
# https://github.com/icons8/flat-color-icons), served by Iconify. They are
# the bright, filled kind of icon Windows 7 drew its places and files with.
#
#   tools/fetch-icons.sh
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$repo/shell/assets/icons"
mkdir -p "$out"

# name in the set -> file name here
icons=(
  "document:documents"
  "download:downloads"
  "picture:pictures"
  "music:music"
  "film-reel:videos"
  "display:computer"
  "settings:control-panel"
  "folder:folder"
  "opened-folder:folder-open"
  "command-line:run"
  "file:file"
  "image-file:file-image"
  "audio-file:file-audio"
  "video-file:file-video"
  "calendar:calendar"
  "empty-trash:recycle-bin"
  "full-trash:recycle-bin-full"
  "home:home"
)

for pair in "${icons[@]}"; do
  name="${pair%%:*}"
  file="${pair##*:}"
  curl -fsS "https://api.iconify.design/flat-color-icons/${name}.svg?width=48&height=48" \
    -o "$out/$file.svg"
  echo "$file.svg <- flat-color-icons:$name"
done

cat >"$out/LICENSE" <<'LICENSE'
The SVG files in this directory are from Icons8's "Flat Color Icons"
(https://github.com/icons8/flat-color-icons), fetched through Iconify by
tools/fetch-icons.sh. Icons8 releases them under the MIT
licence or its own "Good Boy License"; they are used here under MIT.

MIT License

Copyright (c) Icons8

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
LICENSE
