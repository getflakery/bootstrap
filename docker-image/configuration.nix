{ pkgs, config, lib, ... }:
{
  imports = [
    <nixpkgs/nixos/modules/virtualisation/docker-image.nix>
    # <nixpkgs/nixos/modules/installer/cd-dvd/channel.nix>
  ];

  documentation.doc.enable = false;

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

  environment.systemPackages = with pkgs; [
    bashInteractive
    cacert
    nix
  ];
}
