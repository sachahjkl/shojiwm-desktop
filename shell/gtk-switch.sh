#!/bin/sh
# Move the GTK theme to its light or dark variant, for the Control Panel's
# light or dark choice. GTK 3 programs read the theme, not the colour scheme.
#
#   gtk-switch.sh DCONF KEY dark|light [LOOK]
#
# With the desktop's themes (SHOJIWM_DESKTOP_GTK_THEMES, by default Kernel,
# Kernel-dark, Aero and Aero-dark), the theme becomes the light or dark one
# of the desktop's look: LOOK, else the "theme" of the
# shell's settings.json, else kernel. A change of look then changes the GTK
# theme too. SHOJIWM_DESKTOP_GTK_THEMES lists "look:light:dark" triples
# separated by spaces; "none" lists none.
#
# With "none", the theme moves to its own variant when it is installed (Arc and
# Arc-Dark, Adwaita and Adwaita-dark, Foo-Light and Foo-Dark); otherwise the
# theme is left as it is.
current=$("$1" read "$2" | tr -d "'")

dirs="$HOME/.themes ${XDG_DATA_HOME:-$HOME/.local/share}/themes"
for d in $(printf %s "${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" | tr : " "); do
  dirs="$dirs $d/themes"
done
installed() {
  for d in $dirs; do
    [ -d "$d/$1/gtk-3.0" ] && return 0
  done
  return 1
}

pairs="${SHOJIWM_DESKTOP_GTK_THEMES-kernel:Kernel:Kernel-dark aero:Aero:Aero-dark}"
[ "$pairs" = none ] && pairs=
look=$4
if [ -z "$look" ]; then
  settings="${XDG_CONFIG_HOME:-$HOME/.config}/shojiwm/settings.json"
  # The file is written pretty-printed, one key a line.
  look=$(sed -n 's/^[[:space:]]*"theme"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$settings" 2>/dev/null | head -n 1)
fi
look=${look:-kernel}
target=
for pair in $pairs; do
  name=${pair%%:*}
  rest=${pair#*:}
  light=${rest%%:*}
  dark=${rest#*:}
  if [ "$name" = "$look" ]; then
    if [ "$3" = dark ]; then target=$dark; else target=$light; fi
  fi
done
# The session's own themes: the look's one, whatever theme the key holds (the
# key is shared with the other sessions, which may have written theirs).
if [ -n "$pairs" ]; then
  if [ -n "$target" ] && [ "$target" != "$current" ] && installed "$target"; then
    exec "$1" write "$2" "'$target'"
  fi
  exit 0
fi

[ -n "$current" ] || exit 0
base=$(printf %s "$current" | sed -E "s/[-_ ]?([Dd]ark|[Ll]ight)$//")
if [ "$3" = dark ]; then
  candidates="$base-Dark $base-dark ${base}Dark ${base}-Darker"
else
  candidates="$base $base-Light $base-light"
fi
for name in $candidates; do
  [ "$name" = "$current" ] && exit 0
  if installed "$name"; then
    exec "$1" write "$2" "'$name'"
  fi
done
# Adwaita has no Adwaita-light, and its dark variant is lower case.
case "$current/$3" in
  Adwaita/dark) exec "$1" write "$2" "'Adwaita-dark'" ;;
  Adwaita-dark/light) exec "$1" write "$2" "'Adwaita'" ;;
esac
