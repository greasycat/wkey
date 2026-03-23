use assert_cmd::Command;
use predicates::prelude::*;
use std::ffi::OsString;
use std::fs;

#[test]
fn search_only_prints_selected_item_key_without_opening_main_tui() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");
    let fake_bin_dir = temp.path().join("bin");
    let fake_fzf = fake_bin_dir.join("fzf");
    fs::create_dir_all(config_dir.join("groups")).unwrap();
    fs::create_dir_all(&fake_bin_dir).unwrap();

    fs::write(
        config_dir.join("groups/shell.toml"),
        r#"
[shortcuts]
copy = { key = "Ctrl+C", desc = "Copy selection" }

[notes]
tip = { desc = "Remember this" }
"#,
    )
    .unwrap();

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

    let mut path_entries = vec![OsString::from(&fake_bin_dir)];
    if let Some(existing_path) = std::env::var_os("PATH") {
        path_entries.extend(std::env::split_paths(&existing_path).map(OsString::from));
    }
    let path = std::env::join_paths(path_entries).unwrap();

    Command::cargo_bin("wkey")
        .unwrap()
        .env("PATH", path)
        .arg("--config-dir")
        .arg(&config_dir)
        .arg("-S")
        .assert()
        .success()
        .stdout(predicate::eq("note\u{1f}shell\u{1f}tip\n"));
}
