{ pkgs ? import <nixpkgs> {} }:
let
  lib = pkgs.lib;
  includes = with pkgs; [
    xorg.libX11
    xorg.libXrandr
    xorg.libXrender
    xorg.libXext
    xorg.libXft
    xorg.libXcursor
    xorg.libXfixes
    xorg.libXinerama
    xorg.libXi
    libxkbcommon
    libGL
    libGLU
  ];
in
pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.rust-analyzer
    pkgs.pkg-config
    pkgs.openssl
    pkgs.nasm
    pkgs.gnuplot
    pkgs.dav1d.dev
    includes
    pkgs.xorg.libX11.dev
    pkgs.xorg.libXi.dev
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="${lib.makeLibraryPath includes}"
  '';
}
