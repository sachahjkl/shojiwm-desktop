# The tests of tools/desktoptool, then every fragment of changelog.d well
# formed, and the version of Cargo.toml, Cargo.lock and flake.nix the same,
# the last released one. Odin is published for x86_64 Linux only.
{
  inputs,
  lib,
  project,
  ...
}: {
  perSystem = {
    pkgs,
    system,
    ...
  }: {
    checks = lib.optionalAttrs (project.hasOdin system) {
      changelog =
        pkgs.runCommand "shojiwm-desktop-changelog" {
          nativeBuildInputs = [(project.odinFor pkgs)];
        } ''
          cp -R ${project.changelogSource}/. .
          chmod -R u+w .
          export HOME="$TMPDIR"
          mkdir -p build
          odin test tools/desktoptool -collection:gt=${inputs.gui-toolkit}/src -out:build/desktoptool_test
          odin run tools/desktoptool -collection:gt=${inputs.gui-toolkit}/src -out:build/desktoptool -- changelog check
          touch "$out"
        '';
    };
  };
}
