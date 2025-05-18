{pkgs}: let
  manifest = pkgs.lib.importTOML ../Cargo.toml;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.package.name;
    version = manifest.package.version;

    src = pkgs.lib.cleanSource ./..;
    cargoLock.lockFile = ../Cargo.lock;

    # Just as the udev file itself the two phases are copied from spikespaz/slight <3
    postPatch = with pkgs; ''
      substituteInPlace 90-backlight.rules \
        --replace '/bin/chgrp' '${coreutils}/bin/chgrp' \
        --replace '/bin/chmod' '${coreutils}/bin/chmod'
    '';

    postInstall = ''
      # install -Dm444 90-backlight.rules -t $out/etc/udev/rules.d
      install -Dm444 90-backlight.rules -t $out/lib/udev/rules.d
    '';

    meta = {
      description = "A cli to controll the brightness";
      homepage = "https://github.com/dlurak/bright";
      mainProgram = "bright";
    };
  }
