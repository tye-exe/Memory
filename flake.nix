{
  description = "Environment for the shut_the_box program.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # rust-build = pkgs.rust-bin.nightly.latest.default.override {
        #   # 'targets' is equivalent to 'rustup toolchain add'
        #   targets = [ "riscv64imac-unknown-none-elf" ];
        #   # 'extensions' is equivalent to 'rustup component add'
        #   extensions = [
        #     "rust-src" # Source code for the std, used by rust-analyzer
        #   ];
        # };

        rust-build = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          # targets = [ "arm-unknown-linux-gnueabihf" ];
        };

      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              rust-build
              mermaid-cli # Graph generation
              bacon
            ];

            LD_LIBRARY_PATH = let
              libPath = with pkgs;
                lib.makeLibraryPath [
                  libGL
                  libxkbcommon
                  wayland
                  xorg.libX11
                  xorg.libXcursor
                  xorg.libXi
                  xorg.libXrandr
                ];
            in libPath;
          };
      });
}

