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
      # create a route and service for the lb.flakery.dev domain that points localhost:443
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
