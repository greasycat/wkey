# Configuration

`wkey` stores its data in plain files under an XDG-style config directory.

## Default Config Root

By default, `wkey` uses:

```text
$XDG_CONFIG_HOME/wkey/
  keyboard.txt
  groups/
    wkey.toml       # Example
    shell.toml      # Example
    editor.toml     # Example
    your_group.toml # Example
```

## Use a Custom Config Directory

Use:

```bash
wkey --config-dir /path/to/config
```

Short form:

```bash
wkey -C /path/to/config
```

## `keyboard.txt`

If `keyboard.txt` exists, `wkey` uses it to render the keyboard view.

Example:

```text
Esc 1 2 3 4 5 6 7 8 9 0 - = Backspace
Tab Q W E R T Y U I O P [ ] \
Caps A S D F G H J K L ; ' Enter
Shift Z X C V B N M , . / Shift
Ctrl Alt Space Alt Ctrl
```

Formatting rules:

- Whitespace controls horizontal spacing between keys
- Empty lines are ignored
- Comment lines starting with `#` are ignored

## Group Files

Each file under `groups/*.toml` contains shortcuts and notes for one group.

Example:

```toml
[shortcuts]
copy = { key = "Ctrl+C", desc = "Copy selection" }
palette = { key = "Ctrl+Shift+P", desc = "Open command palette" }

[notes]
prompt-tip = { desc = "Use !! to repeat the previous command" }
```

## Bundled Default Group

`wkey init` creates `groups/wkey.toml`, a built-in group that documents the app's own TUI bindings and CLI usage.

## Related Docs

- For first-run examples, see [Getting started](getting-started.md).
- For command behavior, see [CLI reference](cli-reference.md).
