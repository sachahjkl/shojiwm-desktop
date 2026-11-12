# The configuration tree that ShojiWM runs: src, assets, shell (the shell's
# pictures), package.json and tsconfig.json.
{
  project,
  version,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: {
    packages = {
      default = config.packages.config;

      config = pkgs.stdenvNoCC.mkDerivation {
        pname = "shojiwm-desktop";
        inherit version;
        src = project.configSource;

        dontConfigure = true;
        dontBuild = true;

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -R src assets shell package.json tsconfig.json "$out/"
          runHook postInstall
        '';

        meta =
          project.meta
          // {
            description = "A complete Wayland desktop environment, built on ShojiWM";
          };
      };
    };
  };
}
