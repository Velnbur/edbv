use app::{App, Focus};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use miette::{IntoDiagnostic, Result};
use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};
use std::path::PathBuf;

mod app;

pub fn view(path: PathBuf) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::from_db_path(path)?;
    loop {
        terminal
            .draw(|frame| ui_draw(frame, &mut app))
            .into_diagnostic()?;
        if let Event::Key(key) = event::read().into_diagnostic()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if key.code == KeyCode::Char('q') {
                break;
            }

            if key.modifiers == KeyModifiers::CONTROL {
                if app.focus == Focus::Keys {
                    if key.code == KeyCode::Char('p') {
                        app.keys_widget_state.select_previous();
                    }
                    if key.code == KeyCode::Char('n') {
                        app.keys_widget_state.select_next();
                    }
                } else if app.focus == Focus::Value {
                    if key.code == KeyCode::Char('p') {
                        app.value_scroll = app.value_scroll.saturating_sub(1);
                        app.value_scroll_state.prev();
                    }
                    if key.code == KeyCode::Char('n') {
                        app.value_scroll = app.value_scroll.saturating_add(1);
                        app.value_scroll_state.next();
                    }
                }
            }

            if key.code == KeyCode::Tab {
                app.value_scroll = 0;
                app.value_scroll_state.first();
                app.focus.switch();
            }
        }
    }
    ratatui::restore();

    Ok(())
}

pub fn ui_draw(frame: &mut Frame, app: &mut App) {
    let whichkey_and_chunks = Layout::vertical([
        Constraint::Percentage(5),  // title
        Constraint::Percentage(80), // main content
        Constraint::Percentage(15), // usable keys help message
    ])
    .split(frame.area());

    let keys_and_values =
        Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(whichkey_and_chunks[1]);

    // Draw path to current file (and type) as title
    let title = Text::styled(
        format!("LevelDB ({})", app.path.to_str().unwrap()),
        Style::new().add_modifier(Modifier::BOLD),
    );
    frame.render_widget(title, whichkey_and_chunks[0]);

    // Draw keys to terminal
    let items: Vec<_> = app
        .keys
        .iter()
        .map(|k| ListItem::new(Line::from(Span::raw(k.clone()))))
        .collect();
    let mut block = Block::bordered().title("Keys");
    if app.focus == Focus::Keys {
        block = block.border_type(BorderType::Double);
    }

    let list_widget = List::new(items).block(block).highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
    );
    frame.render_stateful_widget(list_widget, keys_and_values[0], &mut app.keys_widget_state);

    // Draw value to terminal
    let mut block = Block::bordered().title("Value");
    if app.focus == Focus::Value {
        block = block.border_type(BorderType::Double);
    }

    let (content_type, current_value) = app.get_value_by_key_idx().unwrap_or_default();
    let lines_count = current_value.lines().count();
    app.value_scroll_state = app.value_scroll_state.content_length(lines_count);
    let value = Paragraph::new(current_value.clone())
        .block(block.title_bottom(content_type.to_line()))
        .scroll((app.value_scroll as u16, 0));
    frame.render_widget(value, keys_and_values[1]);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        keys_and_values[1].inner(Margin {
            vertical: 0,
            horizontal: 1,
        }),
        &mut app.value_scroll_state,
    );
}
