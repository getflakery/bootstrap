{ config, pkgs, ... }:

let
  # Define paths for your configuration files
  # read from /etc/deployment_id
  deploymentID = builtins.readFile "/etc/deployment_id";
  configURL = "https://flakery.dev/api/deployments/lb-config/${deploymentID}";
in
{
  networking.firewall.allowedTCPPorts = [ 80 443 ];

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
          endpoint = configURL;
          pollInterval = "10s";
        };
      };
    };
  };
}
