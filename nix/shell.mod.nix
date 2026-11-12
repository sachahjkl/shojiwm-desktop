# The shell on AntiWebUI (crates/shell): taskbar, Start menu, panels,
# Control Panel, lock screen and log-on screen. Its pictures are in
# shell/assets.
{
  lib,
  project,
  version,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    # wgpu and AntiWebUI load Vulkan, EGL, Wayland and PipeWire at run time:
    # they are on its library path. The programs it runs (dconf, gio, setsid,
    # xdg-open) come after the system's.
    packages.shell = pkgs.rustPlatform.buildRustPackage {
      pname = "shojiwm-shell";
      inherit version;
      src = lib.fileset.toSource {
        root = ../crates/shell;
        fileset = lib.fileset.unions [
          ../crates/shell/Cargo.toml
          ../crates/shell/Cargo.lock
          ../crates/shell/src
        ];
      };
      cargoLock = {
        lockFile = ../crates/shell/Cargo.lock;
        allowBuiltinFetchGit = true;
      };
      nativeBuildInputs = [pkgs.pkg-config pkgs.makeWrapper];
      buildInputs = project.shellLibraries pkgs;
      postFixup = ''
        wrapProgram "$out/bin/shojiwm-shell" \
          --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath (project.shellRuntime pkgs)} \
          --suffix PATH : ${lib.makeBinPath [pkgs.dconf pkgs.glib pkgs.util-linux pkgs.xdg-utils]}
      '';
      meta =
        project.meta
        // {
          description = "The desktop shell of shojiwm-desktop, on AntiWebUI";
          mainProgram = "shojiwm-shell";
        };
    };

    checks = {
      # The shell on AntiWebUI builds.
      inherit (config.packages) shell;

      # The shell's Rust: clippy without warnings, and rustfmt's layout.
      shell-lint = config.packages.shell.overrideAttrs (old: {
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
    };

    # Only what `cargo build` of the shell needs, with no Git hooks:
    # tools/screenshots.sh builds the shell in it.
    devShells.shell = pkgs.mkShell {
      packages =
        [
          pkgs.cargo
          pkgs.rustc
          pkgs.pkg-config
        ]
        ++ project.shellLibraries pkgs;

      # What the shell loads at run time (shellRuntime), for `cargo run`.
      LD_LIBRARY_PATH = lib.makeLibraryPath (project.shellRuntime pkgs);
    };
  };
}
