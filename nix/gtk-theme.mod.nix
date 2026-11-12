# The GTK themes of the desktop's two looks, for GTK 3 and GTK 4: Kernel and Kernel-dark, Aero
# and Aero-dark. Each is GTK's own Adwaita (GTK 3) or Default (GTK 4) theme, compiled from the
# SCSS sources of the GTK in nixpkgs with the look's palette (gtk-theme/_kernel.scss,
# gtk-theme/_aero.scss) and the widgets that carry the look (gtk-theme/_widgets.scss): the
# shell's scroll bar, plates for buttons, the caption plate for header bars, the look's tabs.
# Every other widget stays Adwaita's, from the same GTK version as the programs.
{lib, ...}: {
  perSystem = {pkgs, ...}: {
    packages.gtk-theme =
      pkgs.runCommand "shojiwm-desktop-gtk-theme" {
        src = lib.fileset.toSource {
          root = ../gtk-theme;
          fileset = ../gtk-theme;
        };
        gtk3Source = pkgs.gtk3.src;
        gtk4Source = pkgs.gtk4.src;
        nativeBuildInputs = [pkgs.sassc];
        meta = {
          description = "The GTK 3 and GTK 4 themes of shojiwm-desktop's Kernel and Aero looks, light and dark";
          # Adwaita's sources and assets, which the themes are built from.
          license = lib.licenses.lgpl21Plus;
          platforms = lib.platforms.linux;
        };
      } ''
        # prepare TARBALL THEME-DIRECTORY DESTINATION: the theme's sources from GTK's tarball,
        # made ready for a palette: the top-level variables of _colors.scss and _common.scss
        # become !default, so a palette imported first sets them and what Adwaita derives from
        # them follows; the button mixin becomes adwaita-button, which _button-gtk*.scss wraps.
        # Each step checks that it found what it changes, so a new GTK that moves them fails
        # here, not in a theme that silently stays Adwaita's.
        prepare() {
          mkdir unpacked
          tar -xf "$1" -C unpacked --wildcards "*/$2/*"
          cp -r unpacked/*/"$2" "$3"
          rm -rf unpacked
          chmod -R u+w "$3"
          sed -E -i 's/^(\$[A-Za-z0-9_-]+:.*[^[:space:]]);[[:space:]]*$/\1 !default;/' "$3/_colors.scss" "$3/_common.scss"
          grep -q '^\$selected_bg_color:.* !default;$' "$3/_colors.scss"
          grep -q '^\$button_radius:.* !default;$' "$3/_common.scss"
          [ "$(grep -c '^@mixin button(' "$3/_drawing.scss")" = 1 ]
          sed -i 's/^@mixin button(/@mixin adwaita-button(/' "$3/_drawing.scss"
          cp "$src"/*.scss "$3/"
        }

        # compile SOURCES GTK LOOK VARIANT OUTPUT
        compile() {
          {
            printf "\$variant: '%s';\n\$contrast: 'normal';\n\$gtk: %s;\n" "$4" "$2"
            printf "@import '%s';\n" "$3" colors drawing "button-gtk$2" common colors-public widgets
          } > "$1/entry.scss"
          sassc -M -t compact "$1/entry.scss" "$5"
        }

        prepare "$gtk3Source" gtk/theme/Adwaita gtk3
        prepare "$gtk4Source" gtk/theme/Default gtk4

        for look in kernel aero; do
          case $look in
            kernel) name=Kernel ;;
            aero) name=Aero ;;
          esac
          for theme in "$name" "$name-dark"; do
            # The light theme's gtk-dark.css is its dark variant, for a program that asks for
            # dark; the dark theme is dark in both.
            case $theme in
              *-dark) variant=dark ;;
              *) variant=light ;;
            esac
            for gtk in 3 4; do
              dir="$out/share/themes/$theme/gtk-$gtk.0"
              mkdir -p "$dir"
              compile "gtk$gtk" "$gtk" "$look" "$variant" "$dir/gtk.css"
              compile "gtk$gtk" "$gtk" "$look" dark "$dir/gtk-dark.css"
              # The CSS refers to its check marks, radio dots and scale marks as assets/...,
              # beside it.
              cp -r "gtk$gtk/assets" "$dir/assets"
            done
            printf '[Desktop Entry]\nType=X-GNOME-Metatheme\nName=%s\nComment=shojiwm-desktop: the %s look\nEncoding=UTF-8\n\n[X-GNOME-Metatheme]\nGtkTheme=%s\n' \
              "$theme" "$name" "$theme" > "$out/share/themes/$theme/index.theme"
          done
        done
      '';
  };
}
