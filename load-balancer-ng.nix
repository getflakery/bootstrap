{ config, pkgs, inputs, ... }:

let
  domain = "flakery.xyz";
  email = "your-email@example.com";
in
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

      providers = {
        http = {
          endpoint = "https://flakery.dev/api/deployments/lb-config-ng";
          pollInterval = "10s";
        };
      };
    };

    dynamicConfigOptions = {
      tls = {
        stores = {
          default = {
            defaultCertificate = {
              certFile = "/var/lib/acme/certs/${domain}/fullchain.pem";
              keyFile = "/var/lib/acme/certs/${domain}/key.pem";
            };
            certificates = [{
              certfile = "/var/lib/acme/certs/${domain}/fullchain.pem";
              keyfile = "/var/lib/acme/certs/${domain}/key.pem";
            }];
          };
        };
        default.defaultCertificate = {
          certFile = "/var/lib/acme/certs/${domain}/fullchain.pem";
          keyFile = "/var/lib/acme/certs/${domain}/key.pem";
        };
        certificates = [{
          certfile = "/var/lib/acme/certs/${domain}/fullchain.pem";
          keyfile = "/var/lib/acme/certs/${domain}/key.pem";
        }];
      };
      http = {
        routers = {
          loadbal = {
            rule = "Host(`loadb.flakery.xyz`)";
            service = "loadbal-service";
            entryPoints = [ "websecure" ];
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

  # Now we can configure ACME

  security.acme = {
    acceptTerms = true;
    defaults.email = "rwendt1337@gmail.com";
    certs = {
      "${domain}" = {
        domain = domain;
        # Use DNS challenge for wildcard certificates
        dnsProvider = "route53"; # Update this to your DNS provider if different
        environmentFile = "/var/lib/acme/route53-credentials";
      };
    };
  };
}
