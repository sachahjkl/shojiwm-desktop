{
  description = "A Windows 7 Aero desktop for the ShojiWM Wayland compositor";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2605";

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
        ./assets
        ./shell
        ./package.json
        ./tsconfig.json
      ];
    };
  in {
    packages = forAllSystems (system: let
      pkgs = pkgsFor system;

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
    in {
      default = config;
      inherit config install;
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
    in {
      pre-commit = preCommitCheck;

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
          ]
          ++ self.checks.${system}.pre-commit.enabledPackages;

        inherit (self.checks.${system}.pre-commit) shellHook;
      };
    });
  };
}
