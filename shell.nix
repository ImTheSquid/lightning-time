let
    pkgs = import <nixpkgs> {};
    fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
in
pkgs.mkShell {
  buildInputs = [
    (fenix.complete.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
      "rust-analyzer"
    ])
  ];
}
