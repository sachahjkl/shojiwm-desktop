#!/bin/sh
# Move the GTK 3 theme to its light or dark variant, for the Control Panel's
# light or dark choice. GTK 3 programs read the theme, not the colour scheme.
#
#   gtk-switch.sh DCONF KEY dark|light
#
# The variant is used only when it is installed (Arc and Arc-Dark, Adwaita
# and Adwaita-dark, Foo-Light and Foo-Dark); otherwise the theme is left as
# it is.
current=$("$1" read "$2" | tr -d "'")
[ -n "$current" ] || exit 0
base=$(printf %s "$current" | sed -E "s/[-_ ]?([Dd]ark|[Ll]ight)$//")
if [ "$3" = dark ]; then
  candidates="$base-Dark $base-dark ${base}Dark ${base}-Darker"
else
  candidates="$base $base-Light $base-light"
fi
dirs="$HOME/.themes ${XDG_DATA_HOME:-$HOME/.local/share}/themes"
for d in $(printf %s "${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" | tr : " "); do
  dirs="$dirs $d/themes"
done
for name in $candidates; do
  [ "$name" = "$current" ] && exit 0
  for d in $dirs; do
    if [ -d "$d/$name/gtk-3.0" ]; then
      exec "$1" write "$2" "'$name'"
    fi
  done
done
# Adwaita has no Adwaita-light, and its dark variant is lower case.
case "$current/$3" in
  Adwaita/dark) exec "$1" write "$2" "'Adwaita-dark'" ;;
  Adwaita-dark/light) exec "$1" write "$2" "'Adwaita'" ;;
esac
