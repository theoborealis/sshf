{
  description = "sshf - Simple SSH agent filtering proxy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      crane,
      ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      hmModule =
        { pkgs, lib, ... }:
        {
          imports = [ ./hm-module.nix ];
          services.sshf.package = lib.mkDefault self.packages.${pkgs.system}.default;
        };

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          archConfig =
            {
              "x86_64-linux" = {
                target = "x86_64-unknown-linux-musl";
                pkgsMusl = pkgs.pkgsCross.musl64;
              };
              "aarch64-linux" = {
                target = "aarch64-unknown-linux-musl";
                pkgsMusl = pkgs.pkgsCross.aarch64-multiplatform-musl;
              };
            }
            .${system};

          inherit (archConfig) target pkgsMusl;

          toolchain =
            with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              targets.${target}.stable.rust-std
            ];

          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
            doCheck = false;
            CARGO_BUILD_TARGET = target;
            TARGET_CC = "${pkgsMusl.stdenv.cc}/bin/${pkgsMusl.stdenv.cc.targetPrefix}cc";
            # -B tells gcc where to find ld.mold
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C linker=${pkgsMusl.stdenv.cc}/bin/${pkgsMusl.stdenv.cc.targetPrefix}cc -C link-arg=-fuse-ld=mold -C link-arg=-B${pkgs.mold}/bin";
            nativeBuildInputs = [
              pkgs.mold
              pkgsMusl.stdenv.cc
            ];
            HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
          };

          # Deps cached separately - faster rebuilds on src changes
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          default = self.packages.${system}.sshf;
          sshf = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        }
      );

    };
}
