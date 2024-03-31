{
  description = "A basic rust cli";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixos-generators = {
    url = "github:nix-community/nixos-generators";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.flakery.url = "github:getflakery/flakes";



  outputs =
    { self
    , nixpkgs
    , flake-utils
    , nixos-generators
    , flakery
    , ...
    }:
    (flake-utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Common arguments can be set here to avoid repeating them later


        app = pkgs.rustPlatform.buildRustPackage {
          pname = "app";
          version = "0.0.1";
          # src = ./.;
          # filter nix files 
          src = pkgs.lib.sources.cleanSourceWith {
            src = ./.;
            filter = name: type:
            let 
              baseName = baseNameOf (toString name);
            in (
              (pkgs.lib.sources.cleanSourceFilter name type) || 
              baseName == ".nix"
            );
          };

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
                environment.systemPackages = [ pkgs.sqlite ];
                services.app.enable = true;
                services.app.urlPrefix = "http://localhost:8080/";
                services.app.sqlUrl = "file:///tmp/db.sqlite3";
                services.app.useLocal = "true";
                services.app.applyFlake = "false";
                services.app.after = [ "network.target" "serve.service" "seeddb.service"];

                systemd.services.seeddb = {
                  wantedBy = [ "multi-user.target" ];
                  path = [ pkgs.sqlite ];
                  script = "${./seeddb.sh}";
                  serviceConfig = {
                    Type = "oneshot";
                  };
                };

                systemd.services.serve = {
                  wantedBy = [ "multi-user.target" ];
                  path = [ pkgs.python3 ];
                  script = "${./serve.py}";
                  serviceConfig = {
                    Restart = "always";
                    RestartSec = 0;
                  };
                };
              };
            };

            interactive.nodes.machine1 = import ./debug-host-module.nix;

            testScript = ''
              machine.start()
              # assert /foo/bar.txt contains secret 
              machine1.wait_for_file("/foo/bar.txt")
              response = machine1.succeed("cat /foo/bar.txt")
              assert "secret" in response
            '';
          };

        devShells.default = import ./shell.nix { inherit pkgs; };

      })
    );
}
