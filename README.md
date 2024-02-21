# toki

## Build config

### linux 

#### fedora

```sh
sudo dnf install gtk4-devel gcc
```

#### debian and derivatives:

```sh
sudo apt install libgtk-4-dev build-essential
```

#### arch and derivatives:

```sh
sudo pacman -S gtk4 base-devel
```

### macos

```sh
brew install gtk4 pkg-config

pkg-config --modversion gtk4
```
