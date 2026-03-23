use wkey::config::KeyboardCell;
use wkey::model::{Item, Note, Shortcut};
use wkey::ui::{render_to_string, render_to_string_with_layout};

#[test]
fn render_to_string_shows_items_without_alt_screen_escape() {
    let items = vec![Item::Shortcut(Shortcut::new(
        "copy",
        "Ctrl+C",
        "Copy selection",
        "shell",
    ))];
    let output = render_to_string(&items, None, 100, 22).unwrap();

    assert!(output.contains("Copy selection"));
    assert!(output.contains("Ctrl+C"));
    assert!(!output.contains("\u{1b}[?1049h"));
}

#[test]
fn render_to_string_uses_left_list_and_right_keyboard() {
    let items = vec![
        Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
        Item::Note(Note::new("tip", "Remember this", "shell")),
    ];

    let output = render_to_string(&items, Some("note\u{1f}shell\u{1f}tip"), 120, 24).unwrap();

    assert!(output.contains("Items"));
    assert!(output.contains("Keyboard"));
    assert!(output.contains("Remember this"));
    assert!(output.contains("Type: note"));
    assert!(output.contains("All"));
    assert!(output.contains("Tab for all filter"));
}

#[test]
fn render_to_string_uses_custom_keyboard_layout() {
    let items = vec![Item::Shortcut(Shortcut::new(
        "copy",
        "Fn+F1",
        "Copy selection",
        "shell",
    ))];
    let layout = vec![
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
    ];

    let output = render_to_string_with_layout(
        &layout,
        &items,
        Some("shortcut\u{1f}shell\u{1f}copy"),
        100,
        22,
    )
    .unwrap();

    assert!(output.contains("F1"));
    assert!(output.contains("Fn"));
    assert!(!output.contains("[Esc]"));
}
