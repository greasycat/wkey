mod cli;

use anyhow::{Context, Result, anyhow, ensure};
use clap::Parser;
use serde::Serialize;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use wkey::config::{self, NotePatch, ShortcutPatch};
use wkey::model::{Item, Note, Shortcut};
use wkey::{pipeout, search, ui};

fn main() -> Result<()> {
    let raw_args = std::env::args_os().collect::<Vec<_>>();
    let cli = match cli::Cli::try_parse_from(&raw_args) {
        Ok(cli) => cli,
        Err(error) => exit_with_clap_error(error, &raw_args),
    };

    match cli.command {
        Some(cli::Command::Init(args)) => {
            ensure!(
                !cli.json,
                "--json is only supported for group, shortcut, and note commands"
            );
            init_config(cli.config_dir.as_deref(), args.yes)
        }
        Some(cli::Command::Group { command }) => {
            run_group_command(cli.config_dir.as_deref(), command, cli.json)
        }
        Some(cli::Command::Shortcut { command }) => {
            run_shortcut_command(cli.config_dir.as_deref(), command, cli.json)
        }
        Some(cli::Command::Note { command }) => {
            run_note_command(cli.config_dir.as_deref(), command, cli.json)
        }
        None => {
            ensure!(
                !cli.json,
                "--json is only supported for group, shortcut, and note commands"
            );
            if cli.search_only {
                run_search_only(cli.config_dir.as_deref())
            } else {
                run_tui(cli.config_dir.as_deref(), cli.search)
            }
        }
    }
}

fn exit_with_clap_error(error: clap::Error, raw_args: &[OsString]) -> ! {
    if matches!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
    ) {
        print!("{error}");
        std::process::exit(error.exit_code())
    }

    let raw_args = raw_args
        .iter()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    eprint!("{error}");
    eprintln!();
    eprintln!("{}", cli::contextual_error_help_hint(&raw_args));
    eprintln!();
    eprintln!("{}", cli::contextual_error_examples(&raw_args));
    std::process::exit(error.exit_code())
}

#[derive(Debug, Serialize)]
struct JsonGroupSummary {
    name: String,
    shortcut_count: usize,
    note_count: usize,
}

#[derive(Debug, Serialize)]
struct JsonGroupItems {
    name: String,
    items: Vec<JsonItem>,
}

#[derive(Debug, Serialize)]
struct JsonGroupPath {
    name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct JsonGroupRename {
    old_name: String,
    new_name: String,
    old_path: String,
    new_path: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
enum JsonItem {
    #[serde(rename = "shortcut")]
    Shortcut {
        group: String,
        id: String,
        key: String,
        desc: String,
    },
    #[serde(rename = "note")]
    Note {
        group: String,
        id: String,
        desc: String,
    },
}

impl From<&Item> for JsonItem {
    fn from(item: &Item) -> Self {
        match item {
            Item::Shortcut(shortcut) => Self::Shortcut {
                group: shortcut.group.clone(),
                id: shortcut.id.clone(),
                key: shortcut.key.clone(),
                desc: shortcut.desc.clone(),
            },
            Item::Note(note) => Self::Note {
                group: note.group.clone(),
                id: note.id.clone(),
                desc: note.desc.clone(),
            },
        }
    }
}

fn run_tui(config_dir: Option<&Path>, search_enabled: bool) -> Result<()> {
    let loaded = config::load(config_dir, None)?;
    let selected_id = if search_enabled {
        select_item_id(&loaded)?
    } else {
        None
    };

    ui::render_inline_with_layout_and_pipeout(
        &loaded.keyboard_layout,
        &loaded.items,
        selected_id.as_deref(),
        loaded.app.pipeout_command(),
    )
}

fn run_search_only(config_dir: Option<&Path>) -> Result<()> {
    let loaded = config::load(config_dir, None)?;
    if let Some(selected_id) = select_item_id(&loaded)? {
        let selected_desc = selected_desc(&loaded, &selected_id)?;
        if let Some(command) = loaded.app.pipeout_command() {
            pipeout::write_to_command(command, selected_desc)?;
        }
        println!("{selected_desc}");
    }
    Ok(())
}

fn select_item_id(loaded: &config::LoadedConfig) -> Result<Option<String>> {
    search::search_items_with_fallback(&loaded.items, Path::new("fzf"), |items| {
        ui::select_item_with_layout(&loaded.keyboard_layout, items, None)
    })
}

fn selected_desc<'a>(loaded: &'a config::LoadedConfig, selected_id: &str) -> Result<&'a str> {
    loaded
        .items
        .iter()
        .find(|item| item.selection_key() == selected_id)
        .map(Item::desc)
        .with_context(|| format!("selected item '{selected_id}' was not found"))
}

fn init_config(config_dir: Option<&Path>, yes: bool) -> Result<()> {
    let app_config_path = config::app_config_path(config_dir, None)?;
    let keyboard_path = config::keyboard_layout_path(config_dir, None)?;
    let groups_dir = config::groups_dir_path(config_dir, None)?;
    let default_group_path = groups_dir.join(format!("{}.toml", wkey::config::DEFAULT_GROUP_NAME));

    if should_write(&app_config_path, yes)? {
        config::write_default_app_config(&app_config_path)?;
        println!("Wrote {}", app_config_path.display());
    } else {
        println!("Skipped {}", app_config_path.display());
    }

    if should_write(&keyboard_path, yes)? {
        config::write_default_keyboard_layout(&keyboard_path)?;
        println!("Wrote {}", keyboard_path.display());
    } else {
        println!("Skipped {}", keyboard_path.display());
    }

    config::ensure_groups_dir(&groups_dir)?;
    println!("Ensured {}", groups_dir.display());

    if should_write(&default_group_path, yes)? {
        config::write_default_group(&default_group_path)?;
        println!("Wrote {}", default_group_path.display());
    } else {
        println!("Skipped {}", default_group_path.display());
    }
    Ok(())
}

fn should_write(path: &Path, yes: bool) -> Result<bool> {
    if !path.exists() || yes {
        return Ok(true);
    }

    print!("Overwrite {}? [y/N] ", path.display());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "Yes" | "YES"))
}

fn run_group_command(
    config_dir: Option<&Path>,
    command: cli::GroupCommand,
    json: bool,
) -> Result<()> {
    match command {
        cli::GroupCommand::List => {
            let groups = config::list_group_summaries(config_dir, None)?;
            if json {
                let payload = groups
                    .into_iter()
                    .map(|group| JsonGroupSummary {
                        name: group.name,
                        shortcut_count: group.shortcut_count,
                        note_count: group.note_count,
                    })
                    .collect::<Vec<_>>();
                print_json(&payload)
            } else {
                if groups.is_empty() {
                    println!("No groups configured.");
                    return Ok(());
                }
                for group in groups {
                    println!(
                        "{}\tshortcuts:{}\tnotes:{}",
                        group.name, group.shortcut_count, group.note_count
                    );
                }
                Ok(())
            }
        }
        cli::GroupCommand::Show(args) => {
            let group = config::load_group_items(config_dir, None, &args.group)?;
            if json {
                let payload = JsonGroupItems {
                    name: group.name,
                    items: group.items.iter().map(JsonItem::from).collect(),
                };
                print_json(&payload)
            } else {
                println!("group\t{}", group.name);
                if group.items.is_empty() {
                    println!("(empty)");
                } else {
                    for item in group.items {
                        print_item(&item);
                    }
                }
                Ok(())
            }
        }
        cli::GroupCommand::Create(args) => {
            let path = config::create_group(config_dir, None, &args.group)?;
            if json {
                print_json(&JsonGroupPath {
                    name: group_name_from_path(&path)?,
                    path: display_path(&path),
                })
            } else {
                println!("Created {}", path.display());
                Ok(())
            }
        }
        cli::GroupCommand::Rename(args) => {
            let (old_path, new_path) =
                config::rename_group(config_dir, None, &args.old_group, &args.new_group)?;
            if json {
                print_json(&JsonGroupRename {
                    old_name: group_name_from_path(&old_path)?,
                    new_name: group_name_from_path(&new_path)?,
                    old_path: display_path(&old_path),
                    new_path: display_path(&new_path),
                })
            } else {
                println!("Renamed {} -> {}", old_path.display(), new_path.display());
                Ok(())
            }
        }
        cli::GroupCommand::Delete(args) => {
            let path = config::delete_group(config_dir, None, &args.group)?;
            if json {
                print_json(&JsonGroupPath {
                    name: group_name_from_path(&path)?,
                    path: display_path(&path),
                })
            } else {
                println!("Deleted {}", path.display());
                Ok(())
            }
        }
        cli::GroupCommand::ForceDelete(args) => {
            let path = config::force_delete_group(config_dir, None, &args.group)?;
            if json {
                print_json(&JsonGroupPath {
                    name: group_name_from_path(&path)?,
                    path: display_path(&path),
                })
            } else {
                println!("Deleted {}", path.display());
                Ok(())
            }
        }
    }
}

fn run_shortcut_command(
    config_dir: Option<&Path>,
    command: cli::ShortcutCommand,
    json: bool,
) -> Result<()> {
    match command {
        cli::ShortcutCommand::List(args) => {
            let items = list_items(config_dir, args, Some(wkey::model::ItemKind::Shortcut))?;
            if json {
                print_json(&items.iter().map(JsonItem::from).collect::<Vec<_>>())
            } else {
                print_items(&items)
            }
        }
        cli::ShortcutCommand::Show(args) => {
            let group = config::load_group_items(config_dir, None, &args.group)?;
            let item = group
                .items
                .into_iter()
                .find(|item| matches!(item, Item::Shortcut(shortcut) if shortcut.id == args.id))
                .ok_or_else(|| {
                    anyhow!("shortcut '{}' not found in group '{}'", args.id, args.group)
                })?;
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::ShortcutCommand::Create(args) => {
            let shortcut = config::create_shortcut(
                config_dir,
                None,
                &args.group,
                &args.id,
                &args.key,
                &args.desc,
            )?;
            let item = Item::Shortcut(shortcut);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::ShortcutCommand::Update(args) => {
            ensure!(
                args.new_id.is_some() || args.key.is_some() || args.desc.is_some(),
                "provide at least one field to update"
            );
            let shortcut = config::update_shortcut(
                config_dir,
                None,
                &args.group,
                &args.id,
                ShortcutPatch {
                    new_id: args.new_id,
                    key: args.key,
                    desc: args.desc,
                },
            )?;
            let item = Item::Shortcut(shortcut);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::ShortcutCommand::Delete(args) => {
            let shortcut = config::delete_shortcut(config_dir, None, &args.group, &args.id)?;
            let item = Item::Shortcut(shortcut);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                if let Item::Shortcut(shortcut) = &item {
                    println!("Deleted shortcut {}/{}", shortcut.group, shortcut.id);
                }
                Ok(())
            }
        }
        cli::ShortcutCommand::Move(args) => {
            let shortcut = config::move_shortcut(
                config_dir,
                None,
                &args.from_group,
                &args.to_group,
                &args.id,
            )?;
            let item = Item::Shortcut(shortcut);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
    }
}

fn run_note_command(
    config_dir: Option<&Path>,
    command: cli::NoteCommand,
    json: bool,
) -> Result<()> {
    match command {
        cli::NoteCommand::List(args) => {
            let items = list_items(config_dir, args, Some(wkey::model::ItemKind::Note))?;
            if json {
                print_json(&items.iter().map(JsonItem::from).collect::<Vec<_>>())
            } else {
                print_items(&items)
            }
        }
        cli::NoteCommand::Show(args) => {
            let group = config::load_group_items(config_dir, None, &args.group)?;
            let item = group
                .items
                .into_iter()
                .find(|item| matches!(item, Item::Note(note) if note.id == args.id))
                .ok_or_else(|| anyhow!("note '{}' not found in group '{}'", args.id, args.group))?;
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::NoteCommand::Create(args) => {
            let note = config::create_note(config_dir, None, &args.group, &args.id, &args.desc)?;
            let item = Item::Note(note);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::NoteCommand::Update(args) => {
            ensure!(
                args.new_id.is_some() || args.desc.is_some(),
                "provide at least one field to update"
            );
            let note = config::update_note(
                config_dir,
                None,
                &args.group,
                &args.id,
                NotePatch {
                    new_id: args.new_id,
                    desc: args.desc,
                },
            )?;
            let item = Item::Note(note);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
        cli::NoteCommand::Delete(args) => {
            let note = config::delete_note(config_dir, None, &args.group, &args.id)?;
            let item = Item::Note(note);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                if let Item::Note(note) = &item {
                    println!("Deleted note {}/{}", note.group, note.id);
                }
                Ok(())
            }
        }
        cli::NoteCommand::Move(args) => {
            let note =
                config::move_note(config_dir, None, &args.from_group, &args.to_group, &args.id)?;
            let item = Item::Note(note);
            if json {
                print_json(&JsonItem::from(&item))
            } else {
                print_item(&item);
                Ok(())
            }
        }
    }
}

fn list_items(
    config_dir: Option<&Path>,
    args: cli::ItemListArgs,
    kind: Option<wkey::model::ItemKind>,
) -> Result<Vec<Item>> {
    let items = if let Some(group) = args.group {
        config::load_group_items(config_dir, None, &group)?.items
    } else {
        let _ = args.all;
        config::load(config_dir, None)?.items
    };

    Ok(items
        .into_iter()
        .filter(|item| kind.is_none_or(|expected| item.kind() == expected))
        .collect::<Vec<_>>())
}

fn print_items(items: &[Item]) -> Result<()> {
    if items.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    for item in items {
        print_item(item);
    }
    Ok(())
}

fn print_item(item: &Item) {
    match item {
        Item::Shortcut(Shortcut {
            group,
            id,
            key,
            desc,
        }) => {
            println!("shortcut\t{}\t{}\t{}\t{}", group, id, key, desc);
        }
        Item::Note(Note { group, id, desc }) => {
            println!("note\t{}\t{}\t{}", group, id, desc);
        }
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn group_name_from_path(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
        .with_context(|| format!("unable to determine group name from '{}'", path.display()))
}
