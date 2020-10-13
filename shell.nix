let
  # localOverlay = self: super: {
  #   qemu = super.qemu // {
  #     patches = builtins.trace "Adding patch ./qemu.patch" (super.qemu.patches ++ [ ./qemu.patch ]);
  #   };
  # };
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {
    overlays = [ (import sources.nixpkgs-mozilla) ];
  };
  rust = import ./nix/rust.nix { inherit sources pkgs; };
  armv8Pkgs = import <nixpkgs> {
    crossSystem = (import <nixpkgs/lib>).systems.examples.aarch64-embedded;
  };
  armv7Pkgs = import <nixpkgs> {
    crossSystem = (import <nixpkgs/lib>).systems.examples.arm-embedded;
  };
  qemu_patched = pkgs.qemu.overrideAttrs (attrs: {
    configureFlags = attrs.configureFlags ++ [
      "--target-list=aarch64-softmmu"
      "--disable-gnutls"
      "--disable-nettle"
      "--disable-gcrypt"
      "--disable-auth-pam"
      "--disable-curses"
      "--disable-iconv"
      "--disable-vnc"
      "--disable-virtfs"
      "--disable-mpath"
      "--disable-xen"
      "--disable-brlapi"
      "--disable-curl"
      "--disable-bluez"
      "--disable-kvm" #             KVM acceleration support
      "--disable-hax" #             HAX acceleration support
      "--disable-hvf" #             Hypervisor.framework acceleration support
      "--disable-whpx" #            Windows Hypervisor Platform acceleration support
      "--disable-rdma" #            Enable RDMA-based migration
      "--disable-pvrdma" #          Enable PVRDMA support
      "--disable-vde" #             support for vde network
      "--disable-netmap" #          support for netmap network
      "--disable-linux-aio" #   Linux AIO support
      "--disable-cap-ng" #   libcap-ng support
      "--disable-attr" #   attr and xattr support
      "--disable-vhost-net" #   vhost-net kernel acceleration support
      "--disable-vhost-vsock" #   virtio sockets device support
      "--disable-vhost-scsi" #   vhost-scsi kernel target support
      "--disable-vhost-crypto" #   vhost-user-crypto backend support
      "--disable-vhost-kernel" #   vhost kernel backend support
      "--disable-vhost-user" #   vhost-user backend support
      "--disable-spice" #   spice
      "--disable-rbd" #   rados block device (rbd)
      "--disable-libiscsi" #   iscsi support
      "--disable-libnfs" #   nfs support
      "--disable-smartcard" #   smartcard support (libcacard)
      "--disable-libusb" #   libusb (for usb passthrough)
      "--disable-live-block-migration" #   Block migration in the main migration stream
      "--disable-usb-redir" #   usb network redirection support
      "--disable-lzo" #   support of lzo compression library
      "--disable-snappy" #   support of snappy compression library
      "--disable-bzip2" #   support of bzip2 compression library
      "--disable-lzfse" #   support of lzfse compression library
      "--disable-seccomp" #   seccomp support
      "--disable-coroutine-pool" #   coroutine freelist (better performance)
      "--disable-glusterfs" #   GlusterFS backend
      "--disable-tpm" #   TPM support
      "--disable-libssh" #   ssh block device support
      "--disable-numa" #   libnuma support
      "--disable-libxml2" #   for Parallels image format
      "--disable-replication" #   replication support
      "--disable-opengl" #   opengl support
      "--disable-virglrenderer" #   virgl rendering support
      "--disable-xfsctl" #   xfsctl support
      "--disable-qom-cast-debug" #   cast debugging support
      "--disable-tools" #   build qemu-io, qemu-nbd and qemu-img tools
      "--disable-vxhs" #   Veritas HyperScale vDisk backend support
      "--disable-bochs" #   bochs image format support
    ];
    patches = attrs.patches ++ [ ./local-qemu-mmu-logs.patch ];
  });
  qemu_orig = pkgs.qemu;

  qemu = qemu_orig;

  # rust-host = (pkgs.rustChannelOf {
  #   extensions = [ "rust-src" ];
  #   rustToolchain = ./rust-toolchain;
  #   targets = [ "*" "x86_64-linux" ];
  # }).rust;
  # rust-arm = (pkgs.rustChannelOf {
  #   extensions = [ "rust-src" ];
  #   rustToolchain = ./rust-toolchain;
  #   targets = [ "*" "aarch64-unknown-none" ];
  # }).rust;
  cargo-xbuild = pkgs.stdenv.mkDerivation {
    name = "cargo-xbuild";
    version = "0.5.28";
    buildInputs = [ rust.rust ];
    rust = rust.rust;
    builder = builtins.toFile "builder.sh" ''
      source $stdenv/setup

      echo "$PATH"
      set -x
      set -euo pipefail
      CARGO_HOME="$out" $rust/bin/cargo install cargo-xbuild==0.5.28
    '';
  };
in

pkgs.callPackage (
  {mkShell}:
  mkShell {
    # nativeBuildInputs = [ pkgs.cc ];
    buildInputs = [
      armv8Pkgs.stdenv.cc
      armv7Pkgs.stdenv.cc
      # cargo-xbuild
      pkgs.stdenv.cc
      # pkgs.stdenv.glibc
      pkgs.file
      pkgs.gdb
      # pkgs.gtk3-x11
      pkgs.mtools
      pkgs.parted
      pkgs.patdiff
      pkgs.python3
      pkgs.xorg.xset
      qemu
      # rust.rust
      # rust.rust-src
    ]; # your dependencies here
    shellHook = ''
      export PATH="$PATH:${qemu}/bin"
      export RUSTFLAGS="-C link-arg=-nostartfiles"
    '';
    # export XARGO_RUST_SRC="/home/cassie/projects/raspberry-pi-forth-os/rust-src/src"
    # PATH="$PATH:${qemu}/bin"
    # RUSTFLAGS="-C link-arg=-nostartfiles";
  }
) {}
