{
  description = "Hello world Rust program statically linked against musl";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs, }:
    let
      # Should work with other targets, but not tested.
      supportedSystems = [ "x86_64-linux" ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system}.pkgsStatic;
          pkgs-full = nixpkgsFor.${system};
        in rec {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "h8r";
            version = "1.1.0";

            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
          deb = pkgs.stdenv.mkDerivation {
            name = "h8r";
            phases = [ "installPhase" ];
            installPhase = ''
              mkdir -p ./dpkg/usr/bin
              mkdir -p $out
              cp ${default}/bin/h8r ./dpkg/usr/bin

              mkdir -p ./dpkg/DEBIAN
              cat > ./dpkg/DEBIAN/control <<EOF
Package: h8r
Version: ${default.version}
Architecture: amd64
Maintainer: shebpamm
Description: h8r
EOF
              ${pkgs-full.dpkg}/bin/dpkg-deb --build ./dpkg
              mv ./dpkg.deb $out/h8r.deb
            '';
          };
        });

      devShells = forAllSystems (system:
        let pkgs = nixpkgsFor.${system};
        in {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
            ];
          };
        });
    };
}
