# The GlobalShortcuts portal: programs such as Hex bind their keys through it
# (see portal/ and src/shortcuts.ts).
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
    packages.portal = pkgs.rustPlatform.buildRustPackage {
      pname = "xdg-desktop-portal-shojiwm-desktop";
      inherit version;
      src = lib.fileset.toSource {
        root = ../portal;
        fileset = lib.fileset.unions [
          ../portal/Cargo.toml
          ../portal/Cargo.lock
          ../portal/src
          ../portal/data
        ];
      };
      cargoLock.lockFile = ../portal/Cargo.lock;

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

      meta =
        project.meta
        // {
          description = "The GlobalShortcuts portal for the Windows 7 configuration of ShojiWM";
        };
    };

    checks = {
      # The portal builds and passes its tests.
      inherit (config.packages) portal;

      # The portal's Rust: clippy without warnings, and rustfmt's layout.
      portal-lint = config.packages.portal.overrideAttrs (old: {
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
    };
  };
}
