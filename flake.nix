{
  description = "pathbin - Inspect executable binaries from PATH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable."1.85.0".default.override {
          extensions = [ "clippy" "rustfmt" ];
          targets = [ "x86_64-pc-windows-gnu" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.rust-analyzer
            pkgs.git
            pkgs.gh
          ];

          shellHook = ''
            echo "pathbin Rust development environment"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
          '';
        };
      }
    );
}
