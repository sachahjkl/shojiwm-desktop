{
  description = "A Windows 7 Aero desktop for the ShojiWM Wayland compositor";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2605";

    # The rolling unstable branch, for the programs that move fast and are
    # best taken at their latest: Vicinae. Make it follow your own unstable
    # nixpkgs with `inputs.nixpkgs-unstable.follows`.
    nixpkgs-unstable.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

    git-hooks = {
      url = "https://flakehub.com/f/cachix/git-hooks.nix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The ShojiWM sources, for the TypeScript package the config is checked
    # against. The compositor itself is not built here.
    shojiwm-src = {
      url = "github:bea4dev/ShojiWM";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-unstable,
    git-hooks,
    shojiwm-src,
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
    pkgsFor = system: nixpkgs.legacyPackages.${system};

    version = "0.1.0";

    # Only the files that are installed. Keeping the Nix and git files out
    # means a change to them does not rebuild the configuration.
    configSource = nixpkgs.lib.fileset.toSource {
      root = ./.;
      fileset = nixpkgs.lib.fileset.unions [
        ./src
        # The screenshots of the README live under assets; they are not part
        # of the configuration.
        (nixpkgs.lib.fileset.difference ./assets ./assets/site)
        ./shell
        ./package.json
        ./tsconfig.json
      ];
    };
  in {
    packages = forAllSystems (system: let
      pkgs = pkgsFor system;

      # Taken from unstable: see the input. Its lists scroll by the wheel's
      # step at once, as Windows 7's do: Vicinae has no setting for the
      # smooth scrolling and inertia of its wheel handler, so both are off
      # in the source.
      vicinae = nixpkgs-unstable.legacyPackages.${system}.vicinae.overrideAttrs (old: {
        postPatch =
          (old.postPatch or "")
          + ''
            substituteInPlace vendor/kirigami-wheelhandler/wheelhandler.h \
              --replace-fail "bool m_smoothScroll = true;" "bool m_smoothScroll = false;" \
              --replace-fail "bool m_inertiaScroll = true;" "bool m_inertiaScroll = false;"
          '';
      });

      config = pkgs.stdenvNoCC.mkDerivation {
        pname = "shojiwm-win7";
        inherit version;
        src = configSource;

        dontConfigure = true;
        dontBuild = true;

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -R src assets shell package.json tsconfig.json "$out/"
          # Vicinae is the launcher Super+D opens: point the config at this
          # one, so installing the configuration installs it too.
          substituteInPlace "$out/src/index.tsx" \
            --replace-fail 'const VICINAE = "vicinae";' \
              'const VICINAE = "${vicinae}/bin/vicinae";'
          # dconf for the light or dark preference, gdbus for logind's Lock,
          # gio for the desktop's files.
          substituteInPlace "$out/shell/tools.js" \
            --replace-fail 'var dconf = "dconf";' 'var dconf = "${nixpkgs.lib.getExe' pkgs.dconf "dconf"}";' \
            --replace-fail 'var gdbus = "gdbus";' 'var gdbus = "${nixpkgs.lib.getExe' pkgs.glib "gdbus"}";' \
            --replace-fail 'var gio = "gio";' 'var gio = "${nixpkgs.lib.getExe' pkgs.glib "gio"}";'
          runHook postInstall
        '';

        meta = {
          description = "A Windows 7 Aero desktop for ShojiWM";
          homepage = "https://github.com/sachahjkl/shojiwm-win7";
          license = nixpkgs.lib.licenses.mit;
          platforms = nixpkgs.lib.platforms.linux;
        };
      };

      install = pkgs.writeShellApplication {
        name = "shojiwm-win7-install";
        runtimeInputs = [pkgs.coreutils];
        text = ''
          config_dir="''${XDG_CONFIG_HOME:-$HOME/.config}/shojiwm"
          mkdir -p "$config_dir"

          # node_modules/shoji_wm is created by ShojiWM itself; leave it.
          rm -rf "$config_dir/src" "$config_dir/assets" "$config_dir/shell"
          cp -R ${config}/src ${config}/assets ${config}/shell "$config_dir/"
          cp ${config}/package.json ${config}/tsconfig.json "$config_dir/"
          chmod -R u+w "$config_dir/src" "$config_dir/assets" "$config_dir/shell"
          chmod u+w "$config_dir/package.json" "$config_dir/tsconfig.json"

          echo "installed the Windows 7 desktop in $config_dir"
          echo "log out and start the ShojiWM session, or press Super+Shift+R"
        '';
      };

      # The GlobalShortcuts portal: programs such as Hex bind their keys
      # through it (see portal/ and src/shortcuts.ts).
      portal = pkgs.rustPlatform.buildRustPackage {
        pname = "xdg-desktop-portal-shojiwm-win7";
        inherit version;
        src = nixpkgs.lib.fileset.toSource {
          root = ./portal;
          fileset = nixpkgs.lib.fileset.unions [
            ./portal/Cargo.toml
            ./portal/Cargo.lock
            ./portal/src
            ./portal/data
          ];
        };
        cargoLock.lockFile = ./portal/Cargo.lock;

        postInstall = ''
          mkdir -p "$out/libexec" "$out/share/dbus-1/services" "$out/share/systemd/user"
          mv "$out/bin/xdg-desktop-portal-shojiwm-win7" "$out/libexec/"
          rmdir "$out/bin"
          install -Dm644 data/shojiwm-win7.portal \
            "$out/share/xdg-desktop-portal/portals/shojiwm-win7.portal"
          substitute data/dbus.service \
            "$out/share/dbus-1/services/org.freedesktop.impl.portal.desktop.shojiwm-win7.service" \
            --subst-var out
          substitute data/systemd.service \
            "$out/share/systemd/user/xdg-desktop-portal-shojiwm-win7.service" \
            --subst-var out
        '';

        meta = {
          description = "The GlobalShortcuts portal for the Windows 7 configuration of ShojiWM";
          homepage = "https://github.com/sachahjkl/shojiwm-win7";
          license = nixpkgs.lib.licenses.mit;
          platforms = nixpkgs.lib.platforms.linux;
        };
      };

      # The "Aero" icon theme, for the programs of the session (Thunar): the
      # desktop's icons. It inherits Oxygen and replaces what differs: the
      # yellow folder, the libraries (the folder with an emblem, as on the
      # desktop) and the glass Recycle Bin.
      icon-theme =
        pkgs.runCommand "shojiwm-win7-aero-icons" {
          src = nixpkgs.lib.fileset.toSource {
            root = ./.;
            fileset = nixpkgs.lib.fileset.unions [./assets/icons-src ./shell/assets/icons];
          };
          oxygen = "${pkgs.kdePackages.oxygen-icons}/share/icons/oxygen/base";
          meta = {
            description = "The Aero icon theme of shojiwm-win7: Oxygen with Windows 7's folders and Recycle Bin";
            license = nixpkgs.lib.licenses.lgpl3Plus;
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
            echo "Comment=shojiwm-win7: Oxygen with Windows 7's folders and Recycle Bin"
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
    in {
      default = config;
      inherit config icon-theme install portal vicinae;
    });

    apps = forAllSystems (system: {
      install = {
        type = "app";
        program = "${self.packages.${system}.install}/bin/shojiwm-win7-install";
      };
    });

    checks = forAllSystems (system: let
      pkgs = pkgsFor system;

      preCommitCheck = git-hooks.lib.${system}.run {
        package = pkgs.prek;
        src = ./.;
        hooks = {
          alejandra.enable = true;
          deadnix.enable = true;
          statix.enable = true;
          prettier = {
            enable = true;
            files = "\\.(ts|tsx|json|md)$";
          };
        };
      };
      inherit (self.packages.${system}) portal;
    in {
      pre-commit = preCommitCheck;

      # The portal builds and passes its tests.
      inherit portal;

      # The portal's Rust: clippy without warnings, and rustfmt's layout.
      portal-lint = portal.overrideAttrs (old: {
        pname = "xdg-desktop-portal-shojiwm-win7-lint";
        nativeBuildInputs = old.nativeBuildInputs ++ [pkgs.clippy pkgs.rustfmt];
        buildPhase = ''
          runHook preBuild
          cargo fmt --check
          cargo clippy --all-targets --offline -- -D warnings
          runHook postBuild
        '';
        doCheck = false;
        installPhase = "touch $out";
        postInstall = "";
      });

      # The TSX is type checked against the ShojiWM TypeScript package, the
      # way the compositor's own `npm run typecheck` does it.
      typecheck =
        pkgs.runCommand "shojiwm-win7-typecheck" {
          nativeBuildInputs = [pkgs.typescript];
        } ''
          cp -R ${configSource}/src ./src
          cp ${configSource}/tsconfig.json ./tsconfig.json
          mkdir -p node_modules
          ln -s ${shojiwm-src}/packages/shoji_wm node_modules/shoji_wm
          tsc -p tsconfig.json
          touch "$out"
        '';

      # The QML of the shell, checked against the Quickshell types. The
      # qmldir is written here only: Quickshell generates its own at run time.
      qmllint =
        pkgs.runCommand "shojiwm-win7-qmllint" {
          nativeBuildInputs = [pkgs.qt6.qtdeclarative];
        } ''
          cp -R ${configSource}/shell ./shell
          chmod -R u+w ./shell
          cat > ./shell/qmldir <<EOF
          module shell
          singleton Apps 1.0 Apps.qml
          singleton StartMenuState 1.0 StartMenuState.qml
          singleton StartMenuIpc 1.0 StartMenuIpc.qml
          singleton CompositorIpc 1.0 CompositorIpc.qml
          singleton Settings 1.0 Settings.qml
          singleton Scheme 1.0 Scheme.qml
          singleton LockState 1.0 LockState.qml
          singleton ControlPanelState 1.0 ControlPanelState.qml
          singleton ShutdownState 1.0 ShutdownState.qml
          singleton DragState 1.0 DragState.qml
          singleton Audio 1.0 Audio.qml
          singleton Startup 1.0 Startup.qml
          singleton DisplaySettings 1.0 DisplaySettings.qml
          singleton LauncherEntries 1.0 LauncherEntries.qml
          singleton Network 1.0 Network.qml
          singleton NotificationHistory 1.0 NotificationHistory.qml
          singleton WorldClocks 1.0 WorldClocks.qml
          singleton SystemStats 1.0 SystemStats.qml
          singleton Slideshow 1.0 Slideshow.qml
          singleton StickyNotes 1.0 StickyNotes.qml
          singleton DefaultApps 1.0 DefaultApps.qml
          singleton Maintenance 1.0 Maintenance.qml
          singleton Screens 1.0 Screens.qml
          EOF
          cd ./shell
          qmllint \
            -I ${pkgs.qt6.qtdeclarative}/lib/qt-6/qml \
            -I ${pkgs.quickshell}/lib/qt-6/qml \
            -I . -I components \
            -W 0 \
            --uncreatable-type info \
            --unresolved-type info \
            --unqualified info \
            --missing-type info \
            ./*.qml components/*.qml
          touch "$out"
        '';
    });

    formatter = forAllSystems (system: (pkgsFor system).alejandra);

    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        packages =
          [
            pkgs.typescript
            pkgs.prettier
            pkgs.quickshell
            pkgs.qt6.qtdeclarative
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
          ]
          ++ self.checks.${system}.pre-commit.enabledPackages;

        inherit (self.checks.${system}.pre-commit) shellHook;
      };
    });
  };
}
