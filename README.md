It was fully vibecoded in 2h

Simple ssh socket filter - takes whitelist or blacklist pattern and socket, returns new socket with identities filtered by comment

You could use it if e.g. you need to share one of your keys to VM or container. Although you shouldn't.

```bash
sshf --whitelist "work-*" $SSH_AUTH_SOCK /tmp/filtered.sock &
SSH_AUTH_SOCK=/tmp/filtered.sock ssh user@host
```

## build

```bash
# nix (static musl binary)
nix build --accept-flake-config

# cargo
cargo build --release
```

## home-manager

```nix
# flake.nix inputs
sshf.url = "github:theoborealis/sshf";
sshf.inputs.nixpkgs.follows = "nixpkgs";
```

```nix
# home.nix - with systemd service
{ inputs, ... }:
{
  imports = [ inputs.sshf.hmModule ];
  services.sshf = {
    enable = true;
    mode = "whitelist";  # or "blacklist"
    pattern = "work-*";
    # inputSocket = "%t/ssh-agent";   # default
    # outputSocket = "%t/sshf.sock";  # default
  };
}
```

```nix
# home.nix - package only
{ inputs, pkgs, ... }:
{
  home.packages = [ inputs.sshf.packages.${pkgs.system}.default ];
}
```
