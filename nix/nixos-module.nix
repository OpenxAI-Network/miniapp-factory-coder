{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.miniapp-factory-coder;
  miniapp-factory-coder = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.miniapp-factory-coder = {
      enable = lib.mkEnableOption "Enable the rust app";

      verbosity = lib.mkOption {
        type = lib.types.str;
        default = "warn";
        example = "info";
        description = ''
          The logging verbosity that the app should use.
        '';
      };

      dataDir = lib.mkOption {
        type = lib.types.path;
        default = "/var/lib/miniapp-factory-coder";
        example = "/var/lib/miniapp-factory-coder";
        description = ''
          The main directory to store data.
        '';
      };

      projectsDir = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/projects";
        example = "/var/lib/miniapp-factory-coder/projects";
        description = ''
          The directory to store projects.
        '';
      };

      model = lib.mkOption {
        type = lib.types.str;
        default = "gpt-oss:20b";
        example = "qwen3-coder:30b-a3b-fp16";
        description = ''
          The Ollama-supported LLM to use for code generation. The full list can be found on https://ollama.com/library
        '';
      };

      git = lib.mkOption {
        type = lib.types.package;
        default = pkgs.git;
        example = pkgs.git;
        description = ''
          git equivalent executable to use for project updates.
        '';
      };

      npm = lib.mkOption {
        type = lib.types.package;
        default = pkgs.bun;
        example = pkgs.bun;
        description = ''
          npm equivalent executable to use for project build testing.
        '';
      };

      aider = lib.mkOption {
        type = lib.types.package;
        default = pkgs.aider-chat;
        example = pkgs.aider-chat;
        description = ''
          aider equivalent executable to use for code generation.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.miniapp-factory-coder = { };
    users.users.miniapp-factory-coder = {
      isSystemUser = true;
      group = "miniapp-factory-coder";
    };

    systemd.services.miniapp-factory-coder = {
      description = "Coder server for Miniapp Factory";
      environment = {
        RUST_LOG = cfg.verbosity;
        DATADIR = cfg.dataDir;
        PROJECTSDIR = cfg.projectsDir;
        MODEL = cfg.model;
        GIT = "${cfg.git}/bin/";
        NPM = "${lib.getExe cfg.npm}";
        AIDER = "${cfg.aider}/bin/";
      };
      serviceConfig = {
        ExecStart = "${lib.getExe miniapp-factory-coder}";
        User = "miniapp-factory-coder";
        Group = "miniapp-factory-coder";
        StateDirectory = "miniapp-factory-coder";
        Restart = "on-failure";
      };
    };

    programs.git = {
      enable = true;
      config = {
        user.name = "Mini App Factory";
        user.email = "miniapp-factory@openxai.org";
        github.user = "miniapp-factory";
        hub.protocol = "ssh";
        init.defaultBranch = "main";
        push.autoSetupRemote = true;
        url."git@github.com:".insteadOf = [
          "https://github.com/"
          "github:"
        ];
        core.sshCommand = "${pkgs.openssh}/bin/ssh -o StrictHostKeyChecking=no -i /var/lib/miniapp-factory-coder/.ssh/id_ed25519";
      };
    };

    nixpkgs.config.allowUnfree = true;
    systemd.services.ollama.serviceConfig.DynamicUser = lib.mkForce false;
    systemd.services.ollama.serviceConfig.ProtectHome = lib.mkForce false;
    systemd.services.ollama.serviceConfig.StateDirectory = [ "ollama/models" ];
    services.ollama = {
      enable = true;
      user = "ollama";
      loadModels = [ cfg.model ];
      environmentVariables = {
        OLLAMA_CONTEXT_LENGTH = "32000"; # From https://aider.chat/docs/llms/ollama.html#ollama and https://docs.ollama.com/context-length
      };
    };
    systemd.services.ollama-model-loader.serviceConfig.User = "ollama";
    systemd.services.ollama-model-loader.serviceConfig.Group = "ollama";
    systemd.services.ollama-model-loader.serviceConfig.DynamicUser = lib.mkForce false;
  };
}
