{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
        
        buildInputs = with pkgs; [ ];

        nativeBuildInputs = with pkgs; [ ];
      in
      rec {
        defaultPackage = packages.isopodcli;
        packages =
          {
            isopodcli = naersk'.buildPackage {
              src = ./.;
              nativeBuildInputs = nativeBuildInputs;
              buildInputs = buildInputs;
            };
            container = pkgs.dockerTools.buildImage
              {
                name = "isopodcli";
                config = {
                  entrypoint = [ "${packages.isopodcli}/bin/isopodcli" ];
                };
              };
          };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
            [
              nixfmt
              cmake
              rustc
              rustfmt
              cargo
              clippy
              rust-analyzer 
            ] ++ buildInputs ++ nativeBuildInputs;
        };
      }
    );
}