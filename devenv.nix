{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = with pkgs; [ git ];

  languages.rust = {
    enable = true;
    lsp.enable = true;
    components = [
      "rustfmt"
      "clippy"
      "rust-analyzer"
      "cargo"
      "rustc"
    ];
  };

  git-hooks.hooks = {
    prettier.enable = true;
    nixfmt.enable = true;
    rustfmt.enable = true;
  };
}
