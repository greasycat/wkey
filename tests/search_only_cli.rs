use assert_cmd::Command;
use predicates::prelude::*;
use std::ffi::OsString;
use std::fs;

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[test]
fn search_only_prints_selected_item_desc_without_opening_main_tui() {
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
        .stdout(predicate::eq("Remember this\n"));
}

#[test]
fn search_only_also_pipes_selected_desc_to_pipeout_command() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");
    let fake_bin_dir = temp.path().join("bin");
    let fake_fzf = fake_bin_dir.join("fzf");
    let pipeout_path = temp.path().join("pipeout.txt");
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
        config_dir.join("config.toml"),
        format!(
            "[pipeout]\ncommand = \"cat > {}\"\n",
            shell_quote(&pipeout_path.display().to_string())
        ),
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
        .stdout(predicate::eq("Remember this\n"));

    assert_eq!(fs::read_to_string(pipeout_path).unwrap(), "Remember this");
}
