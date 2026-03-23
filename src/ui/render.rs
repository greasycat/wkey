use crate::config::{APP_TITLE, KeyboardCell};
use crate::model::{AppView, Item, ItemKind, Shortcut};
use crate::ui::keyboard::build_keyboard_lines_with_layout;
use anyhow::Result;
use crossterm::cursor::{MoveToColumn, position};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::{CrosstermBackend, TestBackend};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use std::collections::BTreeSet;
use std::io;
use std::time::Duration;

const STATIC_HEIGHT: u16 = 22;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ActiveFilter {
    #[default]
    Text,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectorAction {
    Continue,
    Accept(String),
    Cancel,
}

#[derive(Debug, Default)]
struct AppState {
    query: String,
    group_query: String,
    active_filter: ActiveFilter,
    selected_index: usize,
    list_offset: usize,
}

impl AppState {
    fn new(items: &[Item], selected_id: Option<&str>) -> Self {
        let mut state = Self::default();
        if let Some(id) = selected_id {
            if let Some(index) = items.iter().position(|item| item.selection_key() == id) {
                state.selected_index = index;
            }
        }
        state
    }

    fn filtered_items(&self, items: &[Item]) -> Vec<Item> {
        items
            .iter()
            .filter(|item| item.matches_query(&self.query) && self.matches_group(item))
            .cloned()
            .collect()
    }

    fn matching_groups(&self, items: &[Item]) -> Vec<String> {
        items
            .iter()
            .map(Item::group)
            .filter(|group| self.matches_group_name(group))
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn matches_group(&self, item: &Item) -> bool {
        self.matches_group_name(item.group())
    }

    fn matches_group_name(&self, group_name: &str) -> bool {
        let terms = self.group_terms();
        if terms.is_empty() {
            return true;
        }

        let group = group_name.to_ascii_lowercase();
        terms.iter().all(|term| fuzzy_matches(&group, term))
    }

    fn group_terms(&self) -> Vec<String> {
        self.group_query
            .split_whitespace()
            .map(|term| term.to_ascii_lowercase())
            .collect()
    }

    fn selected_selection_key(&self, filtered: &[Item]) -> Option<String> {
        filtered.get(self.selected_index).map(Item::selection_key)
    }

    fn selected_selection_key_for_active_list(
        &self,
        filtered: &[Item],
        matching_groups: &[String],
    ) -> Option<String> {
        match self.active_filter {
            ActiveFilter::Text => self.selected_selection_key(filtered),
            ActiveFilter::Group => matching_groups.get(self.selected_index).and_then(|group| {
                filtered
                    .iter()
                    .find(|item| item.group() == group)
                    .map(Item::selection_key)
            }),
        }
    }

    fn active_list_len(&self, filtered_len: usize, matching_groups_len: usize) -> usize {
        match self.active_filter {
            ActiveFilter::Text => filtered_len,
            ActiveFilter::Group => matching_groups_len,
        }
    }

    fn sync_selection(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.selected_index = 0;
            self.list_offset = 0;
            return;
        }

        if self.selected_index >= filtered_len {
            self.selected_index = filtered_len - 1;
        }
    }

    fn move_selection(&mut self, delta: isize, filtered_len: usize) {
        self.sync_selection(filtered_len);
        if filtered_len == 0 {
            return;
        }

        let next = (self.selected_index as isize + delta).clamp(0, filtered_len as isize - 1);
        self.selected_index = next as usize;
    }

    fn push_char(&mut self, ch: char) {
        self.active_filter_string_mut().push(ch);
        self.selected_index = 0;
        self.list_offset = 0;
    }

    fn push_str(&mut self, value: &str) {
        self.active_filter_string_mut().push_str(value);
        self.selected_index = 0;
        self.list_offset = 0;
    }

    fn backspace(&mut self) {
        self.active_filter_string_mut().pop();
        self.selected_index = 0;
        self.list_offset = 0;
    }

    fn toggle_filter_mode(&mut self) {
        self.active_filter = match self.active_filter {
            ActiveFilter::Text => ActiveFilter::Group,
            ActiveFilter::Group => ActiveFilter::Text,
        };
    }

    fn active_filter_string_mut(&mut self) -> &mut String {
        match self.active_filter {
            ActiveFilter::Text => &mut self.query,
            ActiveFilter::Group => &mut self.group_query,
        }
    }

    fn current_group_label(&self) -> &str {
        if self.group_query.trim().is_empty() {
            "All"
        } else {
            self.group_query.trim()
        }
    }

    fn build_list_state(&self, filtered_len: usize) -> ListState {
        if filtered_len == 0 {
            ListState::default()
        } else {
            ListState::default()
                .with_offset(self.list_offset)
                .with_selected(Some(self.selected_index))
        }
    }
}

fn fuzzy_matches(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle_chars = needle.chars();
    let mut current = match needle_chars.next() {
        Some(ch) => ch,
        None => return true,
    };

    for candidate in haystack.chars() {
        if candidate == current {
            match needle_chars.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }

    false
}

fn format_note_label(note_id: &str) -> String {
    let normalized = note_id.replace('-', " ");
    let mut chars = normalized.chars();
    match chars.next() {
        Some(first) => {
            let mut label = String::new();
            label.extend(first.to_uppercase());
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}

pub fn render_to_string(
    items: &[Item],
    selected_id: Option<&str>,
    width: u16,
    height: u16,
) -> Result<String> {
    render_to_string_with_layout(
        &crate::config::default_keyboard_layout(),
        items,
        selected_id,
        width,
        height,
    )
}

pub fn render_to_string_with_layout(
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
    width: u16,
    height: u16,
) -> Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    let mut app = AppState::new(items, selected_id);
    let filtered = app.filtered_items(items);
    let matching_groups = app.matching_groups(items);
    let active_list_len = app.active_list_len(filtered.len(), matching_groups.len());
    app.sync_selection(active_list_len);
    let mut list_state = app.build_list_state(active_list_len);
    let selected_key = app.selected_selection_key_for_active_list(&filtered, &matching_groups);
    terminal.draw(|frame| {
        draw_app(
            frame,
            AppView {
                items: &filtered,
                selected_id: selected_key.as_deref(),
            },
            keyboard_layout,
            &app.query,
            &app.group_query,
            app.current_group_label(),
            &matching_groups,
            app.active_filter,
            &mut list_state,
        )
    })?;

    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let line = (0..width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned();
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

pub fn render_inline(items: &[Item], selected_id: Option<&str>) -> Result<()> {
    render_inline_with_layout(
        &crate::config::default_keyboard_layout(),
        items,
        selected_id,
    )
}

pub fn render_inline_with_layout(
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
) -> Result<()> {
    if !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        let width = crossterm::terminal::size()
            .map(|(width, _)| width.max(80))
            .unwrap_or(120);
        print!(
            "{}",
            render_to_string_with_layout(
                keyboard_layout,
                items,
                selected_id,
                width,
                STATIC_HEIGHT,
            )?
        );
        io::Write::flush(&mut io::stdout())?;
        return Ok(());
    }

    run_interactive(keyboard_layout, items, selected_id)
}

pub fn select_item_with_layout(
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
) -> Result<Option<String>> {
    if items.is_empty() || !crossterm::tty::IsTty::is_tty(&io::stdout()) {
        return Ok(None);
    }

    run_interactive_selector(keyboard_layout, items, selected_id)
}

fn run_interactive(
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
) -> Result<()> {
    let viewport = interactive_viewport(STATIC_HEIGHT)?;
    reserve_viewport_lines(viewport.height.saturating_sub(1))?;
    enable_raw_mode()?;
    let options = TerminalOptions {
        viewport: Viewport::Fixed(viewport),
    };
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(backend, options)?;
    let result = run_app(
        &mut terminal,
        keyboard_layout,
        items,
        selected_id,
        viewport.y,
    );

    let restore_result = (|| -> Result<()> {
        terminal.show_cursor()?;
        execute!(terminal.backend_mut(), MoveToColumn(0), Print("\n"))?;
        disable_raw_mode()?;
        Ok(())
    })();

    result.and(restore_result)
}

fn run_interactive_selector(
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
) -> Result<Option<String>> {
    let viewport = interactive_viewport(STATIC_HEIGHT)?;
    reserve_viewport_lines(viewport.height.saturating_sub(1))?;
    enable_raw_mode()?;
    let options = TerminalOptions {
        viewport: Viewport::Fixed(viewport),
    };
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::with_options(backend, options)?;
    let result = run_selector_app(
        &mut terminal,
        keyboard_layout,
        items,
        selected_id,
        viewport.y,
    );

    let restore_result = (|| -> Result<()> {
        terminal.show_cursor()?;
        execute!(terminal.backend_mut(), MoveToColumn(0), Print("\n"))?;
        disable_raw_mode()?;
        Ok(())
    })();

    match (result, restore_result) {
        (Ok(selection), Ok(())) => Ok(selection),
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
    viewport_top: u16,
) -> Result<()> {
    let mut app = AppState::new(items, selected_id);
    let mut viewport_top = viewport_top;

    loop {
        let filtered = app.filtered_items(items);
        let matching_groups = app.matching_groups(items);
        let active_list_len = app.active_list_len(filtered.len(), matching_groups.len());
        app.sync_selection(active_list_len);
        let mut list_state = app.build_list_state(active_list_len);
        let selected_key = app.selected_selection_key_for_active_list(&filtered, &matching_groups);

        terminal.draw(|frame| {
            draw_app(
                frame,
                AppView {
                    items: &filtered,
                    selected_id: selected_key.as_deref(),
                },
                keyboard_layout,
                &app.query,
                &app.group_query,
                app.current_group_label(),
                &matching_groups,
                app.active_filter,
                &mut list_state,
            )
        })?;
        app.list_offset = list_state.offset();

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if should_quit(&app, key) {
                    return Ok(());
                }
                handle_key_event(&mut app, key, active_list_len);
            }
            Event::Paste(text) => app.push_str(&text),
            Event::Resize(_, _) => {
                let viewport = resized_viewport(viewport_top, STATIC_HEIGHT)?;
                viewport_top = viewport.y;
                terminal.resize(viewport)?;
            }
            _ => {}
        }
    }
}

fn run_selector_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    keyboard_layout: &[Vec<KeyboardCell>],
    items: &[Item],
    selected_id: Option<&str>,
    viewport_top: u16,
) -> Result<Option<String>> {
    let mut app = AppState::new(items, selected_id);
    let mut viewport_top = viewport_top;

    loop {
        let filtered = app.filtered_items(items);
        let matching_groups = app.matching_groups(items);
        let active_list_len = app.active_list_len(filtered.len(), matching_groups.len());
        app.sync_selection(active_list_len);
        let mut list_state = app.build_list_state(active_list_len);
        let selected_key = app.selected_selection_key_for_active_list(&filtered, &matching_groups);

        terminal.draw(|frame| {
            draw_app(
                frame,
                AppView {
                    items: &filtered,
                    selected_id: selected_key.as_deref(),
                },
                keyboard_layout,
                &app.query,
                &app.group_query,
                app.current_group_label(),
                &matching_groups,
                app.active_filter,
                &mut list_state,
            )
        })?;
        app.list_offset = list_state.offset();

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match handle_selector_key_event(&mut app, key, &filtered, &matching_groups) {
                    SelectorAction::Continue => {}
                    SelectorAction::Accept(selection) => return Ok(Some(selection)),
                    SelectorAction::Cancel => return Ok(None),
                }
            }
            Event::Paste(text) => app.push_str(&text),
            Event::Resize(_, _) => {
                let viewport = resized_viewport(viewport_top, STATIC_HEIGHT)?;
                viewport_top = viewport.y;
                terminal.resize(viewport)?;
            }
            _ => {}
        }
    }
}

fn reserve_viewport_lines(line_count: u16) -> Result<()> {
    let mut stdout = io::stdout();
    for _ in 0..line_count {
        execute!(stdout, Print("\n"))?;
    }
    Ok(())
}

fn interactive_viewport(viewport_height: u16) -> Result<Rect> {
    let (width, height) = crossterm::terminal::size()?;
    let (_, cursor_row) = position().unwrap_or((0, height.saturating_sub(viewport_height)));
    Ok(calculate_viewport(
        width,
        height,
        cursor_row,
        viewport_height,
    ))
}

fn resized_viewport(current_top: u16, viewport_height: u16) -> Result<Rect> {
    let (width, height) = crossterm::terminal::size()?;
    let viewport_height = viewport_height.min(height.max(1));
    Ok(Rect::new(
        0,
        current_top.min(height.saturating_sub(viewport_height)),
        width.max(1),
        viewport_height,
    ))
}

fn calculate_viewport(
    terminal_width: u16,
    terminal_height: u16,
    cursor_row: u16,
    viewport_height: u16,
) -> Rect {
    let viewport_height = viewport_height.min(terminal_height.max(1));
    let top = cursor_row.min(terminal_height.saturating_sub(viewport_height));
    Rect::new(0, top, terminal_width.max(1), viewport_height)
}

fn should_quit(app: &AppState, key: KeyEvent) -> bool {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => app.active_filter != ActiveFilter::Group,
        KeyEvent {
            code: KeyCode::Char('c') | KeyCode::Char('C'),
            modifiers,
            ..
        } => modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER),
        _ => false,
    }
}

fn handle_selector_key_event(
    app: &mut AppState,
    key: KeyEvent,
    filtered: &[Item],
    matching_groups: &[String],
) -> SelectorAction {
    match key {
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => app
            .selected_selection_key_for_active_list(filtered, matching_groups)
            .map(SelectorAction::Accept)
            .unwrap_or(SelectorAction::Continue),
        KeyEvent {
            code: KeyCode::Esc, ..
        } => SelectorAction::Cancel,
        KeyEvent {
            code: KeyCode::Char('c') | KeyCode::Char('C'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL)
            || modifiers.contains(KeyModifiers::SUPER) =>
        {
            SelectorAction::Cancel
        }
        _ => {
            let active_list_len = app.active_list_len(filtered.len(), matching_groups.len());
            handle_key_event(app, key, active_list_len);
            SelectorAction::Continue
        }
    }
}

fn handle_key_event(app: &mut AppState, key: KeyEvent, filtered_len: usize) {
    match key.code {
        KeyCode::Tab => app.toggle_filter_mode(),
        KeyCode::Backspace => app.backspace(),
        KeyCode::Char('n') | KeyCode::Char('N')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.move_selection(1, filtered_len);
        }
        KeyCode::Char('p') | KeyCode::Char('P')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.move_selection(-1, filtered_len);
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.move_selection(5, filtered_len);
        }
        KeyCode::Char('u') | KeyCode::Char('U')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.move_selection(-5, filtered_len);
        }
        KeyCode::Char(ch)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
                && !key.modifiers.contains(KeyModifiers::SUPER) =>
        {
            app.push_char(ch);
        }
        KeyCode::Up => app.move_selection(-1, filtered_len),
        KeyCode::Down => app.move_selection(1, filtered_len),
        KeyCode::PageUp => app.move_selection(-5, filtered_len),
        KeyCode::PageDown => app.move_selection(5, filtered_len),
        KeyCode::Home => {
            app.selected_index = 0;
        }
        KeyCode::End => {
            if filtered_len > 0 {
                app.selected_index = filtered_len - 1;
            }
        }
        _ => {}
    }
}

fn draw_app(
    frame: &mut Frame<'_>,
    app: AppView<'_>,
    keyboard_layout: &[Vec<KeyboardCell>],
    query: &str,
    group_query: &str,
    group_label: &str,
    matching_groups: &[String],
    active_filter: ActiveFilter,
    list_state: &mut ListState,
) {
    let outer = Block::default()
        .title(format!(" {} ", APP_TITLE))
        .borders(Borders::ALL);
    let area = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let [filter_area, content_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);
    let [list_area, right_area] =
        Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)])
            .areas(content_area);
    let [detail_area, keyboard_area] =
        Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).areas(right_area);

    let group_width = (group_label.chars().count().max(group_query.chars().count()) as u16)
        .saturating_add(4)
        .clamp(16, filter_area.width.saturating_sub(8).max(16));
    let [group_filter_area, text_filter_area] =
        Layout::horizontal([Constraint::Length(group_width), Constraint::Min(12)])
            .areas(filter_area);

    let filter_line = if query.is_empty() {
        Line::from(vec![
            Span::styled("Type to filter items", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("Tab for all filter", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("Ctrl-C to quit", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(query.to_owned())
    };
    let filter = Paragraph::new(filter_line).block(
        Block::default()
            .title(" Filter ")
            .borders(Borders::ALL)
            .border_style(match active_filter {
                ActiveFilter::Text => Style::default().fg(Color::Yellow),
                ActiveFilter::Group => Style::default(),
            }),
    );
    frame.render_widget(filter, text_filter_area);

    let group_filter_line = if group_query.is_empty() {
        Line::from(group_label.to_owned())
    } else {
        Line::from(group_query.to_owned())
    };
    let group_filter = Paragraph::new(group_filter_line).block(
        Block::default()
            .title(" Group ")
            .borders(Borders::ALL)
            .border_style(match active_filter {
                ActiveFilter::Group => Style::default().fg(Color::Yellow),
                ActiveFilter::Text => Style::default(),
            }),
    );
    frame.render_widget(group_filter, group_filter_area);

    let cursor = match active_filter {
        ActiveFilter::Text => Position::new(
            text_filter_area
                .x
                .saturating_add(1 + query.chars().count() as u16),
            text_filter_area.y.saturating_add(1),
        ),
        ActiveFilter::Group => Position::new(
            group_filter_area
                .x
                .saturating_add(1 + group_query.chars().count() as u16),
            group_filter_area.y.saturating_add(1),
        ),
    };
    frame.set_cursor_position(cursor);

    match active_filter {
        ActiveFilter::Group => {
            let groups = if matching_groups.is_empty() {
                vec![ListItem::new(Line::from(vec![Span::styled(
                    "No groups match the current filter.",
                    Style::default().fg(Color::DarkGray),
                )]))]
            } else {
                matching_groups
                    .iter()
                    .map(|group| ListItem::new(Line::from(group.clone())))
                    .collect::<Vec<_>>()
            };
            let groups_list = List::new(groups)
                .block(Block::default().title(" Groups ").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol("> ");
            frame.render_stateful_widget(groups_list, list_area, list_state);
        }
        ActiveFilter::Text => {
            let items = if app.items.is_empty() {
                vec![ListItem::new(Line::from(vec![Span::styled(
                    "No items match the current filter.",
                    Style::default().fg(Color::DarkGray),
                )]))]
            } else {
                app.items
                    .iter()
                    .map(|item| {
                        let (lead, lead_style) = match item.kind() {
                            ItemKind::Shortcut => (
                                format!("{:<18}", item.key_combo().unwrap_or("")),
                                Style::default().fg(Color::Magenta),
                            ),
                            ItemKind::Note => (
                                format!("{:<18}", format_note_label(item.id())),
                                Style::default(),
                            ),
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(lead, lead_style),
                            Span::raw(item.desc().to_owned()),
                        ]))
                    })
                    .collect::<Vec<_>>()
            };

            let items_list = List::new(items)
                .block(Block::default().title(" Items ").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
                .highlight_symbol("> ");
            frame.render_stateful_widget(items_list, list_area, list_state);
        }
    }

    let detail = Paragraph::new(selected_detail(app))
        .wrap(Wrap { trim: false })
        .block(Block::default().title(" Selection ").borders(Borders::ALL));
    frame.render_widget(detail, detail_area);

    let keyboard = Paragraph::new(build_keyboard_lines_with_layout(
        keyboard_layout,
        selected_shortcut(app.selected()),
    ))
    .wrap(Wrap { trim: false })
    .block(Block::default().title(" Keyboard ").borders(Borders::ALL));
    frame.render_widget(keyboard, keyboard_area);
}

fn selected_shortcut(item: Option<&Item>) -> Option<&Shortcut> {
    match item {
        Some(Item::Shortcut(shortcut)) => Some(shortcut),
        _ => None,
    }
}

fn selected_detail(app: AppView<'_>) -> Vec<Line<'static>> {
    match app.selected() {
        Some(Item::Shortcut(shortcut)) => vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(shortcut.id.clone(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(format!("Type: shortcut")),
            Line::from(format!("Group: {}", shortcut.group)),
            Line::from(format!("Key: {}", shortcut.key)),
            Line::from(shortcut.desc.clone()),
        ],
        Some(Item::Note(note)) => vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(note.id.clone(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from("Type: note"),
            Line::from(format!("Group: {}", note.group)),
            Line::from(note.desc.clone()),
        ],
        None => vec![
            Line::from("Selected: none"),
            Line::from("Keep typing to narrow the list."),
            Line::from("Use Up/Down, Ctrl-N/Ctrl-P, or Ctrl-D/Ctrl-U."),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveFilter, AppState, SelectorAction, calculate_viewport, format_note_label,
        handle_key_event, handle_selector_key_event, should_quit,
    };
    use crate::model::{Item, Note, Shortcut};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::layout::Rect;

    #[test]
    fn filtered_items_follow_query() {
        let items = vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Use !!", "shell")),
        ];
        let mut app = AppState::default();
        app.push_str("note");

        let filtered = app.filtered_items(&items);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id(), "tip");
    }

    #[test]
    fn filtered_items_follow_group_query() {
        let items = vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Use !!", "wkey")),
        ];
        let app = AppState {
            group_query: "wk".to_owned(),
            ..AppState::default()
        };

        let filtered = app.filtered_items(&items);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].group(), "wkey");
    }

    #[test]
    fn matching_groups_follow_group_query() {
        let items = vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Use !!", "wkey")),
            Item::Note(Note::new("wm", "Manage windows", "window-management")),
        ];
        let app = AppState {
            group_query: "wk".to_owned(),
            active_filter: ActiveFilter::Group,
            ..AppState::default()
        };

        let groups = app.matching_groups(&items);

        assert_eq!(groups, vec!["wkey"]);
    }

    #[test]
    fn matching_groups_with_empty_query_show_all_groups() {
        let items = vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Use !!", "wkey")),
            Item::Note(Note::new("paste", "Paste", "shell")),
        ];
        let app = AppState {
            active_filter: ActiveFilter::Group,
            ..AppState::default()
        };

        let groups = app.matching_groups(&items);

        assert_eq!(groups, vec!["shell", "wkey"]);
    }

    #[test]
    fn group_mode_selection_uses_matching_group_index() {
        let filtered = vec![
            Item::Shortcut(Shortcut::new("copy", "Ctrl+C", "Copy selection", "shell")),
            Item::Note(Note::new("tip", "Use !!", "wkey")),
        ];
        let groups = vec!["shell".to_owned(), "wkey".to_owned()];
        let app = AppState {
            active_filter: ActiveFilter::Group,
            selected_index: 1,
            ..AppState::default()
        };

        let selected = app.selected_selection_key_for_active_list(&filtered, &groups);

        assert_eq!(selected.as_deref(), Some("note\u{1f}wkey\u{1f}tip"));
    }

    #[test]
    fn format_note_label_removes_dashes_and_capitalizes_first_letter() {
        assert_eq!(format_note_label("prompt-tip"), "Prompt tip");
    }

    #[test]
    fn group_mode_with_empty_query_shows_all_items() {
        let items = vec![Item::Shortcut(Shortcut::new(
            "copy",
            "Ctrl+C",
            "Copy selection",
            "shell",
        ))];
        let app = AppState {
            active_filter: ActiveFilter::Group,
            ..AppState::default()
        };

        let filtered = app.filtered_items(&items);

        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn group_filter_uses_fuzzy_matching() {
        let items = vec![Item::Shortcut(Shortcut::new(
            "copy",
            "Ctrl+C",
            "Copy selection",
            "window-management",
        ))];
        let app = AppState {
            group_query: "wdmg".to_owned(),
            active_filter: ActiveFilter::Group,
            ..AppState::default()
        };

        let filtered = app.filtered_items(&items);

        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn empty_group_filter_shows_all_groups_label() {
        let app = AppState::default();

        assert_eq!(app.current_group_label(), "All");
    }

    #[test]
    fn esc_does_not_quit_while_editing_group_filter() {
        let app = AppState {
            active_filter: ActiveFilter::Group,
            ..AppState::default()
        };

        assert!(!should_quit(
            &app,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
        ));
    }

    #[test]
    fn sync_selection_clamps_to_filtered_results() {
        let mut app = AppState {
            query: String::new(),
            group_query: String::new(),
            active_filter: ActiveFilter::Text,
            selected_index: 4,
            list_offset: 0,
        };

        app.sync_selection(2);

        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn ctrl_d_moves_selection_down_by_a_page() {
        let mut app = AppState::default();

        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            12,
        );

        assert_eq!(app.selected_index, 5);
    }

    #[test]
    fn ctrl_u_moves_selection_up_by_a_page() {
        let mut app = AppState {
            selected_index: 7,
            ..AppState::default()
        };

        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            12,
        );

        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn selector_accepts_current_item_on_enter() {
        let mut app = AppState::default();
        let filtered = vec![Item::Shortcut(Shortcut::new(
            "copy",
            "Ctrl+C",
            "Copy selection",
            "shell",
        ))];
        let groups = vec!["shell".to_owned()];

        let action = handle_selector_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &filtered,
            &groups,
        );

        assert_eq!(
            action,
            SelectorAction::Accept("shortcut\u{1f}shell\u{1f}copy".to_owned())
        );
    }

    #[test]
    fn selector_cancels_on_escape() {
        let mut app = AppState::default();
        let filtered = vec![Item::Shortcut(Shortcut::new(
            "copy",
            "Ctrl+C",
            "Copy selection",
            "shell",
        ))];
        let groups = vec!["shell".to_owned()];

        let action = handle_selector_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            &filtered,
            &groups,
        );

        assert_eq!(action, SelectorAction::Cancel);
    }

    #[test]
    fn calculate_viewport_starts_at_cursor_when_there_is_room_below() {
        let viewport = calculate_viewport(120, 40, 5, 22);

        assert_eq!(viewport, Rect::new(0, 5, 120, 22));
    }

    #[test]
    fn calculate_viewport_scrolls_up_when_cursor_is_near_bottom() {
        let viewport = calculate_viewport(120, 24, 23, 22);

        assert_eq!(viewport, Rect::new(0, 2, 120, 22));
    }
}
