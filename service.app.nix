app: rebuildSH:
{ config, lib, pkgs, ... }:
let
  cfg = config.services.app;
{
  options.services.app = {
    enable = lib.mkEnableOption "app Service";
    path = [ pkgs.nix pkgs.git ];
    urlPrefix = lib.mkOption {
      type = lib.types.str;
      default = "http://169.254.169.254/latest/meta-data/tags/instance/";
      example = "http://0.0.0.0:8000";
      description = "";
    };
    sqlUrl = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = "libsql://flakery-r33drichards.turso.io";
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
      type = lib.types.str;
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
    setDebugHeaders = lib.mkOption {
      type = lib.types.string;
      default = "false";
      description = "";
    };
    deploymentLogHost = lib.mkOption {
      type = lib.types.str;
      default = "http://localhost:8000";
      example = "http://localhost:8000";
      description = "send rebuild switch logs to this host";
    };
    after = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ "dhcp.service" "network.target" ];
      example = [ "network.target" "serve.service" "seeddb.service" ];
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
        "TEST" = cfg.testEnv;
        "LOG_URL" = cfg.logUrl;
        "SET_DEBUG_HEADER" = cfg.setDebugHeaders;
      };
      description = "bootstraper";
      after = cfg.after;
      wantedBy = [ "multi-user.target" ];
      startLimitIntervalSec = 30;
      startLimitBurst = 50;
      path = [
        pkgs.nix
        pkgs.git
        pkgs.nixos-rebuild
        pkgs.systemd
      ];
      script = ''
        ${app}/bin/app && \
        systemd-run ${rebuildSH}
      '';
      serviceConfig = {
        Type = "simple";
        Restart = "on-failure";
      };
    };
  };
}
