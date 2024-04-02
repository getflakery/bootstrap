app:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.app;
in
{
  options.services.app = {
    enable = lib.mkEnableOption "app Service";
    path = [ pkgs.nix pkgs.git ];
    urlPrefix = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "http://0.0.0.0:8000";
      description = "";
    };
    sqlUrl = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "libsql://localhost:8000";
      description = "";
    };
    useLocal = lib.mkOption {
      type = lib.types.str;
      default = "false";
      example = "true";
      description = "";
    };
    applyFlake = lib.mkOption {
      type =  lib.types.str;
      default = "true";
      example = "true";
      description = "";
    };
    testEnv = lib.mkOption {
      type = lib.types.str;
      default = "";
      example = "true";
      description = "in test environment";
    };
    logUrl = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "http://localhost:8000/log";
      description = "";
    };
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
    systemd.services.bootstrap = {
      environment = {
        "URL_PREFIX" = cfg.urlPrefix;
        "SQL_URL" = cfg.sqlUrl;
        "USE_LOCAL" = cfg.useLocal;
        "APPLY_FLAKE" = cfg.applyFlake;
        "TEST_ENV" = cfg.testEnv;
      };
      description = "bootstraper";
      after = cfg.after;
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${app}/bin/app";
        Type = "oneshot";
      };
    };


  };
}
