{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;
      perSystem = {self', pkgs, lib, system, ...}: {
        _module.args.pkgs =
          import inputs.nixpkgs {
            inherit system;
            overlays = [inputs.rust-overlay.overlays.default];
          };

        apps = {
          release = {
            type = "app";
            program = "${self'.packages.release}/bin/jsonkdl";
          };

          default = self'.apps.release;
        };

        packages = {
          release = pkgs.rustPlatform.buildRustPackage {
            pname = "jsonkdl";
            version = "1.0.0";

            src = ./.;

            cargoHash = "sha256-n1h3/yl9ixTweTah/02ZL+bdU/+j1fPCMdNstzkvIw0=";
          };

          default = self'.packages.release;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [rust-bin.stable.latest.default];
        };
      };
    };
}
