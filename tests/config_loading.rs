use wkey::config::KeyboardCell;
use wkey::model::{Item, ItemKind};

#[test]
fn load_reads_group_files_without_root_config() {
    let temp = tempfile::tempdir().unwrap();
    let config_home = temp.path().join("xdg");
    let app_dir = config_home.join("wkey");
    std::fs::create_dir_all(app_dir.join("groups")).unwrap();

    std::fs::write(
        app_dir.join("groups/shell.toml"),
        r#"
[shortcuts]
copy = { key = "Ctrl+C", desc = "Copy selection" }

[notes]
tip = { desc = "Use !! to repeat the previous command" }
"#,
    )
    .unwrap();

    let loaded = wkey::config::load(None, Some(config_home.as_path())).unwrap();

    assert_eq!(loaded.items.len(), 2);
    assert_eq!(loaded.items[0].kind(), ItemKind::Shortcut);
    assert_eq!(loaded.items[1].kind(), ItemKind::Note);

    match &loaded.items[0] {
        Item::Shortcut(shortcut) => {
            assert_eq!(shortcut.id, "copy");
            assert_eq!(shortcut.group, "shell");
        }
        _ => panic!("expected shortcut"),
    }
}

#[test]
fn load_reads_keyboard_layout_from_keyboard_txt() {
    let temp = tempfile::tempdir().unwrap();
    let config_home = temp.path().join("xdg");
    let app_dir = config_home.join("wkey");
    std::fs::create_dir_all(app_dir.join("groups")).unwrap();

    std::fs::write(
        app_dir.join("groups/shell.toml"),
        "[shortcuts]\n\n[notes]\n",
    )
    .unwrap();
    std::fs::write(app_dir.join("keyboard.txt"), "Fn F1 F2\nCtrl Alt Space\n").unwrap();

    let loaded = wkey::config::load(None, Some(config_home.as_path())).unwrap();

    assert_eq!(
        loaded.keyboard_layout,
        vec![
            vec![
                KeyboardCell::Key("Fn".to_owned()),
                KeyboardCell::Gap(1),
                KeyboardCell::Key("F1".to_owned()),
                KeyboardCell::Gap(1),
                KeyboardCell::Key("F2".to_owned()),
            ],
            vec![
                KeyboardCell::Key("Ctrl".to_owned()),
                KeyboardCell::Gap(1),
                KeyboardCell::Key("Alt".to_owned()),
                KeyboardCell::Gap(1),
                KeyboardCell::Key("Space".to_owned()),
            ],
        ]
    );
}
