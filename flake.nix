{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Rust 1.95.0 with rust-lld from rust-overlay, so `./build.sh` gives the same bytes as CI.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          (pkgs.rust-bin.stable."1.95.0".default.override {
            targets = [ "wasm32-unknown-unknown" ];
          })
          # The real-git tests run git; the other VCS rows are tested on documented output.
          pkgs.git
        ];
      };
    };
}
