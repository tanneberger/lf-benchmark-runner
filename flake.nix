{
  inputs = { 
    nixpkgs.url = github:NixOS/nixpkgs/nixos-21.11;

    naersk = {
      url = github:nix-community/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils.url = "github:numtide/flake-utils";

  };
  outputs = { self, nixpkgs, naersk, utils, ... }:
  utils.lib.eachDefaultSystem (system: let 
    pkgs = nixpkgs.legacyPackages.${system};
    derivation = pkgs.callPackage ./lf-benchmark-runner.nix {
      naersk = naersk.lib."${system}";
    };
      in rec {
        checks = packages;
        packages.lf-benchmark-runner = derivation;
      }
   );
}

