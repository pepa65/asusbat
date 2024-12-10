[![Build Status](https://github.com/pepa65/asusbat/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/pepa65/asusbat/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/pepa65/asusbat/status.svg)](https://deps.rs/repo/github/pepa65/asusbat)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![downloads](https://img.shields.io/crates/d/asusbat.svg)](https://crates.io/crates/asusbat)

# asusbat 0.4.1
**Set battery charge limit on supported laptops on Linux with CLI**

It is now widely acknowledged that the life span of Li-ion batteries is extended by not charging them to the max.
An often recommended battery charge limit is 80.

* License: GPLv3.0
* Authors: github.com/pepa65, github.com/stlenyk
* Repo: https:/github.com/pepa65/asusbat
* After: https://github.com/stlenyk/batterrier
* Required:
  - Linux kernel `5.4-rc1` or later (exposing the charge limit variable)
  - `systemd` version `244` or later (supporting the directives in the included systemd service)
* System variables used: `/sys/class/power_supply/BAT?/*`
* Systemd unit file names: `/etc/systemd/system/asusbat-TARGET.service`
* Systemd targets: `hibernate`, `hybrid-sleep`, `multi-user`, `sleep`, `suspend`, `suspend-then-hibernate`
* Kernel modules and code that supports the charge limit setting:
  - `asus-wmi`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/asus-wmi.c
  - `thinkpad_acpi`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/thinkpad_acpi.c
  - `dell-laptop`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/dell/dell-laptop.c
  - `lg-laptop`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/lg-laptop.c
  - `huawei-wmi`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/huawei-wmi.c
  - `system76_acpi`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/system76_acpi.c
  - `fujitsu-laptop`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/fujitsu-laptop.c
  - `msi-ec`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/msi-ec.c
  - `toshiba_acpi`: https://github.com/torvalds/linux/blob/master/drivers/platform/x86/toshiba_acpi.c

## Features
* Works with supported laptop (`info` works with any machine):
  ASUS, Lenovo (ThinkPad), Dell, LG, Huawei, System76, Fujitsu, MSI en Toshiba.
* `info`: Show battery info (default).
* `limit`: Set battery charge limit (needs root privileges), takes percentage as argument.
* `persist`: Persist the charge limit through creating and enabling systemd services,
  optionally takes percentage as argument for limit (needs root privileges).
* `unpersist`: Unpersist the charge limit by disabling and removing systemd services (needs root privileges).
* `completions`: Generate shell completions (bash, elvish, fish, powershell, zsh).
* Can use abbreviations for the commands, like: `asusbat u` (unpersisting the limit).

## Installation
### Download static single-binary
```
wget https://github.com/pepa65/asusbat/releases/download/0.4.1/asusbat
sudo mv asusbat /usr/local/bin/
sudo chown root:root /usr/local/bin/asusbat
sudo chmod +x /usr/local/bin/asusbat
```

### Using cargo (rust toolchain)
If not installed yet, install a **Rust toolchain**, see https://www.rust-lang.org/tools/install

### Cargo from crates.io
`cargo install asusbat --target=x86_64-unknown-linux-musl`

#### Cargo from git
`cargo install --git https://github.com/pepa65/asusbat --target=x86_64-unknown-linux-musl`

#### Cargo static build (avoid GLIBC incompatibilities)
```
git clone https://github.com/pepa65/asusbat
cd asusbat
rustup target add x86_64-unknown-linux-musl
export RUSTFLAGS='-C target-feature=+crt-static'
cargo build --release --target=x86_64-unknown-linux-musl
```

For smaller binary size: `upx --best --lzma target/x86_64-unknown-linux-musl/release/asusbat`

## Install with cargo-binstall
Even without a full Rust toolchain, rust binaries can be installed with the static binary `cargo-binstall`:

```
# Install cargo-binstall for Linux x86_64
# (Other versions are available at https://crates.io/crates/cargo-binstall)
wget github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
tar xf cargo-binstall-x86_64-unknown-linux-musl.tgz
sudo chown root:root cargo-binstall
sudo mv cargo-binstall /usr/local/bin/
```

Install the musl binary: `cargo-binstall asusbat`

(Then `asusbat` will be installed in `~/.cargo/bin/` which will need to be added to `PATH`!)

## Usage
```
asusbat 0.4.1 - Set battery charge limit on supported laptops on Linux with CLI
Usage: asusbat [COMMAND]
Commands:
  info         Print battery info (default command)
  limit        Set battery charge limit: PERCENT (1..100)
  persist      Persist charge limit with systemd: [PERCENT (1..100)]
  unpersist    Unpersist charge limit: disable and remove systemd services
  completions  Generate completions: SHELL (bash|elvish|fish|powershell|zsh)
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Root privileges required for: limit, persist & unpersist
```
