{
  description = "cria - a Rust TUI task manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        crateName = "cria";
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = crateName;
          version = "0.9.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ rustToolchain pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          meta = with pkgs.lib; {
            description = "cria - a Rust TUI for Vikunja";
            homepage = "https://github.com/frigidplatypus/cria";
            license = licenses.mit;
            maintainers = with maintainers; [ justin ];
            platforms = platforms.linux ++ platforms.darwin;
          };
        };
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}
