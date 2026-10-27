{
	inputs = {
		nixpkgs.url = "nixpkgs/nixos-unstable";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils, ... }:
	flake-utils.lib.eachDefaultSystem (system:
		let
			pkgs = import nixpkgs { inherit system; };
			cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
		in {
			packages.default = pkgs.rustPlatform.buildRustPackage {
				pname = cargoToml.package.name;
				version = cargoToml.package.version;
				src = ./.;
				cargoLock.lockFile = ./Cargo.lock;

				nativeBuildInputs = with pkgs; [
					openssl
					pkg-config
				];

				buildInputs = with pkgs; [
					openssl
					pkg-config
				];
			};

			devShells.default = pkgs.mkShell {
				buildInputs = with pkgs; [
					rustc
					cargo

					pkg-config
					openssl

					rust-analyzer
				];

				shellHook = ''
					export HOME="$PWD/.nix-cache"
				'';
			};
		}
	);
}
