let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> { overlays = [ (import rustOverlay) ]; };
in

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ pkg-config ];
  buildInputs       = with pkgs; [ openssl ];

  packages = with pkgs; [
    rust-bin.stable.latest.default           # rustc + cargo
    rust-bin.stable.latest.rust-analyzer
    rust-bin.stable.latest.rust-src
    cargo-tarpaulin
    cargo-info
  ];

  RUST_SRC_PATH   = "${pkgs.rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

  shellHook = ''
    echo "Rust dev shell ready"
    echo " rustc $(rustc --version)"
    echo " rust-analyzer $(rust-analyzer --version)"
  '';
}
