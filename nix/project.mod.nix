# What several parts of the flake share: the sources they build from, the
# libraries of the shell, and the Odin compiler of tools/desktoptool. The
# other modules take it as their `project` argument.
{lib, ...}: let
  # The Odin compiler of tools/desktoptool, the one gui_toolkit asks for (its odin.version). Odin publishes it
  # statically linked for x86_64 Linux: nothing to patch. Its run links with clang.
  odinVersion = "dev-2026-11";
in {
  _module.args.project = {
    meta = {
      homepage = "https://github.com/sachahjkl/shojiwm-desktop";
      license = lib.licenses.mit;
      platforms = lib.platforms.linux;
    };

    # Only the files that are installed. Keeping the Nix and git files out
    # means a change to them does not rebuild the configuration.
    configSource = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../src
        ../assets
        ../shell
        ../package.json
        ../tsconfig.json
      ];
    };

    # What tools/desktoptool reads for `changelog check`: itself, the toolkit, the changelog and the three files
    # that keep the version.
    changelogSource = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../tools/desktoptool
        ../CHANGELOG.md
        ../changelog.d
        ../crates/shell/Cargo.toml
        ../crates/shell/Cargo.lock
        ../flake.nix
      ];
    };

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

    hasOdin = system: system == "x86_64-linux";
    odinFor = pkgs:
      pkgs.stdenv.mkDerivation {
        pname = "odin";
        version = odinVersion;
        src = pkgs.fetchurl {
          url = "https://github.com/odin-lang/Odin/releases/download/${odinVersion}/odin-linux-amd64-${odinVersion}.tar.gz";
          sha256 = "65c9f6e97d6f8a30930e151f270ce5aeaa98fe8b2ba49fe067394e2af04f7618";
        };
        nativeBuildInputs = [pkgs.makeWrapper];
        dontBuild = true;
        dontStrip = true;
        dontPatchELF = true;
        installPhase = ''
          mkdir -p $out/lib/odin $out/bin
          cp -r . $out/lib/odin
          makeWrapper $out/lib/odin/odin $out/bin/odin --set ODIN_ROOT $out/lib/odin/ \
            --prefix PATH : ${pkgs.lib.makeBinPath [pkgs.clang pkgs.lld]}
        '';
      };
  };
}
