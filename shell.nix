let
  pkgs = import <nixpkgs> {};
  armv8Pkgs = import <nixpkgs> {
    crossSystem = (import <nixpkgs/lib>).systems.examples.aarch64-embedded;
  };
  armv7Pkgs = import <nixpkgs> {
    crossSystem = (import <nixpkgs/lib>).systems.examples.arm-embedded;
  };
in

pkgs.callPackage (
  {mkShell}:
  mkShell {
    # nativeBuildInputs = [ pkgs.cc ];
    buildInputs = [ armv8Pkgs.stdenv.cc armv7Pkgs.stdenv.cc pkgs.gdb pkgs.qemu ]; # your dependencies here
    shellHook = ''
      export PATH="$PATH:${pkgs.qemu}/bin"
      export RUSTFLAGS="-C link-arg=-nostartfiles"
    '';
  }
) {}
