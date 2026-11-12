# The TSX is type checked against the ShojiWM TypeScript package, the way the
# compositor's own `npm run typecheck` does it. The package is the one of the
# `shojiwm` input: the compositor the flake ships.
{
  inputs,
  project,
  ...
}: {
  perSystem = {pkgs, ...}: {
    checks.typecheck =
      pkgs.runCommand "shojiwm-desktop-typecheck" {
        nativeBuildInputs = [pkgs.typescript];
      } ''
        cp -R ${project.configSource}/src ./src
        cp ${project.configSource}/tsconfig.json ./tsconfig.json
        mkdir -p node_modules
        ln -s ${inputs.shojiwm}/packages/shoji_wm node_modules/shoji_wm
        tsc -p tsconfig.json
        touch "$out"
      '';
  };
}
