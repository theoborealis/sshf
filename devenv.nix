{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  languages.rust = {
    enable = true;
    mold.enable = true;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
    nixfmt-rfc-style.enable = true;
  };
}
