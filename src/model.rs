#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub id: String,
    pub key: String,
    pub desc: String,
    pub group: String,
}

impl Shortcut {
    pub fn new(id: &str, key: &str, desc: &str, group: &str) -> Self {
        Self {
            id: id.to_owned(),
            key: key.to_owned(),
            desc: desc.to_owned(),
            group: group.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub desc: String,
    pub group: String,
}

impl Note {
    pub fn new(id: &str, desc: &str, group: &str) -> Self {
        Self {
            id: id.to_owned(),
            desc: desc.to_owned(),
            group: group.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Shortcut,
    Note,
}

impl ItemKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shortcut => "shortcut",
            Self::Note => "note",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Shortcut(Shortcut),
    Note(Note),
}

impl Item {
    pub fn id(&self) -> &str {
        match self {
            Self::Shortcut(shortcut) => &shortcut.id,
            Self::Note(note) => &note.id,
        }
    }

    pub fn desc(&self) -> &str {
        match self {
            Self::Shortcut(shortcut) => &shortcut.desc,
            Self::Note(note) => &note.desc,
        }
    }

    pub fn group(&self) -> &str {
        match self {
            Self::Shortcut(shortcut) => &shortcut.group,
            Self::Note(note) => &note.group,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            Self::Shortcut(_) => ItemKind::Shortcut,
            Self::Note(_) => ItemKind::Note,
        }
    }

    pub fn key_combo(&self) -> Option<&str> {
        match self {
            Self::Shortcut(shortcut) => Some(&shortcut.key),
            Self::Note(_) => None,
        }
    }

    pub fn selection_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}",
            self.kind().as_str(),
            self.group(),
            self.id()
        )
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let terms = query
            .split_whitespace()
            .map(|term| term.to_ascii_lowercase())
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return true;
        }

        let haystack = [
            self.kind().as_str(),
            self.group(),
            self.id(),
            self.desc(),
            self.key_combo().unwrap_or(""),
        ]
        .join("\n")
        .to_ascii_lowercase();

        terms.iter().all(|term| haystack.contains(term))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppView<'a> {
    pub items: &'a [Item],
    pub selected_id: Option<&'a str>,
}

impl<'a> AppView<'a> {
    pub fn selected(self) -> Option<&'a Item> {
        self.selected_id.and_then(|selection| {
            self.items
                .iter()
                .find(|item| item.selection_key() == selection)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Item, Note, Shortcut};

    #[test]
    fn shortcut_matches_query_is_case_insensitive() {
        let item = Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell"));

        assert!(item.matches_query("ctrl+c"));
        assert!(item.matches_query("COPY"));
        assert!(item.matches_query("shell"));
    }

    #[test]
    fn note_matches_query_uses_kind_and_desc() {
        let item = Item::Note(Note::new(
            "tip",
            "Use !! to repeat the previous command",
            "shell",
        ));

        assert!(item.matches_query("note"));
        assert!(item.matches_query("repeat previous"));
        assert!(!item.matches_query("ctrl"));
    }
}
