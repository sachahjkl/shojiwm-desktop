# The Git hooks (prek), as a check, and the formatter.
{inputs, ...}: {
  perSystem = {
    pkgs,
    system,
    ...
  }: {
    checks.pre-commit = inputs.git-hooks.lib.${system}.run {
      package = pkgs.prek;
      src = ../.;
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

    formatter = pkgs.alejandra;
  };
}
