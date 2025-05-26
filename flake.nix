{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs [ "aarch64-linux" "x86_64-linux" ] (
          system: function nixpkgs.legacyPackages.${system}
        );
    in
    rec {
      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);

      packages = forAllSystems (pkgs: rec {
        bright = import ./nix/package.nix { inherit pkgs; };
        default = bright;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            bacon
            rust-analyzer
          ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });

      overlays.default = final: prev: {
        bright = import ./nix/package.nix { pkgs = final; };
      };

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        {
          options.programs.bright.enable = lib.mkEnableOption "Install bright and manage the udev rule";
          config = lib.mkIf config.programs.bright.enable {
            environment.systemPackages = [ pkgs.bright ];
            services.udev.packages = [ pkgs.bright ];
            nixpkgs.overlays = [ overlays.default ];
          };
        };
    };
}
