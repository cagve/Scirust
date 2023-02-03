mod bib;

use bib::Author;
use biblatex::ChunksExt;
use regex::Regex;
use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use tui_textarea::{Input, Key, TextArea};

enum InputMode {
    Normal,
    Insert,
}

struct App {
    input: String,
    mode: InputMode,
    log: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            mode: InputMode::Insert,
            log: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = App::default();
    let mut input = TextArea::default();
    let res = run_app(&mut terminal, app, input);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    return Ok(());
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut textarea: TextArea,
) -> io::Result<()> {
    let author_list = bib::get_authors("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string());
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let leftchunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[0]);

            match app.mode {
                InputMode::Insert => {
                    textarea
                        .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                    let b = textarea
                        .block()
                        .cloned()
                        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
                    textarea.set_block(
                        b.style(Style::default())
                            .title(" Active ")
                            .border_style(Style::default().fg(Color::Yellow)),
                    );
                    let textarea_widget = textarea.widget();

                    let text = textarea.lines().join("\n");


                    let author_item: Vec<ListItem> = filter_authors(&author_list, text.clone());
                    let author_vec: Vec<&Author> = filter_author_vec(&author_list, text.clone());

                    let selected_author = author_vec
                        .get(
                            list_state
                                .selected()
                                .expect("there is always a selected author"),
                        )
                        .expect("exists")
                        .clone();
                    
                    let cell_vec:Vec<Row> = selected_author.books
                        .iter()
                        .map(|x| {
                            let title = x.title().unwrap().format_sentence();
                            let key = &x.key;
                            Row::new(vec![key.to_owned(), title])
                        })
                        .collect();

                    let book_details = Table::new(cell_vec)
                    .header(Row::new(vec![
                        Cell::from(Span::styled(
                            "Key",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Span::styled(
                            "Title",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                    ]))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::White))
                            .title("Detail")
                            .border_type(BorderType::Plain),
                    )
                    .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);

                    let authors_widget = List::new(author_item)
                        .block(Block::default().borders(Borders::ALL).title("Authors"))
                        .highlight_style(
                            Style::default()
                                .bg(Color::Yellow)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");


                    f.render_widget(textarea_widget, leftchunk[0]);
                    f.render_stateful_widget(authors_widget, leftchunk[1], &mut list_state);
                    f.render_widget(book_details, chunks[1]);
                }
                InputMode::Normal => {}
            }
        })?;
        match app.mode {
            InputMode::Normal => match crossterm::event::read()?.into() {
                Input {
                    key: Key::Char('i'),
                    ..
                } => app.mode = InputMode::Insert,
                Input {
                    key: Key::Char('q'),
                    ..
                } => break,
                _ => {}
            },
            InputMode::Insert => match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input { key: Key::Down, .. } => {
                    if list_state.selected().unwrap() < author_list.len()-1{
                        list_state.select(Some(list_state.selected().unwrap()+1));
                    }else {
                        list_state.select(Some(0))
                    }
                },
                Input { key: Key::Up, .. } => {
                    if list_state.selected().unwrap() != 0{
                        list_state.select(Some(list_state.selected().unwrap()-1));
                    }else {
                        list_state.select(Some(author_list.len()-1))
                    }
                },
                input => {
                    textarea.input(input);
                    list_state.select(Some(0))
                }
            },
        }
    }
    return Ok(());
}

fn filter_author_vec(authors: &Vec<Author>, regex:  String) -> Vec<& Author> {
    let re = Regex::new(&regex).unwrap();
    let items: Vec<_> = authors
        .iter()
        .filter(|x| re.is_match(x.get_name()))
        .collect();
    return items;
}

fn filter_authors(authors: &Vec<Author>, regex: String) -> Vec<ListItem> {
    let re = Regex::new(&regex).unwrap();
    let items: Vec<_> = authors
        .iter()
        .filter(|x| re.is_match(x.get_name()))
        .map(|x| ListItem::new(x.get_name().to_owned()))
        .collect();

    return items;
}

fn books_from_author(author: &String) -> Vec<String> {
    let bookitems: Vec<_> = bib::get_entries_by_author(
        "/home/caguiler/Phd/Database/Bib/karubib.bib".to_string(),
        author.to_string(),
    )
    .iter()
    .map(|x| " ".to_string() + &x.title().unwrap().format_sentence())
    .collect();
    return bookitems;
}
