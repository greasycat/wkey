use crate::model::{Item, ItemKind, Note, Shortcut};
use anyhow::{Result, anyhow, ensure};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const APP_TITLE: &str = "wkey";
pub const DEFAULT_APP_CONFIG_TEXT: &str = include_str!("../assets/default-config.toml");
pub const DEFAULT_KEYBOARD_LAYOUT_TEXT: &str = include_str!("../assets/default-keyboard.txt");
pub const DEFAULT_GROUP_NAME: &str = "wkey";
pub const DEFAULT_WKEY_GROUP_TEXT: &str = include_str!("../assets/default-wkey-group.toml");
pub const DEFAULT_FZF_LAYOUT: &str = "reverse";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardCell {
    Key(String),
    Gap(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub items: Vec<Item>,
    pub keyboard_layout: Vec<Vec<KeyboardCell>>,
    pub app: AppConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub pipeout: PipeoutConfig,
}

impl AppConfig {
    pub fn pipeout_command(&self) -> Option<&str> {
        self.pipeout
            .command
            .as_deref()
            .map(str::trim)
            .filter(|command| !command.is_empty())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
pub struct PipeoutConfig {
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSummary {
    pub name: String,
    pub shortcut_count: usize,
    pub note_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupItems {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct GroupFile {
    #[serde(default)]
    shortcuts: BTreeMap<String, ShortcutEntry>,
    #[serde(default)]
    notes: BTreeMap<String, NoteEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct ShortcutEntry {
    key: String,
    desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct NoteEntry {
    desc: String,
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutPatch {
    pub new_id: Option<String>,
    pub key: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NotePatch {
    pub new_id: Option<String>,
    pub desc: Option<String>,
}

pub fn load(cli_override: Option<&Path>, xdg_override: Option<&Path>) -> Result<LoadedConfig> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let keyboard_layout = load_keyboard_layout(&root)?;
    let items = load_all_items(&root)?;
    let app = load_app_config(&root)?;

    Ok(LoadedConfig {
        items,
        keyboard_layout,
        app,
    })
}

pub fn default_keyboard_layout() -> Vec<Vec<KeyboardCell>> {
    parse_keyboard_layout(DEFAULT_KEYBOARD_LAYOUT_TEXT)
}

pub fn keyboard_layout_path(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
) -> Result<PathBuf> {
    Ok(resolve_config_root(cli_override, xdg_override)?.join("keyboard.txt"))
}

pub fn app_config_path(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
) -> Result<PathBuf> {
    Ok(resolve_config_root(cli_override, xdg_override)?.join("config.toml"))
}

pub fn groups_dir_path(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
) -> Result<PathBuf> {
    Ok(resolve_config_root(cli_override, xdg_override)?.join("groups"))
}

pub fn write_default_keyboard_layout(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_KEYBOARD_LAYOUT_TEXT)?;
    Ok(())
}

pub fn write_default_app_config(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_APP_CONFIG_TEXT)?;
    Ok(())
}

pub fn write_default_group(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_WKEY_GROUP_TEXT)?;
    Ok(())
}

pub fn ensure_groups_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn list_group_summaries(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
) -> Result<Vec<GroupSummary>> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let mut summaries = Vec::new();
    for path in discover_group_files(&root)? {
        let group_name = group_name_from_path(&path)?;
        let document = load_group_file(&path)?;
        summaries.push(GroupSummary {
            name: group_name,
            shortcut_count: document.shortcuts.len(),
            note_count: document.notes.len(),
        });
    }
    Ok(summaries)
}

pub fn load_group_items(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
) -> Result<GroupItems> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let canonical = validate_group_name(group_name)?;
    let path = group_file_path_from_root(&root, &canonical);
    let document = load_group_file(&path)?;
    Ok(GroupItems {
        name: canonical.clone(),
        items: items_from_group_document(&canonical, &document),
    })
}

pub fn create_group(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
) -> Result<PathBuf> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let canonical = validate_group_name(group_name)?;
    let groups_dir = root.join("groups");
    fs::create_dir_all(&groups_dir)?;
    let path = group_file_path_from_root(&root, &canonical);
    ensure!(!path.exists(), "group '{}' already exists", canonical);
    write_group_file(&path, &GroupFile::default())?;
    Ok(path)
}

pub fn rename_group(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    old_group: &str,
    new_group: &str,
) -> Result<(PathBuf, PathBuf)> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let old_name = validate_group_name(old_group)?;
    let new_name = validate_group_name(new_group)?;
    ensure!(old_name != new_name, "group name is unchanged");

    let old_path = group_file_path_from_root(&root, &old_name);
    let new_path = group_file_path_from_root(&root, &new_name);
    ensure!(old_path.exists(), "group '{}' does not exist", old_name);
    ensure!(!new_path.exists(), "group '{}' already exists", new_name);
    fs::rename(&old_path, &new_path)?;
    Ok((old_path, new_path))
}

pub fn delete_group(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
) -> Result<PathBuf> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let canonical = validate_group_name(group_name)?;
    let path = group_file_path_from_root(&root, &canonical);
    let document = load_group_file(&path)?;
    ensure!(
        document.shortcuts.is_empty() && document.notes.is_empty(),
        "group '{}' is not empty; use `wkey group force-delete {}` or `wkey g D {}` to remove it anyway",
        canonical,
        canonical,
        canonical
    );
    fs::remove_file(&path)?;
    Ok(path)
}

pub fn force_delete_group(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
) -> Result<PathBuf> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let canonical = validate_group_name(group_name)?;
    let path = group_file_path_from_root(&root, &canonical);
    load_group_file(&path)?;
    fs::remove_file(&path)?;
    Ok(path)
}

pub fn create_shortcut(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
    key: &str,
    desc: &str,
) -> Result<Shortcut> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file_for_write(&path)?;
    ensure_item_name_available(&document, &item_id)?;
    document.shortcuts.insert(
        item_id.clone(),
        ShortcutEntry {
            key: validate_key_combo(key)?,
            desc: desc.trim().to_owned(),
        },
    );
    write_group_file(&path, &document)?;
    Ok(Shortcut::new(&item_id, key.trim(), desc.trim(), &group))
}

pub fn update_shortcut(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
    patch: ShortcutPatch,
) -> Result<Shortcut> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file(&path)?;
    let entry = document
        .shortcuts
        .remove(&item_id)
        .ok_or_else(|| anyhow!("shortcut '{}' does not exist in group '{}'", item_id, group))?;
    let new_id = patch
        .new_id
        .as_deref()
        .map(validate_item_id)
        .transpose()?
        .unwrap_or(item_id);
    ensure_item_name_available_except(&document, &new_id, ItemKind::Shortcut, id)?;
    let updated = ShortcutEntry {
        key: patch
            .key
            .as_deref()
            .map(validate_key_combo)
            .transpose()?
            .unwrap_or(entry.key),
        desc: patch.desc.unwrap_or(entry.desc).trim().to_owned(),
    };
    document.shortcuts.insert(new_id.clone(), updated.clone());
    write_group_file(&path, &document)?;
    Ok(Shortcut::new(&new_id, &updated.key, &updated.desc, &group))
}

pub fn delete_shortcut(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
) -> Result<Shortcut> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file(&path)?;
    let entry = document
        .shortcuts
        .remove(&item_id)
        .ok_or_else(|| anyhow!("shortcut '{}' does not exist in group '{}'", item_id, group))?;
    write_group_file(&path, &document)?;
    Ok(Shortcut::new(&item_id, &entry.key, &entry.desc, &group))
}

pub fn move_shortcut(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    from_group: &str,
    to_group: &str,
    id: &str,
) -> Result<Shortcut> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let from = validate_group_name(from_group)?;
    let to = validate_group_name(to_group)?;
    let item_id = validate_item_id(id)?;
    ensure!(from != to, "source and destination groups are the same");

    let from_path = group_file_path_from_root(&root, &from);
    let to_path = group_file_path_from_root(&root, &to);
    let mut from_document = load_group_file(&from_path)?;
    let mut to_document = load_group_file_for_write(&to_path)?;
    ensure_item_name_available(&to_document, &item_id)?;

    let entry = from_document
        .shortcuts
        .remove(&item_id)
        .ok_or_else(|| anyhow!("shortcut '{}' does not exist in group '{}'", item_id, from))?;
    to_document.shortcuts.insert(item_id.clone(), entry.clone());

    write_group_file(&from_path, &from_document)?;
    write_group_file(&to_path, &to_document)?;

    Ok(Shortcut::new(&item_id, &entry.key, &entry.desc, &to))
}

pub fn create_note(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
    desc: &str,
) -> Result<Note> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file_for_write(&path)?;
    ensure_item_name_available(&document, &item_id)?;
    document.notes.insert(
        item_id.clone(),
        NoteEntry {
            desc: desc.trim().to_owned(),
        },
    );
    write_group_file(&path, &document)?;
    Ok(Note::new(&item_id, desc.trim(), &group))
}

pub fn update_note(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
    patch: NotePatch,
) -> Result<Note> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file(&path)?;
    let entry = document
        .notes
        .remove(&item_id)
        .ok_or_else(|| anyhow!("note '{}' does not exist in group '{}'", item_id, group))?;
    let new_id = patch
        .new_id
        .as_deref()
        .map(validate_item_id)
        .transpose()?
        .unwrap_or(item_id);
    ensure_item_name_available_except(&document, &new_id, ItemKind::Note, id)?;
    let updated = NoteEntry {
        desc: patch.desc.unwrap_or(entry.desc).trim().to_owned(),
    };
    document.notes.insert(new_id.clone(), updated.clone());
    write_group_file(&path, &document)?;
    Ok(Note::new(&new_id, &updated.desc, &group))
}

pub fn delete_note(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    group_name: &str,
    id: &str,
) -> Result<Note> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let group = validate_group_name(group_name)?;
    let item_id = validate_item_id(id)?;
    let path = group_file_path_from_root(&root, &group);
    let mut document = load_group_file(&path)?;
    let entry = document
        .notes
        .remove(&item_id)
        .ok_or_else(|| anyhow!("note '{}' does not exist in group '{}'", item_id, group))?;
    write_group_file(&path, &document)?;
    Ok(Note::new(&item_id, &entry.desc, &group))
}

pub fn move_note(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
    from_group: &str,
    to_group: &str,
    id: &str,
) -> Result<Note> {
    let root = resolve_config_root(cli_override, xdg_override)?;
    let from = validate_group_name(from_group)?;
    let to = validate_group_name(to_group)?;
    let item_id = validate_item_id(id)?;
    ensure!(from != to, "source and destination groups are the same");

    let from_path = group_file_path_from_root(&root, &from);
    let to_path = group_file_path_from_root(&root, &to);
    let mut from_document = load_group_file(&from_path)?;
    let mut to_document = load_group_file_for_write(&to_path)?;
    ensure_item_name_available(&to_document, &item_id)?;

    let entry = from_document
        .notes
        .remove(&item_id)
        .ok_or_else(|| anyhow!("note '{}' does not exist in group '{}'", item_id, from))?;
    to_document.notes.insert(item_id.clone(), entry.clone());

    write_group_file(&from_path, &from_document)?;
    write_group_file(&to_path, &to_document)?;

    Ok(Note::new(&item_id, &entry.desc, &to))
}

fn load_all_items(root: &Path) -> Result<Vec<Item>> {
    let mut items = Vec::new();
    for path in discover_group_files(root)? {
        let group_name = group_name_from_path(&path)?;
        let document = load_group_file(&path)?;
        items.extend(items_from_group_document(&group_name, &document));
    }
    Ok(items)
}

fn items_from_group_document(group_name: &str, document: &GroupFile) -> Vec<Item> {
    let mut items = Vec::new();
    for (id, entry) in &document.shortcuts {
        items.push(Item::Shortcut(Shortcut::new(
            id,
            &entry.key,
            &entry.desc,
            group_name,
        )));
    }
    for (id, entry) in &document.notes {
        items.push(Item::Note(Note::new(id, &entry.desc, group_name)));
    }
    items
}

fn discover_group_files(root: &Path) -> Result<Vec<PathBuf>> {
    let groups_dir = root.join("groups");
    if !groups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(groups_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("toml")
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn load_keyboard_layout(root: &Path) -> Result<Vec<Vec<KeyboardCell>>> {
    let path = root.join("keyboard.txt");
    if !path.exists() {
        return Ok(default_keyboard_layout());
    }

    let rows = parse_keyboard_layout(&fs::read_to_string(path)?);
    if rows.is_empty() {
        Ok(default_keyboard_layout())
    } else {
        Ok(rows)
    }
}

fn load_app_config(root: &Path) -> Result<AppConfig> {
    let path = root.join("config.toml");
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

fn parse_keyboard_layout(source: &str) -> Vec<Vec<KeyboardCell>> {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !line.trim_start().starts_with('#')
        })
        .map(parse_keyboard_row)
        .filter(|row| !row.is_empty())
        .collect()
}

fn parse_keyboard_row(line: &str) -> Vec<KeyboardCell> {
    let mut row = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            let mut width = 1;
            while chars.peek().is_some_and(|next| next.is_whitespace()) {
                chars.next();
                width += 1;
            }

            if chars.peek().is_some() {
                row.push(KeyboardCell::Gap(width));
            }
            continue;
        }

        let mut label = String::from(ch);
        while chars.peek().is_some_and(|next| !next.is_whitespace()) {
            label.push(chars.next().unwrap_or_default());
        }
        row.push(KeyboardCell::Key(label));
    }

    row
}

fn load_group_file(path: &Path) -> Result<GroupFile> {
    ensure!(
        path.exists(),
        "group file '{}' does not exist",
        path.display()
    );
    let parsed: GroupFile = toml::from_str(&fs::read_to_string(path)?)?;
    ensure_no_cross_type_conflicts(&parsed)?;
    Ok(parsed)
}

fn load_group_file_for_write(path: &Path) -> Result<GroupFile> {
    if path.exists() {
        load_group_file(path)
    } else {
        Ok(GroupFile::default())
    }
}

fn write_group_file(path: &Path, document: &GroupFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let rendered = render_group_file(document);
    atomic_write(path, &rendered)
}

fn render_group_file(document: &GroupFile) -> String {
    let mut output = String::new();
    output.push_str("[shortcuts]\n");
    for (id, shortcut) in &document.shortcuts {
        output.push_str(id);
        output.push_str(" = { key = ");
        output.push_str(&toml::Value::String(shortcut.key.clone()).to_string());
        output.push_str(", desc = ");
        output.push_str(&toml::Value::String(shortcut.desc.clone()).to_string());
        output.push_str(" }\n");
    }
    output.push('\n');
    output.push_str("[notes]\n");
    for (id, note) in &document.notes {
        output.push_str(id);
        output.push_str(" = { desc = ");
        output.push_str(&toml::Value::String(note.desc.clone()).to_string());
        output.push_str(" }\n");
    }
    output
}

fn atomic_write(path: &Path, contents: &str) -> Result<()> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temp_path = path.with_extension(format!("toml.tmp.{nonce}"));
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn group_file_path_from_root(root: &Path, group_name: &str) -> PathBuf {
    root.join("groups").join(format!("{group_name}.toml"))
}

fn group_name_from_path(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("invalid group file name '{}'", path.display()))
}

fn ensure_no_cross_type_conflicts(document: &GroupFile) -> Result<()> {
    for id in document.shortcuts.keys() {
        ensure!(
            !document.notes.contains_key(id),
            "group file contains duplicate item id '{}' across shortcuts and notes",
            id
        );
    }
    Ok(())
}

fn ensure_item_name_available(document: &GroupFile, id: &str) -> Result<()> {
    ensure!(
        !document.shortcuts.contains_key(id) && !document.notes.contains_key(id),
        "item '{}' already exists",
        id
    );
    Ok(())
}

fn ensure_item_name_available_except(
    document: &GroupFile,
    id: &str,
    kind: ItemKind,
    current_id: &str,
) -> Result<()> {
    let taken_by_other_shortcut =
        document.shortcuts.contains_key(id) && !(kind == ItemKind::Shortcut && id == current_id);
    let taken_by_other_note =
        document.notes.contains_key(id) && !(kind == ItemKind::Note && id == current_id);
    ensure!(
        !taken_by_other_shortcut && !taken_by_other_note,
        "item '{}' already exists",
        id
    );
    Ok(())
}

fn validate_group_name(group_name: &str) -> Result<String> {
    let trimmed = group_name.trim();
    ensure!(!trimmed.is_empty(), "group name cannot be empty");
    ensure!(
        !trimmed.contains('/') && !trimmed.contains('\\'),
        "group name cannot contain path separators"
    );
    ensure!(trimmed != "." && trimmed != "..", "invalid group name");
    Ok(trimmed.to_owned())
}

fn validate_item_id(id: &str) -> Result<String> {
    let trimmed = id.trim();
    ensure!(!trimmed.is_empty(), "item id cannot be empty");
    ensure!(
        !trimmed.contains('.') && !trimmed.contains(char::is_whitespace),
        "item id cannot contain whitespace or dots"
    );
    Ok(trimmed.to_owned())
}

fn validate_key_combo(key: &str) -> Result<String> {
    let trimmed = key.trim();
    ensure!(!trimmed.is_empty(), "shortcut key cannot be empty");
    Ok(trimmed.to_owned())
}

fn resolve_config_root(
    cli_override: Option<&Path>,
    xdg_override: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = cli_override {
        return Ok(path.to_path_buf());
    }

    if let Some(config_home) = xdg_override {
        return Ok(config_home.join("wkey"));
    }

    xdg::BaseDirectories::with_prefix("wkey")
        .get_config_home()
        .ok_or_else(|| anyhow!("unable to resolve XDG config home for wkey"))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_APP_CONFIG_TEXT, DEFAULT_GROUP_NAME, DEFAULT_WKEY_GROUP_TEXT, GroupFile,
        KeyboardCell, NoteEntry, ShortcutEntry, default_keyboard_layout, parse_keyboard_layout,
        render_group_file,
    };
    use std::collections::BTreeMap;

    #[test]
    fn default_keyboard_layout_is_not_empty() {
        assert!(!default_keyboard_layout().is_empty());
    }

    #[test]
    fn parse_keyboard_layout_preserves_spaces_as_gaps() {
        let parsed = parse_keyboard_layout("Ctrl  Alt   Space\n");

        assert_eq!(
            parsed,
            vec![vec![
                KeyboardCell::Key("Ctrl".to_owned()),
                KeyboardCell::Gap(2),
                KeyboardCell::Key("Alt".to_owned()),
                KeyboardCell::Gap(3),
                KeyboardCell::Key("Space".to_owned()),
            ]]
        );
    }

    #[test]
    fn render_group_file_uses_keyed_tables() {
        let document = GroupFile {
            shortcuts: BTreeMap::from([(
                "copy".to_owned(),
                ShortcutEntry {
                    key: "Ctrl+C".to_owned(),
                    desc: "Copy selection".to_owned(),
                },
            )]),
            notes: BTreeMap::from([(
                "tip".to_owned(),
                NoteEntry {
                    desc: "Use !!".to_owned(),
                },
            )]),
        };

        let rendered = render_group_file(&document);

        assert!(rendered.contains("[shortcuts]"));
        assert!(rendered.contains("copy = { key = \"Ctrl+C\", desc = \"Copy selection\" }"));
        assert!(rendered.contains("[notes]"));
        assert!(rendered.contains("tip = { desc = \"Use !!\" }"));
    }

    #[test]
    fn bundled_default_group_uses_expected_shape() {
        let parsed: GroupFile = toml::from_str(DEFAULT_WKEY_GROUP_TEXT).unwrap();

        assert!(parsed.shortcuts.contains_key("quit"));
        assert!(parsed.notes.contains_key("run"));
        assert!(DEFAULT_GROUP_NAME == "wkey");
    }

    #[test]
    fn bundled_default_app_config_parses() {
        let parsed: super::AppConfig = toml::from_str(DEFAULT_APP_CONFIG_TEXT).unwrap();

        assert_eq!(parsed.pipeout_command(), None);
    }
}
