{
  description = "A basic rust cli";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixos-generators = {
    url = "github:nix-community/nixos-generators";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.flakery.url = "github:getflakery/flakes";

  outputs = { self, nixpkgs, flake-utils, nixos-generators, flakery, ... }:
    (flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          app = pkgs.rustPlatform.buildRustPackage {
            pname = "app";
            version = "0.0.1";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              pkgs.pkg-config
            ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

            buildPhase = ''
              cargo build --release
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/release/app $out/bin/app
            '';

            # disable checkPhase
            doCheck = false;

          };
          appModule = (import ./service.app.nix) app;

        in
        {
          app = app;
          packages.default = app;
          packages.ami = nixos-generators.nixosGenerate {
            system = "x86_64-linux";
            format = "amazon";
            modules = [
              flakery.nixosModules.flakery
              {
                imports = [ appModule ];
                services.app.enable = true;
              }

            ];
          };

          packages.test = pkgs.testers.runNixOSTest
            {
              skipLint = true;
              name = "An awesome test.";

              nodes = {
                machine1 = { pkgs, ... }: {
                  # Empty config sets some defaults
                  imports = [ appModule ];
                  services.app.enable = true;
                  services.app.urlPrefix = "http://localhost:8080";
                };
              };

              interactive.nodes.machine1 = import ./debug-host-module.nix;

              testScript = ''
                machine1.wait_for_unit("bootstrap.service")
              '';
            };

          devShells.default = import ./shell.nix { inherit pkgs; };

        })
    );
}
