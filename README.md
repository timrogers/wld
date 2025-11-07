# `wld`

💡 Control [WLED](https://kno.wled.ge/) lights from the command line

---

## Features

With this tool, you can:

- Save and manage multiple WLED devices by name
- Set a default device for quick access
- Turn your WLED device on and off from the terminal
- Set the brightness level of your WLED device (0-255)
- Control devices by saved name, or directly by IP address

## Installation

### macOS or Linux via [Homebrew](https://brew.sh/)

1. Install the latest version by running `brew tap timrogers/tap && brew install wld`.
1. Run `wld --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via [Cargo](https://doc.rust-lang.org/cargo/), Rust's package manager

1. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, if it isn't already installed.
1. Install the `wld` crate by running `cargo install wld`.
1. Run `wld --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via direct binary download

1. Download the [latest release](https://github.com/timrogers/wld/releases/latest) for your platform. macOS, Linux and Windows devices are supported.
2. Add the binary to `$PATH`, so you can execute it from your shell. For the best experience, call it `wld` on macOS and Linux, and `wld.exe` on Windows.
3. Run `wld --help` to check that everything is working and see the available commands.

## Usage

### From the command line

The `wld` CLI provides the following commands:

#### Device Management

- `wld add <name> <ip>`: Add a new WLED device with a friendly name. The first device added automatically becomes your default.
  ```bash
  wld add desk-light 192.168.1.100
  ```

- `wld delete <name>`: Remove a saved device from your configuration.
  ```bash
  wld delete desk-light
  ```

- `wld ls`: List all saved devices. The default device is marked with `(default)`.
  ```bash
  wld ls
  ```

- `wld set-default <name>`: Set a device as the default for commands that don't specify a device.
  ```bash
  wld set-default desk-light
  ```

#### Device Control

- `wld on`: Turn on your default device, or specify a device with `--device`/`-d`.
  ```bash
  wld on                      # Turn on default device
  wld on -d desk-light        # Turn on a specific saved device
  wld on -d 192.168.1.100     # Turn on a device by IP address
  ```

- `wld off`: Turn off your default device, or specify a device with `--device`/`-d`.
  ```bash
  wld off                     # Turn off default device
  wld off -d desk-light       # Turn off a specific saved device
  wld off -d 192.168.1.100    # Turn off a device by IP address
  ```

- `wld brightness <value>`: Set the brightness of your default device, or specify a device with `--device`/`-d`. Brightness value must be between 0 and 255.
  ```bash
  wld brightness 128          # Set default device to half brightness
  wld brightness 255 -d desk-light  # Set a specific saved device to full brightness
  wld brightness 0 -d 192.168.1.100  # Set a device to minimum brightness
  ```
