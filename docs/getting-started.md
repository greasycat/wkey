# Getting Started

This guide covers the shortest path from a fresh install to a usable `wkey` setup.

## Install

Install via AUR for archlinux user

```bash
yay -S wkey
```

Build from source:

```bash
cargo build --release
```

The binary will be available at:

```text
target/release/wkey
```

Install into Cargo's binary directory from this repository:

```bash
cargo install --path .
```

## Runtime Requirements

- A terminal with standard ANSI support
- `fzf` is optional
- If `fzf` is installed, `wkey --search` uses it first
- If `fzf` is missing or cannot be launched, `wkey` falls back to an internal selector

## Initialize the Config

Create the default config:

```bash
wkey init --yes
```

This creates:

```text
$XDG_CONFIG_HOME/wkey/
  config.toml
  keyboard.txt
  groups/
    wkey.toml
```

The bundled `wkey.toml` group gives a fresh install something useful to browse immediately.

`wkey init` now creates a default `config.toml` too. Optional clipboard-style note pipeout:

```toml
# $XDG_CONFIG_HOME/wkey/config.toml
[pipeout]
command = "wl-copy"
```

## Add Your First Items

Create a shortcut:

```bash
wkey shortcut create --group shell copy --key Ctrl+C --desc "Copy selection"
```

Create a note:

```bash
wkey note create --group shell prompt-tip --desc "Use !! to repeat the previous command"
```

The same flow using aliases:

```bash
wkey s c -g shell copy -k Ctrl+C -d "Copy selection"
wkey n c -g shell prompt-tip -d "Use !! to repeat the previous command"
```

## Open the Interface

Open the TUI:

```bash
wkey
```

Open the TUI after preselecting an item:

```bash
wkey --search
```

Alias form:

```bash
wkey -s
```

Run only the selector and print the selected item key:

```bash
wkey --search-only
```

## Next Steps

- For config files and layout format, see [Configuration](configuration.md).
- For interactive behavior and controls, see [TUI guide](tui.md).
- For all commands and JSON output, see [CLI reference](cli-reference.md).
