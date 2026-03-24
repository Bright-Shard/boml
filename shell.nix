{
  pkgs ? import <nixpkgs> {}
}:

pkgs.mkShell {
  name = "boml-devshell";
  packages = with pkgs; [ gcc ];
}
