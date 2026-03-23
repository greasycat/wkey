use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

const TOP_LEVEL_EXAMPLES: &str = "\
Examples:
  wkey
  wkey --search
  wkey -s
  wkey --search-only
  wkey -S
  wkey init --yes
  wkey i -y
  wkey group list
  wkey g l
  wkey g D shell
  wkey shortcut list --all
  wkey s l -a
  wkey shortcut list --json
  wkey -j s l
  wkey note create --group shell prompt-tip --desc \"Use !! to repeat the previous command\"";

const INIT_EXAMPLES: &str = "\
Examples:
  wkey init
  wkey init --yes
  wkey i -y";

const GROUP_EXAMPLES: &str = "\
Examples:
  wkey group list
  wkey g l
  wkey group show shell
  wkey group create shell
  wkey group rename shell terminal
  wkey group delete terminal
  wkey group force-delete shell
  wkey g D shell
  wkey group list --json";

const SHORTCUT_EXAMPLES: &str = "\
Examples:
  wkey shortcut create --group shell copy --key Ctrl+C --desc \"Copy selection\"
  wkey s c -g shell copy -k Ctrl+C -d \"Copy selection\"
  wkey shortcut show --group shell copy
  wkey shortcut list --group shell
  wkey shortcut list --all
  wkey shortcut move --from-group shell --to-group editor copy
  wkey shortcut update --group shell copy --key Cmd+C
  wkey shortcut list --json";

const NOTE_EXAMPLES: &str = "\
Examples:
  wkey note create --group shell prompt-tip --desc \"Use !! to repeat the previous command\"
  wkey n c -g shell prompt-tip -d \"Use !! to repeat the previous command\"
  wkey note show --group shell prompt-tip
  wkey note list --group shell
  wkey note list --all
  wkey note move --from-group shell --to-group editor prompt-tip
  wkey note update --group shell prompt-tip --desc \"Use Ctrl+R for history search\"
  wkey note list --json";

#[derive(Debug, Parser)]
#[command(
    name = "wkey",
    about = "Interactive terminal cheatsheet for keyboard shortcuts and notes.",
    long_about = "wkey opens an interactive keyboard cheatsheet from TOML files loaded from the XDG config directory.\n\nDefault config layout:\n  config.toml\n  keyboard.txt\n  groups/<group>.toml\n\nRun without subcommands to open the TUI. Use `--search` to preselect an item through fzf when available, or through the built-in selector fallback when fzf cannot be launched. Use `--search-only` to run only the selector and print the selected item's description. Use `init` to bootstrap the config directory.",
    after_long_help = TOP_LEVEL_EXAMPLES
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(
        short = 's',
        long,
        help = "Preselect an item before opening the TUI",
        long_help = "Open an item selector before rendering the TUI. Uses `fzf` when it can be launched, and falls back to the built-in selector when `fzf` is unavailable."
    )]
    pub search: bool,

    #[arg(
        short = 'S',
        long = "search-only",
        conflicts_with = "search",
        help = "Run only the selector and print the selected item's description",
        long_help = "Run the item selector without opening the main TUI. Uses `fzf` when it can be launched, and falls back to the built-in selector when `fzf` is unavailable. Prints the selected item's description to stdout and exits."
    )]
    pub search_only: bool,

    #[arg(
        short = 'C',
        long,
        global = true,
        value_name = "PATH",
        help = "Override the config directory instead of using XDG",
        long_help = "Override the config directory instead of using the XDG config location. When set, `wkey` reads and writes `config.toml`, `keyboard.txt`, and `groups/*.toml` under this directory."
    )]
    pub config_dir: Option<PathBuf>,

    #[arg(
        short = 'j',
        long,
        global = true,
        help = "Emit JSON for data commands",
        long_help = "Emit machine-readable JSON for non-interactive data commands. Supported by `group`, `shortcut`, and `note` subcommands, but not by `init` or TUI launch."
    )]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        visible_alias = "i",
        about = "Create the default app config, keyboard layout, and bundled wkey help group",
        long_about = "Bootstrap the config directory by writing `config.toml`, `keyboard.txt`, and `groups/wkey.toml`. Existing files are preserved unless `--yes` is provided."
    )]
    Init(InitArgs),
    #[command(
        visible_alias = "g",
        about = "Inspect and manage group files",
        after_long_help = GROUP_EXAMPLES
    )]
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    #[command(
        visible_alias = "s",
        about = "Inspect and manage shortcut entries",
        after_long_help = SHORTCUT_EXAMPLES
    )]
    Shortcut {
        #[command(subcommand)]
        command: ShortcutCommand,
    },
    #[command(
        visible_alias = "n",
        about = "Inspect and manage note entries",
        after_long_help = NOTE_EXAMPLES
    )]
    Note {
        #[command(subcommand)]
        command: NoteCommand,
    },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(
        short = 'y',
        long,
        help = "Overwrite existing config.toml, keyboard.txt, and groups/wkey.toml without prompting"
    )]
    pub yes: bool,
}

#[derive(Debug, Subcommand)]
pub enum GroupCommand {
    #[command(visible_alias = "l", about = "List configured groups")]
    List,
    #[command(
        visible_alias = "s",
        about = "Show every shortcut and note in one group"
    )]
    Show(GroupNameArgs),
    #[command(visible_alias = "c", about = "Create an empty group file")]
    Create(GroupNameArgs),
    #[command(visible_alias = "r", about = "Rename a group file")]
    Rename(GroupRenameArgs),
    #[command(visible_alias = "d", about = "Delete an empty group file")]
    Delete(GroupNameArgs),
    #[command(
        visible_alias = "D",
        about = "Delete a group file even when it still contains shortcuts or notes"
    )]
    ForceDelete(GroupNameArgs),
}

#[derive(Debug, Subcommand)]
pub enum ShortcutCommand {
    #[command(
        visible_alias = "l",
        about = "List shortcuts in one group or across all groups",
        long_about = "List shortcut entries. By default this includes every group. Use `--group <group>` to scope the listing, or `--all` as an explicit alias for the default all-groups behavior."
    )]
    List(ItemListArgs),
    #[command(visible_alias = "s", about = "Show one shortcut by id within a group")]
    Show(GroupItemArgs),
    #[command(
        visible_alias = "c",
        about = "Create a shortcut entry, auto-creating the group file when needed"
    )]
    Create(CreateShortcutArgs),
    #[command(visible_alias = "u", about = "Update a shortcut entry in place")]
    Update(UpdateShortcutArgs),
    #[command(visible_alias = "d", about = "Delete a shortcut entry")]
    Delete(GroupItemArgs),
    #[command(
        visible_alias = "m",
        about = "Move a shortcut entry to another group, auto-creating the destination"
    )]
    Move(MoveItemArgs),
}

#[derive(Debug, Subcommand)]
pub enum NoteCommand {
    #[command(
        visible_alias = "l",
        about = "List notes in one group or across all groups",
        long_about = "List note entries. By default this includes every group. Use `--group <group>` to scope the listing, or `--all` as an explicit alias for the default all-groups behavior."
    )]
    List(ItemListArgs),
    #[command(visible_alias = "s", about = "Show one note by id within a group")]
    Show(GroupItemArgs),
    #[command(
        visible_alias = "c",
        about = "Create a note entry, auto-creating the group file when needed"
    )]
    Create(CreateNoteArgs),
    #[command(visible_alias = "u", about = "Update a note entry in place")]
    Update(UpdateNoteArgs),
    #[command(visible_alias = "d", about = "Delete a note entry")]
    Delete(GroupItemArgs),
    #[command(
        visible_alias = "m",
        about = "Move a note entry to another group, auto-creating the destination"
    )]
    Move(MoveItemArgs),
}

#[derive(Debug, Args)]
pub struct GroupNameArgs {
    #[arg(help = "Group name, which maps to groups/<group>.toml")]
    pub group: String,
}

#[derive(Debug, Args)]
pub struct GroupRenameArgs {
    #[arg(help = "Existing group name")]
    pub old_group: String,
    #[arg(help = "New group name")]
    pub new_group: String,
}

#[derive(Debug, Args)]
pub struct ItemListArgs {
    #[arg(
        short = 'g',
        long,
        conflicts_with = "all",
        help = "Only list items from one group"
    )]
    pub group: Option<String>,
    #[arg(
        short = 'a',
        long,
        help = "List items across all groups",
        long_help = "List items across all groups. This is an explicit alias for the default behavior when `--group` is omitted."
    )]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct GroupItemArgs {
    #[arg(short = 'g', long, help = "Group that contains the item")]
    pub group: String,
    #[arg(help = "Item id within the group")]
    pub id: String,
}

#[derive(Debug, Args)]
pub struct CreateShortcutArgs {
    #[arg(short = 'g', long, help = "Target group for the new shortcut")]
    pub group: String,
    #[arg(help = "Shortcut id")]
    pub id: String,
    #[arg(short = 'k', long, help = "Key combination shown in the keyboard view")]
    pub key: String,
    #[arg(short = 'd', long, help = "Human-readable description")]
    pub desc: String,
}

#[derive(Debug, Args)]
pub struct UpdateShortcutArgs {
    #[arg(short = 'g', long, help = "Group that contains the shortcut")]
    pub group: String,
    #[arg(help = "Existing shortcut id")]
    pub id: String,
    #[arg(short = 'n', long, help = "Rename the shortcut id")]
    pub new_id: Option<String>,
    #[arg(short = 'k', long, help = "Replace the key combination")]
    pub key: Option<String>,
    #[arg(short = 'd', long, help = "Replace the description")]
    pub desc: Option<String>,
}

#[derive(Debug, Args)]
pub struct CreateNoteArgs {
    #[arg(short = 'g', long, help = "Target group for the new note")]
    pub group: String,
    #[arg(help = "Note id")]
    pub id: String,
    #[arg(short = 'd', long, help = "Human-readable description")]
    pub desc: String,
}

#[derive(Debug, Args)]
pub struct UpdateNoteArgs {
    #[arg(short = 'g', long, help = "Group that contains the note")]
    pub group: String,
    #[arg(help = "Existing note id")]
    pub id: String,
    #[arg(short = 'n', long, help = "Rename the note id")]
    pub new_id: Option<String>,
    #[arg(short = 'd', long, help = "Replace the description")]
    pub desc: Option<String>,
}

#[derive(Debug, Args)]
pub struct MoveItemArgs {
    #[arg(short = 'f', long, help = "Source group")]
    pub from_group: String,
    #[arg(short = 't', long, help = "Destination group")]
    pub to_group: String,
    #[arg(help = "Item id to move")]
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelpTopic {
    Top,
    Init,
    Group,
    Shortcut,
    Note,
}

pub fn contextual_error_examples(raw_args: &[String]) -> &'static str {
    match detect_help_topic(raw_args) {
        HelpTopic::Top => TOP_LEVEL_EXAMPLES,
        HelpTopic::Init => INIT_EXAMPLES,
        HelpTopic::Group => GROUP_EXAMPLES,
        HelpTopic::Shortcut => SHORTCUT_EXAMPLES,
        HelpTopic::Note => NOTE_EXAMPLES,
    }
}

pub fn contextual_error_help_hint(raw_args: &[String]) -> &'static str {
    match detect_help_topic(raw_args) {
        HelpTopic::Top => "Try `wkey --help` for full usage.",
        HelpTopic::Init => "Try `wkey init --help` for init command usage.",
        HelpTopic::Group => "Try `wkey group --help` for group command usage.",
        HelpTopic::Shortcut => "Try `wkey shortcut --help` for shortcut command usage.",
        HelpTopic::Note => "Try `wkey note --help` for note command usage.",
    }
}

fn detect_help_topic(raw_args: &[String]) -> HelpTopic {
    let mut iter = raw_args.iter().peekable();
    while let Some(token) = iter.next() {
        match token.as_str() {
            "--config-dir" | "-C" => {
                iter.next();
            }
            token if token.starts_with("--config-dir=") => {}
            "--json" | "-j" | "--search" | "-s" | "--search-only" | "-S" => {}
            "init" | "i" => return HelpTopic::Init,
            "group" | "g" => return HelpTopic::Group,
            "shortcut" | "s" => return HelpTopic::Shortcut,
            "note" | "n" => return HelpTopic::Note,
            token if token.starts_with('-') => {}
            _ => return HelpTopic::Top,
        }
    }

    HelpTopic::Top
}
