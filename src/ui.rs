use crate::app::{ActiveView, App};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Odśwież cache szczegółów, jeśli zmieniło się zaznaczenie.
    app.refresh_details();

    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)])
        .split(size);

    match app.active_view {
        ActiveView::ListAndDetails => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(main_chunks[0]);

            let title_text = if app.is_searching {
                format!(" Szukaj: {}█ ", app.search_query)
            } else if !app.search_query.is_empty() {
                format!(" Lista (Filtr: {}) [ESC by wyczyścić] ", app.search_query)
            } else {
                " Historia rozmów AGY [/ - szukaj] ".to_string()
            };

            let list_items: Vec<ListItem> = app
                .filtered_items
                .iter()
                .map(|i| {
                    ListItem::new(format!(
                        "{} | {} (Kroki: {})",
                        i.date_str, i.title, i.steps
                    ))
                })
                .collect();

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(title_text))
                .highlight_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let details_widget = Paragraph::new(app.details_cache.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Szczegóły / Podgląd "),
                )
                .wrap(Wrap { trim: true })
                .scroll((app.details_scroll, 0))
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(details_widget, chunks[1]);
        }
        ActiveView::Transcript => {
            let transcript_widget = Paragraph::new(app.transcript_lines.join("\n"))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Pełny Transkrypt Sesji (transcript.jsonl) [ESC/T = powrót] "),
                )
                .wrap(Wrap { trim: true })
                .scroll((app.transcript_scroll, 0))
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(transcript_widget, main_chunks[0]);
        }
    }

    let help_text = match app.active_view {
        ActiveView::ListAndDetails => {
            if app.is_searching {
                " Wpisz frazę i naciśnij [Enter] aby zatwierdzić, [ESC] aby anulować "
            } else {
                " [↑/↓/Home/End] Nawigacja | [/] Szukaj | [T] Transkrypt | [Enter] Wybierz | [Q] Wyjdź "
            }
        }
        ActiveView::Transcript => {
            " [↑/↓/PageUp/PageDown] Przewiń transkrypt | [ESC / T] Powrót do listy "
        }
    };

    let help_bar =
        Paragraph::new(help_text).style(Style::default().fg(Color::Black).bg(Color::DarkGray));
    f.render_widget(help_bar, main_chunks[1]);
}
