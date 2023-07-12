{ mkShell, rust-bin, rustfmt, clippy, cargo-bloat }: let
  rust = rust-bin.nightly."2022-07-29".minimal.override {
    extensions = [ "rust-src" ];
  };
in mkShell {
  nativeBuildInputs = [
    rust
    # Stable mode is basically useless
    (rustfmt.override { asNightly = true; })
    clippy
    cargo-bloat
  ];
}
