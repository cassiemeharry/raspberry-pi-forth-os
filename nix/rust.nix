{ sources ? import ./sources.nix
, pkgs ? import sources.nixpkgs {
    overlays = [ (import sources.nixpkgs-mozilla) ];
  }
}:

let
  channel = pkgs.rustChannelOf {
    extensions = [ "rustfmt" "rust-src" "rust-std" ];
    rustToolchain = ../rust-toolchain;
    targets = [ "x64_64-linux" "aarch64-unknown-none" ];
  };
in

builtins.trace (builtins.attrNames channel) channel
