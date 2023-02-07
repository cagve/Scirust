use std::{time, thread};

use tui::{Frame, widgets::{Block, Borders, Clear, Paragraph}, backend::Backend, layout::{Layout, Direction, Constraint, Rect}, text::Span};

pub fn create_popup<B: Backend> (f: &mut Frame<B>, width:u16, height:u16, text:String, title:String) {
    let paragraph = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));
    let area = centered_rect(width, height, f.size());

    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
