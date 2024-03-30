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
                  services.app.urlPrefix = "http://localhost/";
                  services.caddy = {
                    enable = true;
                    virtualHosts."localhost".extraConfig = ''
                      handle /turso_token {
                          respond "This is response for turso_token"
                      }
                      handle /file_encryption_key {
                          respond "This is response for file_encryption_key"
                      }

                      handle /template_id {
                          respond "This is response for path3"
                      }
                      handle /flake_url {
                          respond "This is response for path3"
                      }
                    '';
                  };
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
