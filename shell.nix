{ mkShell, rust-bin, pkgsCross, avrdude, udev, rustfmt, clippy, cargo-bloat }: let
  rust = rust-bin.nightly."2023-01-10".minimal.override {
    extensions = [ "rust-src" ];
  };
in mkShell {
  nativeBuildInputs = [
    rust
    pkgsCross.avr.buildPackages.gcc
    avrdude
    udev
    # Stable mode is basically useless
    (rustfmt.override { asNightly = true; })
    clippy
    cargo-bloat
  ];
}
