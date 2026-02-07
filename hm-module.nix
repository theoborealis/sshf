{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.sshf;
  modeFlag = if cfg.mode == "whitelist" then "--whitelist" else "--blacklist";
in
{
  options.services.sshf = {
    enable = lib.mkEnableOption "sshf - SSH agent filtering proxy";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The sshf package to use";
    };

    mode = lib.mkOption {
      type = lib.types.enum [
        "whitelist"
        "blacklist"
      ];
      default = "whitelist";
      description = "Filter mode: whitelist allows only matching keys, blacklist blocks matching keys";
    };

    pattern = lib.mkOption {
      type = lib.types.str;
      example = "work-*";
      description = "Glob pattern to match against key comments";
    };

    inputSocket = lib.mkOption {
      type = lib.types.str;
      default = "%t/ssh-agent";
      description = "Path to upstream SSH agent socket (supports systemd specifiers)";
    };

    outputSocket = lib.mkOption {
      type = lib.types.str;
      default = "%t/sshf.sock";
      description = "Path for filtered agent socket (supports systemd specifiers)";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    systemd.user.services.sshf = {
      Unit = {
        Description = "sshf - SSH agent filtering proxy";
        After = [ "ssh-agent.service" ];
      };
      Service = {
        ExecStartPre = "-rm -f ${cfg.outputSocket}";
        ExecStart = "${cfg.package}/bin/sshf ${modeFlag} \"${cfg.pattern}\" ${cfg.inputSocket} ${cfg.outputSocket}";
        Restart = "on-failure";
      };
      Install.WantedBy = [ "default.target" ];
    };
  };
}
