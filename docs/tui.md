# TUI Guide

Run `wkey` with no subcommand to open the interactive interface.

## What the TUI Shows

- An item filter box
- A group filter box
- A list of matching shortcuts or notes
- A detail panel for the current selection
- A keyboard view that highlights keys for shortcut entries

## TUI Controls

- Type to filter items
- `Tab` switches between item filtering and group filtering
- `Up` / `Down` moves selection
- `Ctrl-N` / `Ctrl-P` moves selection
- `PageUp` / `PageDown` jumps by a page
- `Ctrl-U` / `Ctrl-D` jumps by a page
- `Home` / `End` moves to the start or end
- `Backspace` removes characters from the active filter
- `Enter` sends the selected note text to the configured `pipeout` command, if one is set
- `Ctrl-C` quits

## Search Before Opening the TUI

Use:

```bash
wkey --search
```

Or:

```bash
wkey -s
```

Run only the selector:

```bash
wkey --search-only
wkey -S
```

Behavior:

- If `fzf` can be launched, `wkey` uses it for the initial selection step
- `fzf` is launched with reverse layout by default
- If `fzf` is unavailable, `wkey` falls back to an internal selector
- If the selector is cancelled, `wkey` opens the TUI without a preselected item
- `wkey --search-only` prints the selected item key and exits without opening the main TUI

## Related Docs

- For setup and first commands, see [Getting started](getting-started.md).
- For full command coverage, see [CLI reference](cli-reference.md).
