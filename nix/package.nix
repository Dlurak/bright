{ pkgs }:
let
  manifest = pkgs.lib.importTOML ../Cargo.toml;
in
pkgs.rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;

  src = pkgs.lib.cleanSource ./..;
  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = "A cli to controll the brightness";
    homepage = "https://github.com/dlurak/bright";
    mainProgram = "jiman";
    # license = lib.licenses.eupl12;
    # maintainers = with lib.maintainers; [
    #   dlurak
    # ];
  };
}
