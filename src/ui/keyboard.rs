use crate::config::KeyboardCell;
use crate::model::Shortcut;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashSet;

pub fn build_keyboard_lines_with_layout(
    layout: &[Vec<KeyboardCell>],
    selected_shortcut: Option<&Shortcut>,
) -> Vec<Line<'static>> {
    let mut selected = HashSet::new();

    if let Some(shortcut) = selected_shortcut {
        selected.insert(normalize_key(&shortcut.key));
        let parts = split_key_parts(&shortcut.key).collect::<Vec<_>>();
        let has_modifier_separator = parts.len() > 1;
        for (index, part) in parts.iter().enumerate() {
            selected.insert(normalize_key_part(
                part,
                has_modifier_separator && index == 0,
            ));
        }
    }

    layout
        .iter()
        .map(|row| {
            let mut spans = Vec::new();
            for cell in row {
                match cell {
                    KeyboardCell::Key(label) => {
                        let normalized = normalize_key(label);
                        let style = key_style(selected.contains(&normalized));
                        spans.push(Span::styled(keycap(label, width_for(label)), style));
                    }
                    KeyboardCell::Gap(width) => spans.push(Span::raw(" ".repeat(*width))),
                }
            }
            Line::from(spans)
        })
        .collect()
}

fn key_style(is_selected: bool) -> Style {
    if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    }
}

fn keycap(label: &str, inner_width: usize) -> String {
    format!("{label:^inner_width$}", inner_width = inner_width)
}

fn width_for(label: &str) -> usize {
    match label {
        "Backspace" => 10,
        "Caps" => 8,
        "Ctrl" | "Meta" | "Cmd" => 6,
        "Enter" => 8,
        "Shift" => 9,
        "Space" => 22,
        "Tab" => 6,
        _ => label.len().max(3),
    }
}

fn split_key_parts(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|ch: char| ch == '+' || ch == '-' || ch == ',' || ch.is_whitespace())
        .map(str::trim)
        .filter(|part| !part.is_empty())
}

fn normalize_key_part(value: &str, is_leading_modifier: bool) -> String {
    if is_leading_modifier && value.eq_ignore_ascii_case("c") {
        return "CTRL".to_owned();
    }

    normalize_key(value)
}

fn normalize_key(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "CMD" | "COMMAND" | "SUPER" => "META".to_owned(),
        "CONTROL" => "CTRL".to_owned(),
        "OPTION" => "ALT".to_owned(),
        "RETURN" => "ENTER".to_owned(),
        "ESCAPE" => "ESC".to_owned(),
        " " => "SPACE".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_key_part, split_key_parts};

    #[test]
    fn split_key_parts_supports_multiple_separator_styles() {
        let parts = split_key_parts("Ctrl-Shift, Tab+Space\tCmd").collect::<Vec<_>>();

        assert_eq!(parts, vec!["Ctrl", "Shift", "Tab", "Space", "Cmd"]);
    }

    #[test]
    fn normalize_key_part_maps_leading_c_to_control() {
        assert_eq!(normalize_key_part("c", true), "CTRL");
        assert_eq!(normalize_key_part("C", true), "CTRL");
    }

    #[test]
    fn normalize_key_part_keeps_plain_c_as_letter_key() {
        assert_eq!(normalize_key_part("c", false), "C");
    }
}
