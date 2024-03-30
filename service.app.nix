app:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.app;
in
{
  options.services.app = {
    enable = lib.mkEnableOption "app Service";

    urlPrefix = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "http://0.0.0.0:8000";
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
  };
}
