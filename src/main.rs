mod bib;
mod utils;

use itertools::Itertools;
use bib::Author;
use biblatex::{ChunksExt, Entry};
use regex::Regex;
use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant}, path::{Path, PathBuf},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap, Clear},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event, read, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use tui_textarea::{Input, Key, TextArea};


enum InputMode {
    Author,
    Title,
    Popup,
}

struct App {
    path: String,
    mode: InputMode,
    bib_file: String,
}

impl Default for App {
    fn default() -> App {
        let binding = dirs::home_dir().unwrap();
        let home = binding.as_path().to_str().unwrap();
        let path = home.to_string()+"/Phd/Database";
        let bib_file = home.to_string()+"/Phd/Database/Bib/karubib.bib";
            App {
                path,
                mode: InputMode::Author,
                bib_file,
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

fn render_books(selected_author:Option<&Author>) -> Table{
    let mut cell_vec:Vec<Row> = Vec::new();
    match selected_author {
        Some(y) => {
            cell_vec = y.books
                .iter()
                .map(|x| {
                    let mut title = String::new();
                    match x.file() {
                        Ok(y)  => title = " ".to_string() + &x.title().unwrap().format_sentence(),
                        _ => title = x.title().unwrap().format_sentence()
                    }
                    // let key = " ".to_string()+&x.key;
                    let authors:Vec<String> = x.author().unwrap()
                        .iter()
                        .map(|x|  x.clone().name )
                        .collect();
                    Row::new(vec![title, authors.join(" - ")])
                })
            .collect();
        },
        None => {}
    }

    let book_details = Table::new(cell_vec)
        .header(Row::new(vec![
                         // Cell::from(Span::styled(
                         //         "Key",
                         //         Style::default().add_modifier(Modifier::BOLD),
                         //         )),
                                 Cell::from(Span::styled(
                                         "Title",
                                         Style::default().add_modifier(Modifier::BOLD),
                                         )),
                                 Cell::from(Span::styled(
                                         "Authors",
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
        .widths(&[ Constraint::Percentage(70), Constraint::Percentage(30)]);

    return book_details;
}

fn render_list(author_list: &Vec<Author>, text:String, list_state:ListState) -> (List, Option<&Author>) {
    let author_item: Vec<ListItem> = filter_authors(&author_list, text.clone());
    let author_vec: Vec<&Author> = filter_author_vec(&author_list, text.clone());
    let mut selected_author = None;

    match author_vec.len() {
        0 => {},
        _ => {
            selected_author = Some(author_vec
            .get(
                list_state
                .selected()
                .expect("there is always a selected author"),
                )
            .expect("exists")
            .clone());
        }
    }

    let authors_widget = List::new(author_item)
        .block(Block::default().borders(Borders::ALL).title("Authors"))
        .highlight_style(
            Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
            )
        .highlight_symbol(">> ");

    return (authors_widget, selected_author);
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut textarea: TextArea,
) -> io::Result<()> {
    let author_list = bib::get_authors(app.bib_file);
    let mut list_state = ListState::default();
    let mut table_state = TableState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let leftchunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(80)].as_ref())
                .split(chunks[0]);

            textarea
                .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
            textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            let b = textarea
                .block()
                .cloned()
                .unwrap_or_else(|| Block::default().borders(Borders::ALL));
            textarea.set_block(
                b.style(Style::default())
                .title(" Search by Author ")
                .border_style(Style::default().fg(Color::Yellow)),
                );

            let textarea_widget = textarea.widget();
            let text = textarea.lines().join("\n");

            let (list_widget, selected_author) = render_list(&author_list, text, list_state.clone());
            let book_widget = render_books(selected_author);

            let style = Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);


            match app.mode {
                InputMode::Author => {
                    f.render_widget(textarea_widget, leftchunk[0]);
                    f.render_stateful_widget(list_widget.highlight_style(style), leftchunk[1], &mut list_state);
                    f.render_stateful_widget(book_widget, chunks[1], &mut table_state);
                }
                InputMode::Title => {
                    f.render_widget(textarea_widget, leftchunk[0]);
                    f.render_stateful_widget(list_widget, leftchunk[1], &mut list_state);
                    f.render_stateful_widget(book_widget.highlight_style(style), chunks[1], &mut table_state);

                },
                InputMode::Popup => {
                    utils::create_popup(f, 50, 20, "Error".to_string(), "Popup".to_string());
                    app.mode = InputMode::Title;
                }
            }

        })?;

        match app.mode {
            InputMode::Popup => match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                _ => {}
            },
            InputMode::Title => match crossterm::event::read()?.into() {                
                Event::Key(KeyEvent {code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, ..}) => {
                    terminal.clear()?;
                    break;
                }
                Event::Key(KeyEvent {code: KeyCode::Enter, modifiers: KeyModifiers::NONE, ..}) => {
                    let text = textarea.lines().join("\n");
                    let (_, selected_author) = render_list(&author_list, text, list_state.clone());
                    let entry = selected_author.unwrap().books.get(table_state.selected().unwrap()).unwrap();
                    match entry.file() {
                        Ok(y)  => {
                            let complete_path = y.split(":").nth(1).unwrap();
                            let file = Path::new(complete_path).file_name().unwrap().to_str().unwrap();
                            let path_file = app.path.clone()+"/Papers/"+file;
                            opener::open(path_file);
                        }
                        _ => app.mode=InputMode::Popup
                    }
                },
                Event::Key(KeyEvent {code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, ..}) | Event::Key(KeyEvent {code: KeyCode::Backspace, modifiers: KeyModifiers::NONE, ..})  => {
                    app.mode = InputMode::Author;
                },
                 Event::Key(KeyEvent {code: KeyCode::Down, modifiers: KeyModifiers::NONE, ..}) | Event::Key(KeyEvent {code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, ..}) | Event::Key(KeyEvent {code: KeyCode::Tab, modifiers: KeyModifiers::NONE, ..})  => {
                    let text = textarea.lines().join("\n");
                    let (_, selected_author) = render_list(&author_list, text, list_state.clone());
                    match table_state.selected() {
                        Some(x) => {
                            if x < selected_author.unwrap().books.len()-1 {
                                table_state.select(Some(x+1));
                            }else {
                                table_state.select(Some(0));
                            }
                        },
                        None => table_state.select(Some(0)),
                    }
                },
                Event::Key(KeyEvent {code: KeyCode::Up, modifiers: KeyModifiers::NONE, ..}) | Event::Key(KeyEvent {code: KeyCode::Char('k'), modifiers: KeyModifiers::NONE, ..}) |Event::Key(KeyEvent {code: KeyCode::BackTab, modifiers: KeyModifiers::SHIFT, ..})  => {
                    let text = textarea.lines().join("\n");
                    let (_, selected_author) = render_list(&author_list, text, list_state.clone());
                    match table_state.selected() {
                        Some(x) => {
                            if x > 0 {
                                table_state.select(Some(x-1));
                            }else {
                                table_state.select(Some(selected_author.unwrap().books.len()-1));
                            }
                        },
                        None => table_state.select(Some(0)),
                    }
                },
                _ => {}
            },
            InputMode::Author => match crossterm::event::read()?.into() {
                Event::Key(KeyEvent {code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, ..}) => break,
                Event::Key(KeyEvent {code: KeyCode::Enter, modifiers: KeyModifiers::NONE, ..}) => {
                    app.mode = InputMode::Title;
                    table_state.select(Some(0));
                },
                Event::Key(KeyEvent {code: KeyCode::Down, modifiers: KeyModifiers::NONE, ..}) => {
                    if list_state.selected().unwrap() < author_list.len()-1{
                        list_state.select(Some(list_state.selected().unwrap()+1));
                    }else {
                        list_state.select(Some(0))
                    }
                },
                Event::Key(KeyEvent {code: KeyCode::Up, modifiers: KeyModifiers::NONE, ..}) => {
                    if list_state.selected().unwrap() != 0{
                        list_state.select(Some(list_state.selected().unwrap()-1));
                    }else {
                        list_state.select(Some(author_list.len()-1))
                    }
                },
                Event::Key(KeyEvent {code: KeyCode::Tab, modifiers: KeyModifiers::NONE, ..}) => {
                    app.mode = InputMode::Title;
                    table_state.select(Some(0));
                },
                input => {
                    textarea.input(input);
                    table_state.select(None);
                    list_state.select(Some(0));
                }
            },
        }
    }
    return Ok(());
}

fn filter_author_vec(authors: &Vec<Author>, regex:  String) -> Vec<& Author> {
    let re = Regex::new(&regex.to_lowercase()).unwrap();
    let items: Vec<_> = authors
        .iter()
        .filter(|x| re.is_match(&x.get_name().to_lowercase()))
        .collect();
    return items;
}

fn filter_authors(authors: &Vec<Author>, regex: String) -> Vec<ListItem> {
    let re = Regex::new(&regex.to_lowercase()).unwrap();
    let items: Vec<_> = authors
        .iter()
        .filter(|x| re.is_match(&x.get_name().to_lowercase()))
        .map(|x| ListItem::new(x.get_name().to_owned()))
        .collect();
    return items;
}


