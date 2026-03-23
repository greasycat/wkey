use crate::config::DEFAULT_FZF_LAYOUT;
use crate::model::{Item, ItemKind};
use anyhow::{Result, anyhow};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn search_items(items: &[Item], fzf_bin: &Path) -> Result<Option<String>> {
    search_items_with_fallback(items, fzf_bin, |_| Ok(None))
}

pub fn search_items_with_fallback<F>(
    items: &[Item],
    fzf_bin: &Path,
    fallback: F,
) -> Result<Option<String>>
where
    F: FnOnce(&[Item]) -> Result<Option<String>>,
{
    if items.is_empty() {
        return Ok(None);
    }

    match run_fzf(items, fzf_bin) {
        Ok(selection) => Ok(selection),
        Err(SearchError::Spawn(error))
            if matches!(
                error.kind(),
                ErrorKind::NotFound
                    | ErrorKind::PermissionDenied
                    | ErrorKind::InvalidInput
                    | ErrorKind::Unsupported
            ) =>
        {
            fallback(items)
        }
        Err(SearchError::Spawn(error)) => Err(error.into()),
        Err(SearchError::Execution(error)) => Err(error),
    }
}

enum SearchError {
    Spawn(std::io::Error),
    Execution(anyhow::Error),
}

fn run_fzf(items: &[Item], fzf_bin: &Path) -> std::result::Result<Option<String>, SearchError> {
    let input = items
        .iter()
        .map(|item| {
            format!(
                "{}\t{}\t{}\t{}\t{}",
                item.selection_key(),
                item.kind().as_str(),
                item.group(),
                item.key_combo().unwrap_or(""),
                item.desc()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new(fzf_bin)
        .args([
            "--delimiter",
            "\t",
            "--with-nth",
            "2..",
            "--layout",
            DEFAULT_FZF_LAYOUT,
            "--prompt",
            "wkey> ",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(SearchError::Spawn)?;

    child
        .stdin
        .take()
        .expect("fzf stdin is available")
        .write_all(input.as_bytes())
        .map_err(|error| SearchError::Execution(error.into()))?;

    let output = child
        .wait_with_output()
        .map_err(|error| SearchError::Execution(error.into()))?;
    if matches!(output.status.code(), Some(1 | 130)) {
        return Ok(None);
    }

    if !output.status.success() {
        return Err(SearchError::Execution(anyhow!(
            "fzf exited with status {:?}",
            output.status.code()
        )));
    }

    Ok(String::from_utf8(output.stdout)
        .map_err(anyhow::Error::from)
        .map_err(SearchError::Execution)?
        .lines()
        .next()
        .and_then(|line| line.split('\t').next())
        .filter(|value| !value.is_empty())
        .map(str::to_owned))
}

pub fn kind_from_selection_key(selection_key: &str) -> Option<ItemKind> {
    let mut parts = selection_key.split('\u{1f}');
    match parts.next()? {
        "shortcut" => Some(ItemKind::Shortcut),
        "note" => Some(ItemKind::Note),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::search_items_with_fallback;
    use crate::model::{Item, Note, Shortcut};
    use std::path::Path;

    fn sample_items() -> Vec<Item> {
        vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Remember this", "shell")),
        ]
    }

    #[test]
    fn missing_fzf_uses_fallback_selector() {
        let items = sample_items();
        let selected =
            search_items_with_fallback(&items, Path::new("/definitely/missing-fzf"), |_| {
                Ok(Some("note\u{1f}shell\u{1f}tip".to_owned()))
            })
            .unwrap();

        assert_eq!(selected.as_deref(), Some("note\u{1f}shell\u{1f}tip"));
    }
}
