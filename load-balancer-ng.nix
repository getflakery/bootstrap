{ config, pkgs, inputs, ... }:

{
  imports = [
    inputs.comin.nixosModules.comin
  ];

  networking.firewall.allowedTCPPorts = [ 80 443 ];

  services.tailscale = {
    enable = true;
    authKeyFile = "/tsauthkey";
    extraUpFlags = [ "--ssh" "--hostname" "flakery-load-balancer" ];
  };

  services.comin = {
    enable = true;
    hostname = "lb-ng";
    remotes = [
      {
        name = "origin";
        url = "https://github.com/getflakery/bootstrap";
        poller.period = 2;
        branches.main.name = "master";
      }
    ];
  };

  # Enable the Traefik service
  services.traefik = {
    enable = true;

    # Path to the Traefik binary
    package = pkgs.traefik;

    staticConfigOptions = {
      entryPoints = {
        web = {
          address = ":80";
        };
        websecure = {
          address = ":443";
        };
      };
      certificatesResolvers = {
        letsencrypt = {
          acme = {
            email = "rwendt1337@gmail.com";
            storage = "/var/lib/traefik/acme.json";
            httpChallenge = {
              entryPoint = "web";
            };
          };
        };
      };
      providers = {
        http = {
          endpoint = "https://flakery.dev/api/deployments/lb-config-ng";
          pollInterval = "10s";
        };
      };
    };

    dynamicConfigOptions = {
      http = {
        routers = {
          loadbal = {
            rule = "Host(`loadb.flakery.xyz`)";
            service = "loadbal-service";
            entryPoints = [ "websecure" ];
            tls = {
              certResolver = "letsencrypt";
            };
          };
        };
        services = {
          loadbal-service = {
            loadBalancer = {
              servers = [
                { url = "http://0.0.0.0:443"; }
              ];
            };
          };
        };
      };
    };
  };
}
