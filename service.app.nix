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
      default = [  "dhcp.service"  "network.target" ];
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
        "TEST" = cfg.testEnv;
        "LOG_URL" = cfg.logUrl;
      };
      description = "bootstraper";
      after = cfg.after;
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${app}/bin/app";
        Type = "simple";
      };
    };


    security.sudo.wheelNeedsPassword = false;

    users.users.alice = {
      isNormalUser = true;
      extraGroups = [ "wheel" ]; # Enable ‘sudo’ for the user.
      packages = with pkgs; [ ];
      group = "alice";
      # set shell to zsh 
      # passwordFile = "/persist/passwords/alice";
    };

    users.groups.alice = { };
    services.openssh = {
      enable = true;
      # require public key authentication for better security
      settings.PasswordAuthentication = false;
      settings.KbdInteractiveAuthentication = false;
      # settings.PermitRootLogin = "yes";
    };
    users.users."alice".openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFt1Kc7AuNgW0n+Zu4bMfRAWFfScLbzivxNtqC69dTS+ alice@ip-10-0-0-229.us-west-1.compute.internal" # content of authorized_keys file
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDK9oSqErvoipqDl8hx0WEiWeLEfdEOPqbwVzVLNGgRF2s7Nn45DBduZCmpRMSEYDbPOtN+jxa/mj4/omiRv1Y6jTMl1YYzfEOJdwdjhf/T8x1oXbOIsfgoYZnJpUfmIBGaqtSzIU/zWUKYENc6EtfzEjV98tdtxPd23QpzNWsXTD2BcIYEizGD75lbnDBb2EZQZhnNnTk62zzxL42pQ7g6SdVGBmVDV4IduUGPX2O1AF9DeIeorrrTkqeTSVmK8Q86GN33Tx7ClQIqWLzwanMbCYeFvpl8/Y3AMDVFGPwcAm/6MpiKR6XkxuSL5zYQ6Ys3c9L+drd1bnpmWRSsfm0OdeUj9DaCVWY7oQ5O/KviwBVu+J5AoNQDxfcX+a+KKDqHO0q6N0Gd5xMaB0rRILkJRRteg65tSaLLWVQKwjcdU9ydbmC6GC2hCxgVtipTIgispp+GqOk+5c/NZEZ+zkUuNhsmbRNsgANxbgrbNrfATTk1T7fMiQZFFtfuzhjg86M= robertwendt@roberts-air.lan"

      # note: ssh-copy-id will add user@your-machine after the public key
      # but we can remove the "@your-machine" part
    ];


  };
}
