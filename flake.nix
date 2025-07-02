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
      perSystem = {self', pkgs, system, ...}: {
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
          release = pkgs.stdenv.mkDerivation {
            name = "jsonkdl";
            version = "1.0.0";
            src = ./.;

            buildInputs = with pkgs; [
              rust-bin.stable.latest.default
            ];
            
            buildPhase = ''
              cargo build --release
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp $src/target/release/jsonkdl $out/bin
            '';
          };

          default = self'.packages.release;
        };
      };
    };
}
