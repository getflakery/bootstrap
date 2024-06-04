app:
{ config, lib, pkgs, ... }:
let
  cfg = config.services.app;
  rebuildScript = pkgs.writeShellScript "rebuild.sh" (lib.optionalString (cfg.applyFlake == "true") ''
      export RUST_BACKTRACE=1
      export DEPLOYMENT=`${app}/bin/app --print-deployment-id`
      export NIX_CONFIG="access-tokens = github.com=`${app}/bin/app --print-github-token`"
      ${pkgs.fluent-bit}/bin/fluent-bit \
        -i exec -p 'command=${pkgs.nixos-rebuild}/bin/nixos-rebuild switch --flake `${app}/bin/app --print-flake` --refresh --no-write-lock-file --impure 2>&1' \
        -o http://flakery.dev/api/deployments/log/rebuild/$DEPLOYMENT -p 'tls=on' -m '*' -p 'Port=443' -p 'Format=json' \
        -p exit_after_oneshot=true \
        -p propagate_exit_code=true \
        -p oneshot=true

  '');
in
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
        systemd-run ${rebuildScript}
      '';
      serviceConfig = {
        Type = "simple";
        Restart = "on-failure";
      };
    };
  };
}
