use anyhow::{Context, Result, ensure};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn write_to_command(command: &str, input: &str) -> Result<()> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn pipeout command '{command}'"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .with_context(|| format!("failed to write note text to pipeout command '{command}'"))?;
    }

    let status = child
        .wait()
        .with_context(|| format!("failed waiting for pipeout command '{command}'"))?;
    ensure!(
        status.success(),
        "pipeout command '{}' exited with status {}",
        command,
        status
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_to_command;

    fn shell_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }

    #[test]
    fn write_to_command_pipes_input_to_stdin() {
        let temp = tempfile::tempdir().unwrap();
        let output_path = temp.path().join("note.txt");
        let command = format!("cat > {}", shell_quote(&output_path.display().to_string()));

        write_to_command(&command, "Remember this note").unwrap();

        assert_eq!(
            std::fs::read_to_string(output_path).unwrap(),
            "Remember this note"
        );
    }
}
