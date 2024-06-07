{
  description = "A basic rust cli";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.fenix = {
    url = "github:nix-community/fenix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  inputs.nixos-generators = {
    url = "github:nix-community/nixos-generators";
    inputs.nixpkgs.follows = "nixpkgs";
  };


  inputs.flakery.url = "github:getflakery/flakes";

  inputs.comin.url = "github:r33drichards/comin/8f8352537ca4ecdcad06b1b4ede4465d37dbd00c";


  outputs =
    { self
    , nixpkgs
    , flake-utils
    , nixos-generators
    , flakery
    , fenix
    , comin
    , ...
    }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
      let
        toolchain = fenix.packages.${system}.minimal.toolchain;

        pkgs = import nixpkgs {
          inherit system;
        };

        bootstrap = (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        }).buildRustPackage {
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
                  pkgs.lib.hasSuffix ".py"
                  baseName
              );
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = with pkgs;
            lib.optionals stdenv.isDarwin [
              darwin.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
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
        vectorConfig = pkgs.writeText "vector.yaml" builtins.readFile ./vector.yaml;
        helloVector = pkgs.writeScript "vector.sh" ''
          echo "Hello, Vector!" | ${pkgs.vector}/bin/vector --config ${vectorConfig}/vector.yaml
        '';
        rebuildScript = app: ''
          export RUST_BACKTRACE=1
          export DEPLOYMENT=$(${app}/bin/app --print-deployment-id)
          export NIX_CONFIG="access-tokens = github.com=$(${app}/bin/app --print-github-token)"
          ${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake $(${app}/bin/app --print-flake) --refresh --no-write-lock-file --impure 2>&1 | \
          ${pkgs.vector}/bin/vector --config ${vectorConfig}/vector.yaml
        '';
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
          rebuild = flake-utils.lib.mkApp {
            drv = pkgs.writeScript "rebuild.sh" (rebuildScript bootstrap);
            exePath = "/bin/rebuild.sh";
          };
          vector = flake-utils.lib.mkApp {
            drv = helloVector;
            exePath = "/bin/vector.sh";
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
            flakery.nixosModules.flakery
            ./load-balancer.nix
            sshconfMod
          ];
        };


        packages.nixosConfigurations.lb-ng = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs;
          };

          modules = [
            flakery.nixosModules.flakery
            ./load-balancer-ng.nix
            sshconfMod
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
        packages.testlb = pkgs.testers.runNixOSTest
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


                systemd.services.serve = {
                  wantedBy = [ "multi-user.target" ];
                  path = [ pkgs.python3 ];
                  script = "${./serve.py} --enable-lb";
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
              # assert /etc/deployment_id contains deployment_id
              machine1.wait_for_file("/etc/deployment_id")
              response = machine1.succeed("cat /etc/deployment_id")
              assert "00f00f" in response
            '';
          };
        packages.test = pkgs.testers.runNixOSTest
          {
            skipLint = true;
            name = "Test bootstrap";

            nodes = {

              machine1 = { pkgs, ... }: {

                environment.systemPackages = [ pkgs.sqlite ];

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
              # todo add me back
              # response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT * FROM target;'")
              # assert "00f00f" in response
            '';
          };
      })
    );
}
