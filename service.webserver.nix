app:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.webserver;
in
{
  options.services.webserver = {
    enable = lib.mkEnableOption "webserver Service";
    path = [ pkgs.nix pkgs.git ];

    after = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [  "network.target" ];
      example = [ "network.target" "serve.service" "seeddb.service"];
      description = "";
    };
  };

  config = lib.mkIf cfg.enable {
    system.stateVersion = "23.05";

    nix = {
      settings = {
        substituters = [
          "https://cache.garnix.io"
          "https://nix-community.cachix.org"
          "https://cache.nixos.org/"
        ];
        trusted-public-keys = [
          "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
          "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
        ];
      };
    };
    systemd.services.webserver = {
      environment = {

      };
      description = "webserver";
      after = cfg.after;
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${app}/bin/app";
        Type = "simple";
      };
    };


  };
}
