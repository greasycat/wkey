use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

#[test]
fn help_mentions_subcommands_and_config_dir() {
    Command::cargo_bin("wkey")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("-s, --search"))
        .stdout(predicate::str::contains("-S, --search-only"))
        .stdout(predicate::str::contains("-C, --config-dir <PATH>"))
        .stdout(predicate::str::contains("-j, --json"))
        .stdout(predicate::str::contains("--search"))
        .stdout(predicate::str::contains("--search-only"))
        .stdout(predicate::str::contains("--config-dir"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("group files [aliases: g]"))
        .stdout(predicate::str::contains("shortcut entries [aliases: s]"))
        .stdout(predicate::str::contains("note entries [aliases: n]"))
        .stdout(predicate::str::contains("wkey g D shell"))
        .stdout(predicate::str::contains("group"))
        .stdout(predicate::str::contains("shortcut"))
        .stdout(predicate::str::contains("note"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("wkey g l"))
        .stdout(predicate::str::contains("wkey s l -a"))
        .stdout(predicate::str::contains("wkey --search"))
        .stdout(predicate::str::contains("wkey --search-only"))
        .stdout(predicate::str::contains("wkey shortcut list --json"));
}

#[test]
fn init_writes_default_app_config_keyboard_layout_and_group() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .arg("--config-dir")
        .arg(&config_dir)
        .arg("init")
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"))
        .stdout(predicate::str::contains("keyboard.txt"))
        .stdout(predicate::str::contains("groups"))
        .stdout(predicate::str::contains("groups/wkey.toml"));

    assert_eq!(
        fs::read_to_string(config_dir.join("config.toml")).unwrap(),
        wkey::config::DEFAULT_APP_CONFIG_TEXT
    );
    assert_eq!(
        fs::read_to_string(config_dir.join("keyboard.txt")).unwrap(),
        wkey::config::DEFAULT_KEYBOARD_LAYOUT_TEXT
    );
    assert_eq!(
        fs::read_to_string(config_dir.join("groups/wkey.toml")).unwrap(),
        wkey::config::DEFAULT_WKEY_GROUP_TEXT
    );
}

#[test]
fn group_and_item_crud_commands_update_group_files() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["init", "--yes"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "shortcut",
            "create",
            "--group",
            "shell",
            "copy",
            "--key",
            "Ctrl+C",
            "--desc",
            "Copy selection",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "shortcut\tshell\tcopy\tCtrl+C\tCopy selection",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "note",
            "create",
            "--group",
            "shell",
            "tip",
            "--desc",
            "Remember this",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("note\tshell\ttip\tRemember this"));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["shortcut", "list", "--group", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "shortcut\tshell\tcopy\tCtrl+C\tCopy selection",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["note", "list", "--group", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note\tshell\ttip\tRemember this"));

    let group_file = fs::read_to_string(config_dir.join("groups/shell.toml")).unwrap();
    assert!(group_file.contains("[shortcuts]"));
    assert!(group_file.contains("copy = { key = \"Ctrl+C\", desc = \"Copy selection\" }"));
    assert!(group_file.contains("[notes]"));
    assert!(group_file.contains("tip = { desc = \"Remember this\" }"));
}

#[test]
fn item_create_and_move_auto_create_destination_groups() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["init", "--yes"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "shortcut",
            "create",
            "--group",
            "shell",
            "copy",
            "--key",
            "Ctrl+C",
            "--desc",
            "Copy selection",
        ])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "note",
            "create",
            "--group",
            "shell",
            "tip",
            "--desc",
            "Remember this",
        ])
        .assert()
        .success();

    assert!(config_dir.join("groups/shell.toml").exists());

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "shortcut",
            "move",
            "--from-group",
            "shell",
            "--to-group",
            "editor",
            "copy",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "shortcut\teditor\tcopy\tCtrl+C\tCopy selection",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "note",
            "move",
            "--from-group",
            "shell",
            "--to-group",
            "notes",
            "tip",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("note\tnotes\ttip\tRemember this"));

    assert!(config_dir.join("groups/editor.toml").exists());
    assert!(config_dir.join("groups/notes.toml").exists());
}

#[test]
fn list_help_mentions_all_group_alias_and_examples() {
    Command::cargo_bin("wkey")
        .unwrap()
        .args(["shortcut", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-g, --group <GROUP>"))
        .stdout(predicate::str::contains("-a, --all"))
        .stdout(predicate::str::contains(
            "By default this includes every group",
        ))
        .stdout(predicate::str::contains("--all"))
        .stdout(predicate::str::contains("--group <GROUP>"));
}

#[test]
fn shortcut_list_all_matches_default_and_group_conflicts_with_all() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["init", "--yes"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "shortcut",
            "create",
            "--group",
            "shell",
            "copy",
            "--key",
            "Ctrl+C",
            "--desc",
            "Copy selection",
        ])
        .assert()
        .success();

    let default_output = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["shortcut", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let all_output = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["shortcut", "list", "--all"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(default_output, all_output);

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["shortcut", "list", "--group", "shell", "--all"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn json_output_works_for_group_and_item_commands() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["init", "--yes"])
        .assert()
        .success();

    let create_group = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["--json", "group", "create", "shell"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let create_group: Value = serde_json::from_slice(&create_group).unwrap();
    assert_eq!(create_group["name"], "shell");
    assert!(
        create_group["path"]
            .as_str()
            .unwrap()
            .ends_with("groups/shell.toml")
    );

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "--json",
            "shortcut",
            "create",
            "--group",
            "shell",
            "copy",
            "--key",
            "Ctrl+C",
            "--desc",
            "Copy selection",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\": \"shortcut\""))
        .stdout(predicate::str::contains("\"group\": \"shell\""));

    let list_output = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["--json", "shortcut", "list", "--all"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let items: Value = serde_json::from_slice(&list_output).unwrap();
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "copy")
    );

    let show_group = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["--json", "group", "show", "shell"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let group: Value = serde_json::from_slice(&show_group).unwrap();
    assert_eq!(group["name"], "shell");
    assert_eq!(group["items"].as_array().unwrap()[0]["id"], "copy");
}

#[test]
fn json_is_rejected_for_tui_and_init() {
    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "only supported for group, shortcut, and note",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--json", "init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "only supported for group, shortcut, and note",
        ));
}

#[test]
fn short_aliases_work_for_commands_and_args() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["i", "-y"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args([
            "s",
            "c",
            "-g",
            "shell",
            "copy",
            "-k",
            "Ctrl+C",
            "-d",
            "Copy selection",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "shortcut\tshell\tcopy\tCtrl+C\tCopy selection",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["n", "c", "-g", "shell", "tip", "-d", "Remember this"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note\tshell\ttip\tRemember this"));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["s", "l", "-g", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "shortcut\tshell\tcopy\tCtrl+C\tCopy selection",
        ));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["n", "m", "-f", "shell", "-t", "notes", "tip"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note\tnotes\ttip\tRemember this"));

    let json_output = Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["-j", "s", "l", "-a"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let payload: Value = serde_json::from_slice(&json_output).unwrap();
    assert!(
        payload
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "copy")
    );
}

#[test]
fn group_force_delete_removes_non_empty_group() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["i", "-y"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args([
            "s",
            "c",
            "-g",
            "shell",
            "copy",
            "-k",
            "Ctrl+C",
            "-d",
            "Copy selection",
        ])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["g", "d", "shell"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("group 'shell' is not empty"))
        .stderr(predicate::str::contains("wkey group force-delete shell"))
        .stderr(predicate::str::contains("wkey g D shell"));

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["-C"])
        .arg(&config_dir)
        .args(["g", "D", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("groups/shell.toml"));

    assert!(!config_dir.join("groups/shell.toml").exists());
}

#[test]
fn group_force_delete_supports_long_command_and_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["init", "--yes"])
        .assert()
        .success();

    Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args([
            "note",
            "create",
            "--group",
            "notes",
            "tip",
            "--desc",
            "Remember this",
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("wkey")
        .unwrap()
        .args(["--config-dir"])
        .arg(&config_dir)
        .args(["--json", "group", "force-delete", "notes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let payload: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(payload["name"], "notes");
    assert!(
        payload["path"]
            .as_str()
            .unwrap()
            .ends_with("groups/notes.toml")
    );
    assert!(!config_dir.join("groups/notes.toml").exists());
}

#[test]
fn parse_errors_include_contextual_examples_for_shortcut_commands() {
    Command::cargo_bin("wkey")
        .unwrap()
        .args(["shortcut", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains(
            "Try `wkey shortcut --help` for shortcut command usage.",
        ))
        .stderr(predicate::str::contains("Examples:"))
        .stderr(predicate::str::contains(
            "wkey shortcut create --group shell copy --key Ctrl+C --desc \"Copy selection\"",
        ))
        .stderr(predicate::str::contains(
            "wkey s c -g shell copy -k Ctrl+C -d \"Copy selection\"",
        ));
}

#[test]
fn parse_errors_include_contextual_examples_for_group_commands() {
    Command::cargo_bin("wkey")
        .unwrap()
        .args(["group", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'nope'"))
        .stderr(predicate::str::contains(
            "Try `wkey group --help` for group command usage.",
        ))
        .stderr(predicate::str::contains("wkey group force-delete shell"))
        .stderr(predicate::str::contains("wkey g D shell"));
}
