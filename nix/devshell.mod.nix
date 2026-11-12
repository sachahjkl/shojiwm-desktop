# The development shell: Rust, TypeScript, Odin, prettier, the shell's
# libraries and the Git hooks.
{
  inputs,
  lib,
  project,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: {
    devShells.default = pkgs.mkShell {
      inputsFrom = [config.devShells.shell];
      packages =
        [
          pkgs.typescript
          pkgs.prettier
          pkgs.clippy
          pkgs.rustfmt
        ]
        ++ lib.optional (project.hasOdin system) (project.odinFor pkgs)
        ++ config.checks.pre-commit.enabledPackages;

      LD_LIBRARY_PATH = lib.makeLibraryPath (project.shellRuntime pkgs);
      GUI_TOOLKIT_SRC = "${inputs.gui-toolkit}/src";

      inherit (config.checks.pre-commit) shellHook;
    };
  };
}
