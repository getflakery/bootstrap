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
            cargo build --release -p webserver
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/webserver $out/bin/app
          '';

          # disable checkPhase
          doCheck = false;

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
            cargo build --release -p bootstrap
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
            services.app.logUrl = "https://p.jjk.is/log";
          }
        ];
      in
      {
        # Executed by `nix run .#<name>`
        apps = {
          app = flake-utils.lib.mkApp { drv = app; };
          default = flake-utils.lib.mkApp { drv = app; };
          webserver = flake-utils.lib.mkApp { drv = app; };
          bootstrap = flake-utils.lib.mkApp {
            drv = bootstrap;
            exePath = "/bin/app";
          };
        };
        packages.default = app;
        packages.app = app;

        # devShells.default = app;
        devShells.default = import ./shell.nix { inherit pkgs; };
        packages.bootstrap = bootstrap;

        nixosModules.bootstrap = ((import ./service.app.nix) self.packages."${system}".bootstrap);
        nixosModules.webserver = ((import ./service.webserver.nix) self.packages."${system}".app);

        packages.nixosConfigurations.bootstrap = nixpkgs.lib.nixosSystem {
          inherit system;

          modules = bootstrapModules;
        };

        packages.nixosConfigurations.webserver = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            {
              imports = [
                self.nixosModules."${system}".webserver
              ];
              services.webserver.enable = true;
            }
          ];
        };
        packages.ami = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "amazon";
          modules = bootstrapModules ++ [
            {
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
              ];

            }
          ];

        };
        packages.amiDebug = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "docker";
          modules = [
            {
              services.tailscale.enable = true;
              systemd.services.tailscale-autoconnect = {
                description = "Automatic connection to Tailscale";

                # make su`re tailscale is running before trying to connect to tailscale
                after = [ "network-pre.target" "tailscale.service" ];
                wants = [ "network-pre.target" "tailscale.service" ];
                wantedBy = [ "multi-user.target" ];

                # set this service as a oneshot job
                serviceConfig.Type = "oneshot";

                # have the job run this shell script
                script = with pkgs; ''
                  # wait for tailscaled to settle
                  sleep 2

                  # check if we are already authenticated to tailscale
                  status="$(${tailscale}/bin/tailscale status -json | ${jq}/bin/jq -r .BackendState)"
                  if [ $status = "Running" ]; then # if so, then do nothing
                    exit 0
                  fi

                  # otherwise authenticate with tailscale
                  ${tailscale}/bin/tailscale up --ssh -authkey '' + "tskey-auth-kZX6CpmaY111CNTRL-Ay4cxbqjyJ7ihHv4C9X9J7prHj2AXcSUe" + '' -auth- --hostname test-ami
                '';
              };
            }
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
                  self.nixosModules."${system}".webserver
                ];

                services.app.enable = true;
                services.app.urlPrefix = "http://localhost:8080/";
                services.app.sqlUrl = "file:///tmp/db.sqlite3";
                services.app.useLocal = "true";
                services.app.applyFlake = "false";
                services.app.setDebugHeaders = "true";

                services.app.after = [ "network.target" "serve.service" "seeddb.service" ];

                services.webserver.enable = true;

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
              response = machine1.succeed("journalctl -xeu webserver.service")
              assert "Log:" in response
            '';
          };
      })
    );
}
