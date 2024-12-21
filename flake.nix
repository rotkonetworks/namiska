{
  description = "keyboard mouse - move mouse cursor using keyboard shortcuts(meta+arrows)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
      packages.default = pkgs.rustPlatform.buildRustPackage {
      pname = "namiska";
      version = "0.1.2";
      src = ./.;
      cargoLock = {
      lockFile = ./Cargo.lock;
      };
      nativeBuildInputs = [ pkgs.pkg-config ];
      buildInputs = with pkgs; [
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
      ];
      };

      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          rust-bin.stable.latest.default
            pkg-config
        ];
      };
      }
  );
}
