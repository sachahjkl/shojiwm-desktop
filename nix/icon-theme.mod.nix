# The "Aero" icon theme, for the programs of the session (Thunar): the
# desktop's icons. It inherits Oxygen and replaces what differs: the yellow
# folder, the libraries (the folder with an emblem, as on the desktop) and the
# glass Recycle Bin.
{lib, ...}: {
  perSystem = {pkgs, ...}: {
    packages.icon-theme =
      pkgs.runCommand "shojiwm-desktop-aero-icons" {
        src = lib.fileset.toSource {
          root = ../.;
          fileset = lib.fileset.unions [../assets/icons-src ../shell/assets/icons];
        };
        oxygen = "${pkgs.kdePackages.oxygen-icons}/share/icons/oxygen/base";
        meta = {
          description = "The Aero icon theme of shojiwm-desktop: Oxygen with Windows 7's folders and Recycle Bin";
          license = lib.licenses.lgpl3Plus;
        };
      } ''
        theme="$out/share/icons/Aero"
        sizes="16 22 32 48 64 128 256"
        directories=""
        for size in $sizes; do
          dir="$theme/''${size}x''${size}/places"
          mkdir -p "$dir"
          for name in folder inode-directory; do
            cp "$oxygen/''${size}x''${size}/places/folder-yellow.png" "$dir/$name.png"
          done
          directories="$directories''${size}x''${size}/places,"
        done
        mkdir -p "$theme/aero/places" "$theme/scalable/places"
        icons="$src/shell/assets/icons"
        cp "$icons/folder-open.png" "$theme/aero/places/folder-open.png"
        cp "$icons/documents.png" "$theme/aero/places/folder-documents.png"
        cp "$icons/downloads.png" "$theme/aero/places/folder-download.png"
        cp "$icons/pictures.png" "$theme/aero/places/folder-pictures.png"
        cp "$icons/music.png" "$theme/aero/places/folder-music.png"
        cp "$icons/videos.png" "$theme/aero/places/folder-videos.png"
        cp "$icons/home.png" "$theme/aero/places/user-home.png"
        # The file manager is the folder, as Windows 7's Explorer.
        mkdir -p "$theme/aero/apps"
        for name in org.xfce.thunar thunar system-file-manager; do
          cp "$icons/folder.png" "$theme/aero/apps/$name.png"
        done
        cp "$src/assets/icons-src/recycle-bin.svg" "$theme/scalable/places/user-trash.svg"
        cp "$src/assets/icons-src/recycle-bin-full.svg" "$theme/scalable/places/user-trash-full.svg"
        cp "$icons/LICENSE" "$theme/LICENSE"
        {
          echo "[Icon Theme]"
          echo "Name=Aero"
          echo "Comment=shojiwm-desktop: Oxygen with Windows 7's folders and Recycle Bin"
          echo "Inherits=oxygen,hicolor"
          echo "Directories=''${directories}aero/places,aero/apps,scalable/places"
          for size in $sizes; do
            printf '\n[%sx%s/places]\nSize=%s\nContext=Places\nType=Fixed\n' "$size" "$size" "$size"
          done
          printf '\n[aero/places]\nSize=128\nMinSize=16\nMaxSize=512\nContext=Places\nType=Scalable\n'
          printf '\n[aero/apps]\nSize=128\nMinSize=16\nMaxSize=512\nContext=Applications\nType=Scalable\n'
          printf '\n[scalable/places]\nSize=128\nMinSize=16\nMaxSize=512\nContext=Places\nType=Scalable\n'
        } > "$theme/index.theme"
      '';
  };
}
