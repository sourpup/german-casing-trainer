let
  sources = import ./lon.nix;
  pkgs = import sources.nixpkgs { };
  pkgs-wasm-bindgen = import sources.nixpkgs-wasm-bindgen {};
in
pkgs.mkShell {
  packages = [
    pkgs.nixfmt
    pkgs.nix-prefetch-git
    pkgs.clippy
    pkgs.rustfmt
    pkgs.rust-analyzer
    pkgs.leptosfmt
    pkgs.trunk
    pkgs.dart-sass
    pkgs.rustc
    pkgs.lld
    pkgs.cargo-leptos
    pkgs-wasm-bindgen.wasm-bindgen-cli_0_2_127
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
