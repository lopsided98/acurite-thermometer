{
  description = "Acurite Thermometer Firmware";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: let
    systems = [ "x86_64-linux" ];
  in flake-utils.lib.eachSystem systems (system: let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ (import rust-overlay) ];
    };
  in {
    devShells.default = pkgs.callPackage ./shell.nix { };
  });
}
