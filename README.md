# Bright

Backlight (and LED) control for linux.

## Installation

Install from source using cargo by cloning this repository and then running `cargo install --path ./bright`. Also copy `90-backlight.rules` into `/etc/udev/rules.d`.

### Nix

If you use Nix you can use this flake.
For NixOS users I recommend to import the module provided by this flake and enable it using

```nix
programs.bright.enable = true;
```

To see what the flake provides you can use this command:

```sh
nix flake show github:dlurak/bright
```

