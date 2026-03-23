use std::fs;
use wkey::model::{Item, Note, Shortcut};

#[test]
fn search_items_returns_selected_item_key_from_fzf_output() {
    let temp = tempfile::tempdir().unwrap();
    let fake_fzf = temp.path().join("fake-fzf");
    fs::write(
        &fake_fzf,
        "#!/bin/sh\nprintf 'note\\037shell\\037tip\\tnote\\tshell\\t\\tRemember this\\n'",
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake_fzf).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_fzf, perms).unwrap();
    }

    let items = vec![
        Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
        Item::Note(Note::new("tip", "Remember this", "shell")),
    ];
    let selected = wkey::search::search_items(&items, fake_fzf.as_path()).unwrap();

    assert_eq!(selected.as_deref(), Some("note\u{1f}shell\u{1f}tip"));
}

#[test]
fn search_items_with_fallback_does_not_invoke_fallback_on_cancel() {
    let temp = tempfile::tempdir().unwrap();
    let fake_fzf = temp.path().join("fake-fzf");
    fs::write(&fake_fzf, "#!/bin/sh\nexit 130\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake_fzf).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_fzf, perms).unwrap();
    }

    let items = vec![
        Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
        Item::Note(Note::new("tip", "Remember this", "shell")),
    ];
    let selected = wkey::search::search_items_with_fallback(&items, fake_fzf.as_path(), |_| {
        panic!("fallback should not be used when fzf exits with cancel")
    })
    .unwrap();

    assert_eq!(selected, None);
}
