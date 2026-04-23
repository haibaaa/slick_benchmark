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
        # The shell provides both the Python scientific stack used by the
        # plotting scripts and the C++ runtime needed by native Python wheels.
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
          # Prefer the Nix-provided interpreter so `uv run` reuses the shell
          # environment instead of creating an incompatible managed runtime.
          export UV_PYTHON=python3
          export UV_NO_MANAGED_PYTHON=1
          export UV_PYTHON_DOWNLOADS=never
        '';
      };
    };
}
