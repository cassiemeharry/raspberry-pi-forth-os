{ sources ? import ./nix/sources.nix
, pkgs ? import sources.nixpkgs {}
}:

let
  rust = import ./nix/rust.nix { inherit sources pkgs; };
  naersk = pkgs.callPackage sources.naersk {
    rustc = rust;
    cargo = rust;
  };

  src = builtins.filterSource
    (path: type: type != "directory" || (builtins.all (folder: folder != builtins.baseNameOf path) ["build" "target"]))
    ./.;

  cargoBuild = old:
    ''cargo install cargo-xbuild && cargo $cargo_options xbuild $cargo_build_options >> $cargo_build_output_json'';
    # builtins.trace "cargoBuild.new:"
    # (builtins.concatStringsSep "xbuild"
    #   (builtins.split "\bbuild\b" (builtins.trace "cargoBuild.old:" old)));
in
builtins.trace src
naersk.buildPackage {
  inherit cargoBuild src;
  remapPathPrefix = true;
}
