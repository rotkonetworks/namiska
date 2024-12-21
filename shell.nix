{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
    xorg.libX11
    xorg.libXtst
    xorg.libXi
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXfixes
    libinput
    xorg.libXext
    xorg.libxcb
    xorg.xinput
    libxkbcommon
    libevdev
    xdotool
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath ([
      pkgs.xorg.libX11
      pkgs.xorg.libXtst
      pkgs.xorg.libXi
      pkgs.xorg.libXrandr
      pkgs.xorg.libXinerama
      pkgs.xorg.libXcursor
      pkgs.xorg.libXfixes
      pkgs.libinput
      pkgs.xorg.libXext
      pkgs.xorg.libxcb
      pkgs.libxkbcommon
      pkgs.libevdev
      pkgs.xdotool
    ])}
  '';
}
