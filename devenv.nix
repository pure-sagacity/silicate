{
  pkgs,
  ...
}:

{
  profiles = {
    bun.module = {
      packages = [ pkgs.bun ];
    };
  };

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
    shellcheck.enable = true;
    prettier.enable = true;
    nixfmt.enable = true;
    rustfmt.enable = true;
  };
}
