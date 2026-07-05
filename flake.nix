{
  description = "Silicate -- a simple password manager, built for speed.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };
  outputs =
    {
      self,
      nixpkgs,
      devenv,
      naersk,
      ...
    }@inputs:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f system (nixpkgs.legacyPackages.${system}));
    in
    {
      devShells = eachSystem (
        system: pkgs: {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              ./devenv.nix
            ];
          };
        }
      );

      packages = eachSystem (
        system: pkgs:
        let
          naerskLib = pkgs.callPackage naersk { };
        in
        {
          default = naerskLib.buildPackage {
            name = "silicate";
            version = "1.1.0";
            src = ./.;
          };
        }
      );
    };
}
