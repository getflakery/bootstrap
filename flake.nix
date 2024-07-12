{
  description = "A basic rust cli";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/24.05";
  inputs.unstable.url = "github:NixOS/nixpkgs/nixos-unstable";


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

  inputs.comin.url = "github:r33drichards/comin/c10b125b364bf13e3a11293a34a6e4d2fd0fcd4b";


  outputs =
    { self
    , nixpkgs
    , flake-utils
    , nixos-generators
    , flakery
    , fenix
    , comin
    , unstable
    , ...
    }@inputs:
    (flake-utils.lib.eachDefaultSystem
      (system:
      let
        toolchain = fenix.packages.${system}.minimal.toolchain;

        pkgs = import nixpkgs {
          inherit system;
        };
        upkgs = import unstable {
          inherit system;
        };


        woodpecker = pkgs.woodpecker-server.overrideAttrs (final: prev:
          {
            src = pkgs.fetchFromGitHub {
              owner = "woodpecker-ci";
              repo = "woodpecker";
              rev = "v2.6.0";
              sha256 = "sha256-SuTizOHsj1t4WovbOX5MuMZixbPo7TyCnD6nnf62/H4=";
            };
            vendorHash = null;
            version = "2.6.0";
            pname = "woodpecker-server";

            subPackages = "cmd/server";

            CGO_ENABLED = 1;

            passthru = {
              updateScript = ./update.sh;
            };

          });


        # )
        # woodpecker-agent = (pkgs.callPackage ./agent.nix { });
        woodpecker-agent = pkgs.woodpecker-agent.overrideAttrs (final: prev:
          {
            src = pkgs.fetchFromGitHub {
              owner = "woodpecker-ci";
              repo = "woodpecker";
              rev = "v2.6.0";
              sha256 = "sha256-SuTizOHsj1t4WovbOX5MuMZixbPo7TyCnD6nnf62/H4=";
            };
            vendorHash = null;
            version = "2.6.0";
            subPackages = "cmd/agent";
            CGO_ENABLED = 0;
          });


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
          };
          # allow sudo without password for wheel
          security.sudo.wheelNeedsPassword = false;

          # port 22
          networking.firewall.allowedTCPPorts = [ 22 ];

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
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJLb6cphbbtWQEVDpotwTY9IAam6WFpt8Dluap4wFiww root@ip-10-0-2-147.us-west-2.compute.internal"
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGENQMz7ldqG4Zk/wfcwz1Uhl67eP5TLx1ZEmOUbqkME rw@rws-MacBook-Air.local"
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIK9tjvxDXYRrYX6oDlWI0/vbuib9JOwAooA+gbyGG/+Q robertwendt@Roberts-Laptop.local"
          ];

        };
        vectorConfigText = builtins.readFile ./vector.yaml;

        vectorConfig = builtins.toFile "vector.yaml" vectorConfigText;
        helloVector = pkgs.writeScript "vector.sh" ''
          ${pkgs.vector}/bin/vector --config ${vectorConfig}
        '';
        vectorStdout = builtins.toFile "vector.yaml" (builtins.readFile ./stdout.yaml);
        stdoutScript = pkgs.writeScript "stdout.sh" ''
          ${pkgs.vector}/bin/vector --config ${vectorStdout}
        '';

        exit1 = pkgs.writeScript "exit1.sh" ''
          echo '{ "hello": "world" }'
          exit 1
        '';

        exit0 = pkgs.writeScript "exit0.sh" ''
          echo '{ "hello": "world" }'
          exit 0
        '';

        vec0 = pkgs.writeShellApplication {
          name = "vec0";
          text = ''
            ${exit0} | ${stdoutScript}
            echo $?
          '';
          checkPhase = "";
        };

        vec1 = upkgs.writeShellApplication {
          name = "vec1";
          text = ''
            ${exit1} | ${stdoutScript}
            echo $?
          '';
          checkPhase = "";
          bashOptions = [ "nounset" "pipefail" ];
        };

        rebuildScript = app: ''
          export RUST_BACKTRACE=1
          export DEPLOYMENT=$(${app}/bin/app --print-deployment-id)
          export NIX_CONFIG="access-tokens = github.com=$(${app}/bin/app --print-github-token)"
          ${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake $(${app}/bin/app --print-flake) --refresh --no-write-lock-file --impure 2>&1 | \
          ${app}/bin/app --wrap_with_deployment_id | \
          ${helloVector}
          ${app}/bin/app --exit-code $?
          touch /tmp/rebuilt
        '';
        rebuildSH = upkgs.writeShellApplication {
          name = "rebuild";
          text = rebuildScript bootstrap;
          checkPhase = "";
          bashOptions = [ "nounset" "pipefail" ];


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
          rebuild = flake-utils.lib.mkApp {
            drv = rebuildSH;
            exePath = "/bin/rebuild";
          };
          vector = flake-utils.lib.mkApp {
            drv = helloVector;
            exePath = "";
          };
          stdout = flake-utils.lib.mkApp {
            drv = stdoutScript;
            exePath = "";
          };
          vec0 = flake-utils.lib.mkApp {
            drv = vec0;
          };
          vec1 = flake-utils.lib.mkApp {
            drv = vec1;
          };
          exit0 = flake-utils.lib.mkApp {
            drv = exit0;
            exePath = "";

          };
        };
        packages.default = bootstrap;

        # devShells.default = app;
        devShells.default = import ./shell.nix { inherit pkgs; };
        packages.bootstrap = bootstrap;

        nixosModules.bootstrap = (((import ./service.app.nix) self.packages."${system}".bootstrap) rebuildSH);

        packages.nixosConfigurations.bootstrap = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = bootstrapModules;
        };


        packages.nixosConfigurations.debugSystem = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs;
          };

          modules = [
            flakery.nixosModules.flakery
            sshconfMod
            {
              networking.firewall.allowedTCPPorts = [ 22 80 443 ];

              services.tailscale = {
                enable = true;
                authKeyFile = "/tsauthkey";
                extraUpFlags = [ "--ssh" "--hostname" "debug-flakery" ];
              };

              virtualisation.docker.enable = true;


              # simple caddy server on port 8080
              services.caddy = {
                enable = true;
                config = ''
                  http://localhost:8080 {
                    respond "Hello, world!"
                  }
                '';
              };
            }
          ];
        };

        packages.nixosConfigurations.grafana = nixpkgs.lib.nixosSystem
          {
            inherit system;
            specialArgs = {
              inherit inputs;
            };
            modules = [
              inputs.comin.nixosModules.comin
              flakery.nixosModules.flakery
              sshconfMod
              {
                networking.firewall.allowedTCPPorts = [ 3000 ];

                services.prometheus = {
                  enable = true;
                  port = 9090;
                  exporters = {
                    node = {
                      enable = true;
                      enabledCollectors = [ "systemd" ];
                      port = 9002;
                    };

                  };
                  scrapeConfigs = [
                    {
                      job_name = "node";
                      static_configs = [{
                        targets = [
                          "127.0.0.1:9002"
                          "flakery-load-balancer-2:9002"
                          "flakery-load-balancer-1:9002"
                          "flakery-load-balancer:9002"
                          "woodpecker:9002"
                          "woodpecker-1:9002"
                          "woodpecker-2:9002"
                          "ip-10-0-4-215:9002"
                        ];
                      }];
                    }
                  ];
                };

                nix = {
                  gc = {
                    automatic = true;
                    dates = "weekly";
                    options = "--delete-older-than 7d";
                  };
                  settings = {
                    experimental-features = [ "nix-command" "flakes" ];
                    substituters = [
                      # "https://cache.garnix.io"
                      "https://nix-community.cachix.org"
                      "https://cache.nixos.org/"
                      "https://binary-cache-6b1b4a.flakery.xyz"
                    ];
                    trusted-public-keys = [
                      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
                      # "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
                      "binary-cache-6b1b4a.flakery.xyz:Du7IeCqQQiJpvdhizPnX2ZN2GTlMeUR7C+r9x8Xkjz0="
                    ];
                  };
                };

                services.comin = {
                  enable = true;
                  hostname = "grafana";
                  remotes = [
                    {
                      name = "origin";
                      url = "https://github.com/getflakery/bootstrap";
                      poller.period = 2;
                      branches.main.name = "master";
                    }
                  ];
                };

                services.tailscale = {
                  enable = true;
                  authKeyFile = "/tsauthkey";
                  extraUpFlags = [ "--ssh" "--hostname" "grafana" ];
                };
                services.grafana = {
                  enable = true;
                  analytics.reporting.enable = false;


                  settings = {
                    server = {
                      domain = builtins.readFile /grafana-domain;
                      root_url = builtins.readFile /grafana-root-url;
                      http_addr = "0.0.0.0";
                      http_port = 3000;
                    };
                  };
                };

                #
                services.loki = {
                  enable = true;
                  configFile = "${pkgs.grafana-loki.src}/cmd/loki/loki-local-config.yaml";
                };
                services.promtail = {
                  enable = true;
                  configuration = {
                    server = {
                      http_listen_port = 9080;
                      grpc_listen_port = 0;
                    };
                    clients = [{ url = "http://localhost:3100/loki/api/v1/push"; }];
                    scrape_configs = [
                      {
                        job_name = "system";
                        static_configs = [
                          {
                            targets = [ "localhost" ];
                            labels = {
                              job = "varlogs";
                              __path__ = "/var/log/*log";
                            };
                          }

                        ];
                      }
                      {
                        job_name = "journal";
                        journal = {
                          max_age = "12h";
                          labels = {
                            job = "systemd-journal";
                            host = "grafana";
                          };
                        };
                        relabel_configs = [{
                          source_labels = [ "__journal__systemd_unit" ];
                          target_label = "unit";
                        }];
                      }

                    ];
                  };
                };
              }
            ];
          };

        packages.nixosConfigurations.woodpecker = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs;
          };
          modules = [
            inputs.comin.nixosModules.comin
            flakery.nixosModules.flakery
            flakery.nixosConfigurations.base
            {
              services.promtail = {
                enable = true;
                configuration = {
                  server = {
                    http_listen_port = 9080;
                    grpc_listen_port = 0;
                  };
                  clients = [{ url = "http://grafana:3100/loki/api/v1/push"; }];
                  scrape_configs = [
                    {
                      job_name = "system";
                      static_configs = [
                        {
                          targets = [ "localhost" ];
                          labels = {
                            job = "varlogs";
                            __path__ = "/var/log/*log";
                          };
                        }

                      ];
                    }
                    {
                      job_name = "journal";
                      journal = {
                        max_age = "12h";
                        labels = {
                          job = "systemd-journal";
                          host = "woodpecker";
                        };
                      };
                      relabel_configs = [{
                        source_labels = [ "__journal__systemd_unit" ];
                        target_label = "unit";
                      }];
                    }

                  ];
                };
              };

              networking.firewall.allowedTCPPorts = [ 3007 9002 ];
              services.prometheus = {
                enable = true;
                port = 9090;
                exporters = {
                  node = {
                    enable = true;
                    enabledCollectors = [ "systemd" ];
                    port = 9002;
                  };

                };
              };

              services.tailscale = {
                enable = true;
                authKeyFile = "/tsauthkey";
                extraUpFlags = [ "--ssh" "--hostname" "woodpecker" ];
              };

              services.woodpecker-server = {
                enable = true;
                package = woodpecker;

                environment = {
                  WOODPECKER_SERVER_ADDR = ":3007";
                  WOODPECKER_HOST = "https://woodpecker-ci-19fcc5.flakery.xyz";
                  WOODPECKER_OPEN = "true";
                  WOODPECKER_ORGS = "getflakery";
                  WOODPECKER_GITHUB = "true";
                  WOODPECKER_GITHUB_CLIENT = "Ov23li77VshZc9W7M4Gp";
                  WOODPECKER_GITHUB_SECRET = builtins.readFile /github-client-secret;
                  WOODPECKER_AGENT_SECRET = builtins.readFile /agent-secret;
                  WOODPECKER_ADMIN = "r33drichards";
                  WOODPECKER_DATABASE_DRIVER = "postgres";
                  WOODPECKER_DATABASE_DATASOURCE = builtins.readFile /pgurl;
                };
                # You can pass a file with env vars to the system it could look like:
                # environmentFile = "/path/to/my/secrets/file";
              };

              # This sets up a woodpecker agent
              services.woodpecker-agents.agents."docker" = {
                enable = true;
                package = woodpecker-agent;

                # We need this to talk to the podman socket
                extraGroups = [ "podman" ];
                environment = {
                  WOODPECKER_SERVER = "localhost:9000";
                  WOODPECKER_MAX_WORKFLOWS = "4";
                  DOCKER_HOST = "unix:///run/podman/podman.sock";
                  WOODPECKER_BACKEND = "docker";
                  WOODPECKER_AGENT_SECRET = builtins.readFile /agent-secret;


                };
                # Same as with woodpecker-server
                # environmentFile = [ "/var/lib/secrets/woodpecker.env" ];
              };

              # Here we setup podman and enable dns
              virtualisation.podman = {
                enable = true;
                defaultNetwork.settings = {
                  dns_enabled = true;
                };
                dockerSocket.enable = true;
              };
              # This is needed for podman to be able to talk over dns
              networking.firewall.interfaces."podman0" = {
                allowedUDPPorts = [ 53 ];
                allowedTCPPorts = [ 53 ];
              };
            }

          ];
        };

        packages.nixosConfigurations.binary-cache = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = {
            inherit inputs;
          };
          modules = [
            inputs.comin.nixosModules.comin
            flakery.nixosModules.flakery
            flakery.nixosConfigurations.base
            sshconfMod
            {
              networking.firewall.allowedTCPPorts = [ 5000 9002 ];
              # set perms fcpr "/var/cache-priv-key.pem" to 600 
              # before running nix-serve
              systemd.services.setPerms = {
                wantedBy = [ "multi-user.target" ];
                script = ''
                  cd /var
                  # todo curl is tech debt \o/
                  ${pkgs.nix}/bin/nix-store --generate-binary-cache-key `${pkgs.curl}/bin/curl http://169.254.169.254/latest/meta-data/local-ipv4` cache-priv-key.pem cache-pub-key.pem
                  chmod 600 /var/cache-priv-key.pem
                '';
                serviceConfig = {
                  Type = "oneshot";
                };
              };

              services.nix-serve = {
                enable = true;
                secretKeyFile = "/var/cache-priv-key.pem";
              };
              # follow setPerms 
              systemd.services.nix-serve.after = [ "setPerms.service" ];

              # add flakery as trusted user
              nix.settings.trusted-users = [ "flakery" ];
              services.promtail = {
                enable = true;
                configuration = {
                  server = {
                    http_listen_port = 9080;
                    grpc_listen_port = 0;
                  };
                  clients = [{ url = "http://grafana:3100/loki/api/v1/push"; }];
                  scrape_configs = [
                    {
                      job_name = "system";
                      static_configs = [
                        {
                          targets = [ "localhost" ];
                          labels = {
                            job = "varlogs";
                            __path__ = "/var/log/*log";
                          };
                        }

                      ];
                    }
                    {
                      job_name = "journal";
                      journal = {
                        max_age = "12h";
                        labels = {
                          job = "systemd-journal";
                          host = "binary-cache";
                        };
                      };
                      relabel_configs = [{
                        source_labels = [ "__journal__systemd_unit" ];
                        target_label = "unit";
                      }];
                    }

                  ];
                };
              };

              services.prometheus = {
                enable = true;
                port = 9090;
                exporters = {
                  node = {
                    enable = true;
                    enabledCollectors = [ "systemd" ];
                    port = 9002;
                  };

                };
              };

              services.comin = {
                enable = true;
                hostname = "binary-cache";
                remotes = [
                  {
                    name = "origin";
                    url = "https://github.com/getflakery/bootstrap";
                    poller.period = 2;
                    branches.main.name = "master";
                  }
                ];
              };

            }
          ];
        };



        packages.ami = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "amazon";
          # modules = bootstrapModules ++ [ sshconfMod ];
          modules = bootstrapModules;
        };

        packages.amiDebug = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "amazon";
          modules = [
            sshconfMod
          ];
        };

        packages.raw = nixos-generators.nixosGenerate {
          system = "x86_64-linux";
          format = "raw";

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

                environment.systemPackages = [ pkgs.sqlite pkgs.gnugrep ];

                # Empty config sets some defaults
                imports = [
                  self.nixosModules."${system}".bootstrap
                ];

                services.app.enable = true;
                services.app.urlPrefix = "http://localhost:8080/";
                services.app.ipv4Prefix = "http://localhost:8080/";
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
            testScript = builtins.readFile ./testScript.py;
          };
        packages.testWriteFiles = pkgs.testers.runNixOSTest
          {
            skipLint = true;
            name = "Test bootstrap write files";



            nodes = {

              machine1 = { pkgs, ... }: {

                environment.systemPackages = [ pkgs.sqlite pkgs.gnugrep ];

                # Empty config sets some defaults
                imports = [
                  self.nixosModules."${system}".bootstrap
                ];

                services.app.enable = true;
                services.app.urlPrefix = "http://localhost:8080/";
                services.app.ipv4Prefix = "http://localhost:8080/";
                services.app.sqlUrl = "file:///tmp/db.sqlite3";
                services.app.useLocal = "true";
                services.app.applyFlake = "false";
                services.app.setDebugHeaders = "true";

                services.app.after = [ "network.target" "serve.service" "seeddb.service" ];
                services.app.script = "${bootstrap}/bin/app --write-files --template-id 0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f --encryption-key 0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f";


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
