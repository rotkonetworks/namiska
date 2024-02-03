# namiska

Namiska allows you to control your mouse cursor using keyboard shortcuts.
It's designed to enhance productivity for users who lack thinkpad and
prefer keyboard navigation over traditional mouse or touchpad interactions.

## Usage

Use the Super/Meta/Windows key as a modifier key along with the arrow keys
to move the mouse cursor around your screen.

- **Meta + RCtrl**: Perform a left-click.
- **Meta + RShift**: Perform a right-click.


## Requirements

Namiska works on Linux, macOS, and potentially on Windows with some modifications.

### Linux

For X11 users, `libxdo-dev` or `xdotool` might be required.

- **Debian/Ubuntu**:
  ```sh
  sudo apt-get install libxdo-dev
  ```
- **Arch Linux**:
  ```sh
  sudo pacman -S xdotool
  ```

### Windows

Currently, there's no direct support or installation method provided for Windows users. Stay tuned for future updates.

### macOS

Namiska should work on macOS, leveraging accessibility features to control the mouse. Ensure you have permissions set correctly for terminal access to control your computer.

## Installation

### From Source

Clone the repository and build from source using Cargo, Rust's package manager and build system.

```sh
git clone https://github.com/rotkonetworks/namiska
cd namiska
cargo build --release
cp target/release/namiska ~/.local/bin/
```

### From Binary

Download the latest binary release directly and make it executable:

```sh
curl -s https://api.github.com/repos/rotkonetworks/namiska/releases/latest | grep "browser_download_url.*namiska" | cut -d '"' -f 4 | wget -i - -O namiska
chmod +x namiska
mv namiska ~/.local/bin/
```

### Run

To start Namiska, simply run it from the terminal. To stop it, you can use `pkill`.

```sh
# Run Namiska
namiska &

# To stop Namiska
pkill namiska
```

### Install as a systemd service

For continuous background operation, you can install Namiska as a systemd user service.

```sh
mkdir -p ~/.config/systemd/user
cp namiska.service ~/.config/systemd/user/
systemctl --user enable --now namiska
```

## Customizing Keybindings

Its a bit painful to customize keybindings due to rust being staticly typed.
I recommend you fork the code and build with changed const values for keybindings.

## Customizing sensitivity
Adding config.toml into config directory will alllow you to dynamically
change the sensitivity of the mouse movement.

```sh
```sh
mkdir -p ~/.config/namiska
cp config.toml ~/.config/namiska/
```

## License

Namiska is open-sourced under the MIT License. See the LICENSE file for more details.

© 2024 Rotko Networks
