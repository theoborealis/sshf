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

# home.nix
home.packages = [ inputs.sshf.packages.${pkgs.system}.default ];
```
