pub fn display_lines(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim_end_matches('\n');

    if trimmed.is_empty() {
        return vec![String::new()];
    }

    trimmed.split('\n').map(str::to_owned).collect()
}

pub fn single_line_preview(text: &str) -> String {
    let lines = display_lines(text);
    let first = lines.first().cloned().unwrap_or_default();

    if lines.len() > 1 {
        format!("{first}...")
    } else {
        first
    }
}

#[cfg(test)]
mod tests {
    use super::{display_lines, single_line_preview};

    #[test]
    fn display_lines_normalize_crlf_and_preserve_blank_lines() {
        assert_eq!(
            display_lines("First line\r\n\r\nSecond line\rThird line"),
            vec![
                "First line".to_owned(),
                String::new(),
                "Second line".to_owned(),
                "Third line".to_owned(),
            ]
        );
    }

    #[test]
    fn display_lines_trim_trailing_line_endings() {
        assert_eq!(display_lines("First line\n"), vec!["First line".to_owned()]);
        assert_eq!(
            display_lines("First line\r\n\r\n"),
            vec!["First line".to_owned()]
        );
    }

    #[test]
    fn single_line_preview_uses_first_line_and_marks_hidden_content() {
        assert_eq!(single_line_preview("First line"), "First line");
        assert_eq!(
            single_line_preview("First line\nSecond line"),
            "First line..."
        );
        assert_eq!(
            single_line_preview("First line\r\n\r\nSecond line"),
            "First line..."
        );
    }
}
