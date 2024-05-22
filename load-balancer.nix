{ config, pkgs, ... }:

let
  # Define paths for your configuration files
  # read from /etc/deployment_id
  deploymentID = builtins.readFile "/etc/deployment_id";
  configURL = "https://flakery.dev/api/deployments/lb-config/${deploymentID}";
in
{
  # Enable the Traefik service
  services.traefik = {
    enable = true;

    # Path to the Traefik binary
    package = pkgs.traefik;
# entryPoints:
#   web:
#     address: ":80"
#   websecure:
#     address: ":443"

# certificatesResolvers:
#   myresolver:
#     acme:
#       email: your-email@example.com
#       storage: acme.json
#       httpChallenge:
#         entryPoint: web

# providers:
#   http:
#     endpoint:
#       url: "http://config-server:8000/traefik/dynamic-config"
#     pollInterval: "10s"

    staticConfigOptions = {
      entryPoints = {
        web = {
          address = ":80"
        };
        websecure = {
          address = ":443"
        };
      };
      certificatesResolvers = {
        myresolver = {
          acme = {
            email = "rwendt1337@gmail.com";
            storage = "acme.json";
            httpChallenge = {
              entryPoint = "web";
            };
          };
        };
      };
      providers = {
        http = {
          endpoint = {
            url = configURL;
          };
          pollInterval = "10s";
        };
      };
    };
  };
}

