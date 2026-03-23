# CLI Reference

## Top-Level Commands

- `init` / `i`
- `group` / `g`
- `shortcut` / `s`
- `note` / `n`

## Global Flags

- `--search`, `-s`
- `--search-only`, `-S`
- `--config-dir <PATH>`, `-C <PATH>`
- `--json`, `-j`

Use `wkey --help` or `wkey <command> --help` for generated help with examples.

Selector behavior:

- `wkey --search` opens the selector first, then launches the main TUI with the chosen item preselected
- `wkey --search-only` runs only the selector, prints the selected item's description, and exits
- If `pipeout.command` is configured, `wkey --search-only` also writes that selected description to the pipeout command

## Group Commands

List groups:

```bash
wkey group list
wkey g l
```

Show one group:

```bash
wkey group show shell
wkey g s shell
```

Create a group:

```bash
wkey group create shell
wkey g c shell
```

Rename a group:

```bash
wkey group rename shell terminal
wkey g r shell terminal
```

Delete an empty group:

```bash
wkey group delete terminal
wkey g d terminal
```

Force-delete a non-empty group:

```bash
wkey group force-delete terminal
wkey g D terminal
```

Important behavior:

- `group delete` only removes empty groups
- If the group still contains shortcuts or notes, `wkey` tells you to use `group force-delete` or `g D`
- `group force-delete` removes the whole group file even when it is non-empty

## Shortcut Commands

Create:

```bash
wkey shortcut create --group shell copy --key Ctrl+C --desc "Copy selection"
wkey s c -g shell copy -k Ctrl+C -d "Copy selection"
```

Show:

```bash
wkey shortcut show --group shell copy
wkey s s -g shell copy
```

List:

```bash
wkey shortcut list
wkey shortcut list --all
wkey shortcut list --group shell
wkey s l
wkey s l -a
wkey s l -g shell
```

Update:

```bash
wkey shortcut update --group shell copy --key Cmd+C
wkey shortcut update --group shell copy --desc "Copy in macOS apps"
wkey shortcut update --group shell copy --new-id duplicate
wkey s u -g shell copy -k Cmd+C
```

Delete:

```bash
wkey shortcut delete --group shell copy
wkey s d -g shell copy
```

Move:

```bash
wkey shortcut move --from-group shell --to-group editor copy
wkey s m -f shell -t editor copy
```

Important behavior:

- `shortcut create` auto-creates the target group file if needed
- `shortcut move` auto-creates the destination group if needed
- `shortcut list` defaults to all groups
- `--all` is an explicit alias for the default all-groups behavior
- `--group` and `--all` cannot be used together

## Note Commands

Create:

```bash
wkey note create --group shell prompt-tip --desc "Use !! to repeat the previous command"
wkey n c -g shell prompt-tip -d "Use !! to repeat the previous command"
```

Show:

```bash
wkey note show --group shell prompt-tip
wkey n s -g shell prompt-tip
```

List:

```bash
wkey note list
wkey note list --all
wkey note list --group shell
wkey n l
wkey n l -a
wkey n l -g shell
```

Update:

```bash
wkey note update --group shell prompt-tip --desc "Use Ctrl+R for history search"
wkey note update --group shell prompt-tip --new-id history-tip
wkey n u -g shell prompt-tip -d "Use Ctrl+R for history search"
```

Delete:

```bash
wkey note delete --group shell prompt-tip
wkey n d -g shell prompt-tip
```

Move:

```bash
wkey note move --from-group shell --to-group editor prompt-tip
wkey n m -f shell -t editor prompt-tip
```

Important behavior:

- `note create` auto-creates the target group file if needed
- `note move` auto-creates the destination group if needed
- `note list` defaults to all groups
- `--all` is an explicit alias for the default all-groups behavior
- `--group` and `--all` cannot be used together

## JSON Output

Use `--json` or `-j` with `group`, `shortcut`, and `note` commands:

```bash
wkey --json group list
wkey -j shortcut list --all
wkey -j note show --group shell prompt-tip
```

JSON is supported for non-interactive data commands only. It is not supported for:

- `wkey` TUI launch
- `wkey --search`
- `wkey --search-only`
- `wkey init`

### Output Shapes

`group list --json`

```json
[
  {
    "name": "shell",
    "shortcut_count": 3,
    "note_count": 2
  }
]
```

`group show --json`

```json
{
  "name": "shell",
  "items": [
    {
      "kind": "shortcut",
      "group": "shell",
      "id": "copy",
      "key": "Ctrl+C",
      "desc": "Copy selection"
    },
    {
      "kind": "note",
      "group": "shell",
      "id": "prompt-tip",
      "desc": "Use !! to repeat the previous command"
    }
  ]
}
```

`shortcut` payload:

```json
{
  "kind": "shortcut",
  "group": "shell",
  "id": "copy",
  "key": "Ctrl+C",
  "desc": "Copy selection"
}
```

`note` payload:

```json
{
  "kind": "note",
  "group": "shell",
  "id": "prompt-tip",
  "desc": "Use !! to repeat the previous command"
}
```

## Errors and Validation

`wkey` validates common mistakes and tries to make recovery obvious.

Examples:

- Missing CLI arguments print the original Clap error, then show contextual examples
- Invalid item ids are rejected if they contain whitespace or dots
- Group names cannot contain path separators
- A normal `group delete` on a non-empty group tells you to use `wkey group force-delete <group>` or `wkey g D <group>`
