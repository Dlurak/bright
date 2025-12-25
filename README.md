# Bright

Backlight (and LED) control for linux.

## Features

- Precise control over with device to target
    - `BRIGHT_DEVICE` environment variable
    - `--device` cli flag
- Animations
- Linear **looking** brightness values
- Various values for the brightness
    - Absolute values
    - Percentages
    - Changes (`5%+`, `500-`)
    - `restore`
    - Fancy functions
        - `max(50%, 10%+, 200)`
        - `clamp(1, 5%+, 75%)`
        - Some more
- Saving and restoring the brightness
    - Save the devices brightness before changing it
    - Restore it using `restore`
    - Example use case: Idle-Demons

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

