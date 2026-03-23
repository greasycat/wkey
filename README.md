# wkey


[![Rust](https://github.com/greasycat/wkey/actions/workflows/rust.yml/badge.svg)](https://github.com/greasycat/wkey/actions/workflows/rust.yml) ![AUR Version](https://img.shields.io/aur/version/wkey?label=AUR)

https://github.com/user-attachments/assets/d5a4c757-491a-4d25-b98f-caf42b173b83

`wkey` is a terminal-first cheatsheet for keyboard shortcuts and working notes.

It combines:

- a TUI for browsing shortcuts and notes
- plain-text configuration files built on TOML
- CLI commands for managing groups, shortcuts, and notes
- JSON output for automation
- optional `fzf` preselection before opening the TUI

`wkey` keeps the workflow simple: your data lives in files, the UI runs in the terminal, and the CLI stays scriptable.

## Installation

Install via script

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/greasycat/wkey/releases/download/v0.1.0/wkey-installer.sh | sh
```

Install via AUR for archlinux user

```bash
yay -S wkey
```

Build from source:

```bash
cargo build --release
```

Install from this repository:

```bash
cargo install --path .
```

Runtime notes:

- a terminal with standard ANSI support is required
- `fzf` is optional but highly recommended.
- if `fzf` is missing, `wkey --search` falls back to an internal selector

## Quick Start

Initialize the default config:

```bash
wkey init --yes
```

Add a shortcut:

```bash
wkey shortcut create --group shell copy --key Ctrl+C --desc "Copy selection"
```

Add a note:

```bash
wkey note create --group shell prompt-tip --desc "Use !! to repeat the previous command"
```

Open the TUI:

```bash
wkey
```

Open the TUI after preselecting an item:

```bash
wkey --search
```

## More Docs

- [Getting started](docs/getting-started.md)
- [Configuration](docs/configuration.md)
- [TUI guide](docs/tui.md)
- [CLI reference](docs/cli-reference.md)
- [Development](docs/development.md)

## License

`wkey` is released under the terms of the [MIT License](LICENSE).
