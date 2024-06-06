{ config, pkgs, inputs, ... }:

{

  imports = [
    inputs.comin.nixosModules.comin
  ];



  networking.firewall.allowedTCPPorts = [ 80 443 ];


  services.tailscale = {
    enable = true;
    authKeyFile = "/tsauthkey";
    extraUpFlags = [ "--ssh" "--hostname" "flakery-tutorial" ];
  };
  services.comin = {
    enable = true;
    hostname = "lb-ng";
    remotes = [
      {
        name = "origin";
        url = "https://github.com/getflakery/bootstrap";
        poller.period = 2;
      }
    ];
  };

  security.acme.acceptTerms = true;
  security.acme.defaults.email = "rwendt1337@gmail.com";

  # /var/lib/acme/.challenges must be writable by the ACME user
  # and readable by the Nginx user. The easiest way to achieve
  # this is to add the Nginx user to the ACME group.
  users.users.nginx.extraGroups = [ "acme" ];

  services.nginx = {
    enable = true;
    virtualHosts = {
      "acmechallenge.flakery.xyz" = {
        # Catchall vhost, will redirect users to HTTPS for all vhosts
        serverAliases = [ "*.flakery.xyz" ];
        locations."/.well-known/acme-challenge" = {
          root = "/var/lib/acme/.challenges";
        };
        locations."/" = {
          return = "301 https://$host$request_uri";
        };
      };
    };
  };

  security.acme.certs."lb.flakery.xyz" = {
    webroot = "/var/lib/acme/.challenges";
    email = "rwendt1337@gmail.com";
    # Ensure that the web server you use can read the generated certs
    # Take a look at the group option for the web server you choose.
    group = "nginx";
    # Since we have a wildcard vhost to handle port 80,
    # we can generate certs for anything!
    # Just make sure your DNS resolves them.
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
      # route lb.flakery.dev to 
      providers = {
        http = {
          endpoint = "https://flakery.dev/api/deployments/lb-config-ng";
          pollInterval = "10s";
        };
      };
    };
  };
}
