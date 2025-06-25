{
  pkgs ? import <nixpkgs> { },
}:
let
  libs = [
    pkgs.libGL
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.libxkbcommon
  ];
in
pkgs.mkShell {
  buildInputs = [ pkgs.rustup ] ++ libs;
  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath libs}"
  '';
}
