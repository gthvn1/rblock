{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") {} }:

pkgs.mkShell {
  packages = with pkgs; [
    zig
    zls
  ];

  shellHook = ''
    echo "Zig version:"
    zig version
  '';
}

