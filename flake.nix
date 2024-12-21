{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        buildTarget = "wasm32-unknown-unknown";
        pkgs = import nixpkgs {
			    inherit system;
          overlays = [ rust-overlay.overlays.default ];
		    };

		    rustToolchain = pkgs.rust-bin.stable.latest.default.override {
			    targets = [ buildTarget ];
          extensions = [ "rust-analyzer" "rust-src" ];
		    };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in rec {
        # Build the wasm file for the online editor 
		    packages.wavedrom-wasm = rustPlatform.buildRustPackage rec {
          name = "wavedrom-wasm";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          buildPhase = ''
            cargo build --release -p ${name} --target=${buildTarget}
          '';

          installPhase = ''
            mkdir -p $out/lib
            cp target/${buildTarget}/release/*.wasm $out/lib/
          '';

          doCheck = false;
		    };

        # Build the all the files for the online editor 
        packages.editor = pkgs.stdenv.mkDerivation {
          pname = "wavedrom-rs-editor";
          version = "0.1.0";

          src = ./.;

          buildPhase = ''
            cd wavedrom-wasm

            mkdir -p $out

            ${pkgs.wabt}/bin/wasm-strip         -o $out/wavedrom.wasm    ${packages.wavedrom-wasm}/lib/wavedrom_wasm.wasm 
            ${pkgs.tailwindcss}/bin/tailwindcss -o $out/index.css     -i index.scss  --minify
            ${pkgs.minify}/bin/minify           -o $out/index.html       index.html
            ${pkgs.minify}/bin/minify           -o $out/index.js         index.js

            cp -r assets                           $out/assets
          '';
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            wabt
            tailwindcss
            minify
            rustToolchain
          ];

          shellHook = ''
            export RUST_SRC_BIN="${rustToolchain}/lib/rustlib/src/rust/library";
          '';
        };
    }
  );
}