use crate::tui::keybinds::KEYBINDS;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Modifier, Color};
use ratatui::text::{Span, Line, Text};
use ratatui::Frame;
use crate::tui::app::state::App;

pub fn draw_keybinds_modal(
    f: &mut Frame,
    area: Rect,
    _app: &App,
) {
    // Centered modal size
    let width = 48;
    let height = (KEYBINDS.len() as u16 + 4).min(area.height - 4);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect { x, y, width, height };
    f.render_widget(Clear, modal_area);

    let mut text = Text::from("");
    for kb in KEYBINDS {
        text.lines.push(Line::from(vec![
            Span::styled(format!("{:<12}", kb.key), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::raw(kb.description),
        ]));
    }
    let block = Block::default()
        .title(" Keybinds ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(para, modal_area);
}
