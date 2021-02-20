{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell {
  buildInputs = [
    postgresql
    openssl.dev
    pkg-config
    rustup
  ];

  shellHook = ''
    export DATABASE_URL=postgres://baraddur:baraddur@localhost/baraddur
  '';
}
