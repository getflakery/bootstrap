app:
{ config, lib, pkgs, ... }:
{

  system.stateVersion = "23.05";

  systemd.services.bootstrap = {
    description = "bootstraper";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${app}/bin/app";
      Restart = "always";
      KillMode = "process";
    };
  };

}
