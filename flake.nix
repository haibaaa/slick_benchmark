{
  description = "SlickBench development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          python3
          python3Packages.numpy
          python3Packages.matplotlib
          python3Packages.pandas
          uv
          gcc
          git
        ];

        shellHook = ''
          export UV_PYTHON=python3
          export UV_NO_MANAGED_PYTHON=1
          export UV_PYTHON_DOWNLOADS=never
        '';
      };
    };
}
