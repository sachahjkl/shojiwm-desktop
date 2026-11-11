{
  description = "A Windows 7 Aero desktop for the ShojiWM Wayland compositor";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2605";

    git-hooks = {
      url = "https://flakehub.com/f/cachix/git-hooks.nix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The ShojiWM sources, for the TypeScript package the config is checked
    # against: the author's fork, whose integration branch carries what this
    # config uses before upstream has it (labels' textShadow; see its
    # FORK.md). The compositor itself is not built here.
    shojiwm-src = {
      url = "github:sachahjkl/ShojiWM/integration";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    git-hooks,
    shojiwm-src,
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
    pkgsFor = system: nixpkgs.legacyPackages.${system};

    version = "0.1.0";

    # The libraries the shell on AntiWebUI links (fonts, keyboard, Wayland,
    # PAM, and xcb for its X11 backend) and the ones it loads at run time.
    shellLibraries = pkgs: [
      pkgs.fontconfig
      pkgs.freetype
      pkgs.libxkbcommon
      pkgs.wayland
      pkgs.pam
      pkgs.libxcb
    ];
    shellRuntime = pkgs: [
      pkgs.vulkan-loader
      pkgs.libGL
      pkgs.wayland
      pkgs.libxkbcommon
      pkgs.pipewire
    ];

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

      config = pkgs.stdenvNoCC.mkDerivation {
        pname = "shojiwm-desktop";
        inherit version;
        src = configSource;

        dontConfigure = true;
        dontBuild = true;

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -R src assets shell package.json tsconfig.json "$out/"
          runHook postInstall
        '';

        meta = {
          description = "A Windows 7 Aero desktop for ShojiWM";
          homepage = "https://github.com/sachahjkl/shojiwm-desktop";
          license = nixpkgs.lib.licenses.mit;
          platforms = nixpkgs.lib.platforms.linux;
        };
      };

      install = pkgs.writeShellApplication {
        name = "shojiwm-desktop-install";
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
        pname = "xdg-desktop-portal-shojiwm-desktop";
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
          mv "$out/bin/xdg-desktop-portal-shojiwm-desktop" "$out/libexec/"
          rmdir "$out/bin"
          install -Dm644 data/shojiwm-desktop.portal \
            "$out/share/xdg-desktop-portal/portals/shojiwm-desktop.portal"
          substitute data/dbus.service \
            "$out/share/dbus-1/services/org.freedesktop.impl.portal.desktop.shojiwm-desktop.service" \
            --subst-var out
          substitute data/systemd.service \
            "$out/share/systemd/user/xdg-desktop-portal-shojiwm-desktop.service" \
            --subst-var out
        '';

        meta = {
          description = "The GlobalShortcuts portal for the Windows 7 configuration of ShojiWM";
          homepage = "https://github.com/sachahjkl/shojiwm-desktop";
          license = nixpkgs.lib.licenses.mit;
          platforms = nixpkgs.lib.platforms.linux;
        };
      };

      # The shell on AntiWebUI (crates/shell): taskbar, Start menu, panels,
      # Control Panel, lock screen and log-on screen. Its pictures are in
      # shell/assets. wgpu and AntiWebUI load Vulkan, EGL, Wayland and
      # PipeWire at run time: they are on its library path. The programs it
      # runs (dconf, gio, setsid, xdg-open) come after the system's.
      shell = pkgs.rustPlatform.buildRustPackage {
        pname = "shojiwm-shell";
        inherit version;
        src = nixpkgs.lib.fileset.toSource {
          root = ./crates/shell;
          fileset = nixpkgs.lib.fileset.unions [
            ./crates/shell/Cargo.toml
            ./crates/shell/Cargo.lock
            ./crates/shell/src
          ];
        };
        cargoLock = {
          lockFile = ./crates/shell/Cargo.lock;
          allowBuiltinFetchGit = true;
        };
        nativeBuildInputs = [pkgs.pkg-config pkgs.makeWrapper];
        buildInputs = shellLibraries pkgs;
        postFixup = ''
          wrapProgram "$out/bin/shojiwm-shell" \
            --prefix LD_LIBRARY_PATH : ${nixpkgs.lib.makeLibraryPath (shellRuntime pkgs)} \
            --suffix PATH : ${nixpkgs.lib.makeBinPath [pkgs.dconf pkgs.glib pkgs.util-linux pkgs.xdg-utils]}
        '';
        meta = {
          description = "The desktop shell of shojiwm-desktop, on AntiWebUI";
          homepage = "https://github.com/sachahjkl/shojiwm-desktop";
          license = nixpkgs.lib.licenses.mit;
          platforms = nixpkgs.lib.platforms.linux;
          mainProgram = "shojiwm-shell";
        };
      };

      # The "Aero" icon theme, for the programs of the session (Thunar): the
      # desktop's icons. It inherits Oxygen and replaces what differs: the
      # yellow folder, the libraries (the folder with an emblem, as on the
      # desktop) and the glass Recycle Bin.
      icon-theme =
        pkgs.runCommand "shojiwm-desktop-aero-icons" {
          src = nixpkgs.lib.fileset.toSource {
            root = ./.;
            fileset = nixpkgs.lib.fileset.unions [./assets/icons-src ./shell/assets/icons];
          };
          oxygen = "${pkgs.kdePackages.oxygen-icons}/share/icons/oxygen/base";
          meta = {
            description = "The Aero icon theme of shojiwm-desktop: Oxygen with Windows 7's folders and Recycle Bin";
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
    in {
      default = config;
      inherit config icon-theme install portal shell;
    });

    apps = forAllSystems (system: {
      install = {
        type = "app";
        program = "${self.packages.${system}.install}/bin/shojiwm-desktop-install";
      };
    });

    # The Windows 7 log-on screen as the display manager (greetd): see
    # nix/greeter.nix and services.shojiwm-desktop-greeter.
    nixosModules.greeter = {
      config,
      lib,
      pkgs,
      ...
    }:
      import ./nix/greeter.nix {
        package = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        inherit (self.packages.${pkgs.stdenv.hostPlatform.system}) shell;
      } {inherit config lib pkgs;};

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

      # The shell on AntiWebUI builds.
      inherit (self.packages.${system}) shell;

      # The shell's Rust: clippy without warnings, and rustfmt's layout.
      shell-lint = self.packages.${system}.shell.overrideAttrs (old: {
        pname = "shojiwm-shell-lint";
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
        postFixup = "";
      });

      # The portal's Rust: clippy without warnings, and rustfmt's layout.
      portal-lint = portal.overrideAttrs (old: {
        pname = "xdg-desktop-portal-shojiwm-desktop-lint";
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
        pkgs.runCommand "shojiwm-desktop-typecheck" {
          nativeBuildInputs = [pkgs.typescript];
        } ''
          cp -R ${configSource}/src ./src
          cp ${configSource}/tsconfig.json ./tsconfig.json
          mkdir -p node_modules
          ln -s ${shojiwm-src}/packages/shoji_wm node_modules/shoji_wm
          tsc -p tsconfig.json
          touch "$out"
        '';

      # The log-on screen in a VM: greetd, Hyprland, shojiwm-shell greeter, a password,
      # and the session it starts.
      greeter = import ./tests/greeter.nix {
        inherit pkgs;
        module = self.nixosModules.greeter;
      };
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
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.pkg-config
          ]
          ++ shellLibraries pkgs
          ++ self.checks.${system}.pre-commit.enabledPackages;

        # What the shell loads at run time (shellRuntime), for `cargo run`.
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (shellRuntime pkgs);

        inherit (self.checks.${system}.pre-commit) shellHook;
      };
    });
  };
}
