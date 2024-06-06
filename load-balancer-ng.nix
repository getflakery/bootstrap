{ config, pkgs, ... }:

{
  networking.firewall.allowedTCPPorts = [ 80 443 ];

  services.tailscale = {
    enable = true;
    authKeyFile = "/tsauthkey";
    extraUpFlags = [ "--ssh" "--hostname" "flakery-tutorial" ];
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
  };
}
