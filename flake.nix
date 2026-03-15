{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;

      python = pkgs.python312;
    in
    {
      devShells.x86_64-linux = {
        default = pkgs.mkShell {
          packages = [
            python
            pkgs.uv
            pkgs.tectonic
          ];
        };
      };
    };
}
