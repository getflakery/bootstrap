{
  description = "A basic rust cli";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/23.11";
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

        bootstrap = pkgs.rustPlatform.buildRustPackage {
          pname = "bootstrap";
          version = "0.0.1";
          # src = ./.;
          # filter nix files 
          src = pkgs.lib.sources.cleanSourceWith {
            src = ./.;
            filter = name: type:
              let
                baseName = baseNameOf (toString name);
              in
              (
                (pkgs.lib.sources.cleanSourceFilter name type) ||
                # base name ends with .nix
                pkgs.lib.hasSuffix ".nix" baseName ||
                baseName == ".direnv" ||
                baseName == "target" ||
                # has prefix flake 
                pkgs.lib.hasPrefix "flake" baseName
                # has suffix .py
                pkgs.lib.hasSuffix ".py" baseName
              );
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = with pkgs; [
            # darwin.Security # todo only if darwin
            # darwin.apple_sdk.frameworks.SystemConfiguration # todo only if darwin
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          buildPhase = ''
            RUST_BACKTRACE=1 cargo build --release -p bootstrap
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/bootstrap $out/bin/app
          '';

          # disable checkPhase
          doCheck = false;
        };
        bootstrapModules = [
          flakery.nixosModules.flakery
          {
            imports = [
              self.nixosModules."${system}".bootstrap

            ];
            services.app.enable = true;
            services.app.deploymentLogHost = "flakery.dev";
          }
        ];
        sshconfMod = {
          users.users.flakery = {
            isNormalUser = true;
            extraGroups = [ "wheel" ]; # Enable ‘sudo’ for the user.
            password = "flakery"; # Set the password for the user.
          };
          # allow sudo without password for wheel
          security.sudo.wheelNeedsPassword = false;

          services.openssh = {
            enable = true;
            # require public key authentication for better security
            settings.PasswordAuthentication = false;
            settings.KbdInteractiveAuthentication = false;
          };

          users.users."flakery".openssh.authorizedKeys.keys = [
            # replace with your ssh key 
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCs/e5M8zDNH5DUmqCGKM0OhHKU5iHFum3IUekq8Fqvegur7G2fhsmQnp09Mjc5pEw2AbfTYz11WMHsvC5WQdRWSS2YyZHYsPb9zIsVBNcss+H5x63ItsDjmbrS6m/9r7mRBOiN265+Mszc5lchFtRFetpi9f+EBis9r8atyPlsz86IoS2UxSSWonBARU4uwy2+TT7+mYg3cQf7kp1Y1sTqshXmcHUC5UVSRk3Ny9IbIMhk19fOxr3y8gaXoT5lB0NSLO8XFNbNT6rjZXH1kpiPJh3xLlWBPQtbcLrpm8oSS51zH7+zAGb7mauDHu2RcfBgq6m1clZ6vff65oVuHOI7"
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCqWQCbzrNA2JSWktRiN/ZCBihwgE7D9HJSvHqjdw/TOL8WrHVkkBCp8nm3z5THeXDAfpr5tYDE2KU0f6LSr88bmbn7DjAORgdTKdyJpzHGQeaS3YWnTi+Bmtv4mvCWk5HCCei0pciTh5KS8FFU8bGruFEUZAmDyk1EllFC+Gx8puPrAL3tl5JX6YXzTFFZirigJIlSP22WzN/1xmj1ahGo9J0E88mDMikPBs5+dhPOtIvNdd/qvi/wt7Jnmz/mZITMzPaKrei3gRQyvXfZChJpgGCj0f7wIzqv0Hq65kMILayHVT0F2iaVv+bBSvFq41n3DU4f5mn+IVIIPyDFaG/X"
          ];

        };
      in
      {
        # Executed by `nix run .#<name>`
        apps = {
          default = flake-utils.lib.mkApp {
            drv = bootstrap;
            exePath = "/bin/app";
          };
          bootstrap = flake-utils.lib.mkApp {
            drv = bootstrap;
            exePath = "/bin/app";
          };
        };
        packages.default = bootstrap;

        # devShells.default = app;
        devShells.default = import ./shell.nix { inherit pkgs; };
        packages.bootstrap = bootstrap;

        nixosModules.bootstrap = ((import ./service.app.nix) self.packages."${system}".bootstrap);

        packages.nixosConfigurations.bootstrap = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = bootstrapModules;
        };

        packages.nixosConfigurations.lb = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            ./load-balancer.nix
          ];
        };
        
        packages.ami = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "amazon";
          modules = bootstrapModules ++ [
            sshconfMod
          ];

        };
        packages.amiDebug = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "amazon";
          modules = [
            sshconfMod
          ];
        };
        packages.test = pkgs.testers.runNixOSTest
          {
            skipLint = true;
            name = "Test bootstrap";

            nodes = {
              machine1 = { pkgs, ... }: {

                # Empty config sets some defaults
                imports = [
                  self.nixosModules."${system}".bootstrap
                ];

                services.app.enable = true;
                services.app.urlPrefix = "http://localhost:8080/";
                services.app.sqlUrl = "file:///tmp/db.sqlite3";
                services.app.useLocal = "true";
                services.app.applyFlake = "false";
                services.app.setDebugHeaders = "true";

                services.app.after = [ "network.target" "serve.service" "seeddb.service" ];


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
      })
    );
}
