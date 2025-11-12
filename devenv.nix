{ pkgs, lib, config, inputs, ... }:

{

  packages = with pkgs; [ openssl ];

  languages.rust.enable = true;

  scripts.hello.exec = ''
  '';

  enterShell = ''
  '';
  enterTest = ''
  '';

}
