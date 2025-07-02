[![Build Status](https://github.com/pepa65/batlimit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/pepa65/batlimit/actions/workflows/ci.yml)
[![Dependencies](https://deps.rs/repo/github/pepa65/batlimit/status.svg)](https://deps.rs/repo/github/pepa65/batlimit)
[![Docs](https://img.shields.io/badge/Docs-batlimit-blue)](https://docs.rs/crate/batlimit/latest)
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Downloads](https://img.shields.io/crates/d/batlimit.svg)](https://crates.io/crates/batlimit)

# batlimit 0.10.0
**Set battery charge limit on supported laptops on Linux with CLI**

It is now widely acknowledged that the life span of Li-ion batteries is extended by not charging them to the max.
An often recommended battery charge limit is 80.

* License: GPLv3.0
* Authors: github.com/pepa65, github.com/stlenyk
* Repo: https://github.com/pepa65/batlimit
* After: https://github.com/stlenyk/batterrier
* Required:
  - Linux kernel `5.4-rc1` or later (exposing the charge limit variable)
  - `systemd` version `244` or later (supporting the directives in the included systemd service)
* System variables used:
  - In general: `/sys/class/power_supply/BAT?/*`
  - Particularly: `/sys/class/power_supply/BAT?/charge_control_end_threshold`
  - For some models: `/sys/class/power_supply/BAT?/charge_control_start_threshold`
* Systemd unit file names: `/etc/systemd/system/batlimit-TARGET.service`
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
	- `applesmc-next`: Needs to be compiled from https://github.com/c---/applesmc-next

## Features
* Works with supported laptop (`info` works with any machine):
  ASUS, Lenovo (ThinkPad), Dell, LG, Huawei, System76, Fujitsu, MSI, Toshiba and Intel Apple.
* `info`: Show battery info (default).
* `limit`: Set battery charge limit (needs root privileges), takes percentage as argument.
* `clear`: Clear battery charge limit (needs root privileges)
* `persist`: Persist the charge limit through creating and enabling systemd services,
  optionally takes percentage as argument for limit (needs root privileges).
* `unpersist`: Unpersist the charge limit by disabling and removing systemd services (needs root privileges).
* `shell`: Generate shell completions (bash, elvish, fish, powershell, zsh).
* Can use abbreviations for the commands, like: `batlimit u` (unpersisting the limit).

## Installation
### Download static single-binary
```
wget https://github.com/pepa65/batlimit/releases/download/0.10.0/batlimit
sudo mv batlimit /usr/local/bin/
sudo chown root:root /usr/local/bin/batlimit
sudo chmod +x /usr/local/bin/batlimit
```

### Using cargo (rust toolchain)
If not installed yet, install a **Rust toolchain**, see https://www.rust-lang.org/tools/install

### Cargo from crates.io
`cargo install batlimit --target=x86_64-unknown-linux-musl`

#### Cargo from git
`cargo install --git https://github.com/pepa65/batlimit --target=x86_64-unknown-linux-musl`

#### Cargo static build (avoid GLIBC incompatibilities)
```
git clone https://github.com/pepa65/batlimit
cd batlimit
rustup target add x86_64-unknown-linux-musl
export RUSTFLAGS='-C target-feature=+crt-static'
cargo build --release --target=x86_64-unknown-linux-musl
```

For smaller binary size: `upx --best --lzma target/x86_64-unknown-linux-musl/release/batlimit`

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

Install the musl binary: `cargo-binstall batlimit`

(Then `batlimit` will be installed in `~/.cargo/bin/` which will need to be added to `PATH`!)

## Usage
```
batlimit 0.10.0 - Set battery charge limit on supported laptops on Linux with CLI
Usage: batlimit [COMMAND]
Commands:
  info       Print battery info (default command)
  limit      Set battery charge limit: PERCENT (1..99)
  clear      Clear charge limit
  persist    Persist charge limit with systemd: [PERCENT (1..99)]
  unpersist  Unpersist charge limit: disable and remove systemd services
  shell      Generate completions: SHELL (bash|elvish|fish|powershell|zsh)
  readme     Output the readme file from the repo
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Commands can be abbreviated up to their first letter.
Root privileges required for: limit & clear, persist & unpersist
```
