mod bib;

use skim::prelude::*;
use std::io::Cursor;

use std::{io, thread, time::{Duration, Instant}, sync::mpsc};
use bib::get_authors;
use std::process::Command;
use biblatex::ChunksExt;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table},
    Terminal,
};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

 
enum Event<I> {
    Input(I),
    Tick,
}

fn render_list<'a>(list_state: &ListState, category:String) -> (List<'a>, Table<'a>) {
    let list = bib::get_bibliography("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string());
    let items: Vec<_> = list
        .iter()
        .map(|x| ListItem::new(x.key.to_string()))
        .collect();

    let selected_entry = list
        .get(
            list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("Exists");

    let max = selected_entry.author().unwrap().len();
    let table = Table::new(vec![Row::new(vec![
        Cell::from(selected_entry.author().unwrap().iter().map(|x| {" ".to_string()+&x.to_string()}).collect::<Vec<String>>().join("\n")),
        Cell::from(Span::raw(selected_entry.title().unwrap().format_sentence())),
        Cell::from(Span::raw(selected_entry.key.to_string())),
    ]).height(max as u16)])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Author",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            selected_entry.entry_type.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Key",
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
    .widths(&[
        Constraint::Percentage(30),
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]);

    let renderlist = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    return (renderlist, table);
}

fn render_author<'a>(list_state: &ListState) -> (List<'a>,List<'a>) {
    let list = bib::get_bibliography("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string());
    let authors = bib::get_authors("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string());
    let items: Vec<_> = authors
        .iter()
        .map(|x| ListItem::new(x.to_string()))
        .collect();

    let selected_entry = authors
        .get(
            list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("Exists");

    let bookitems: Vec<_> = bib::get_entries_by_author("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string(), selected_entry.to_string())
        .iter()
        .map(|x| ListItem::new(" ".to_string()+&x.title().unwrap().format_sentence()))
        .collect();

    let renderlist = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Author"))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let title  = "List of books of ".to_string()+&selected_entry.to_string();

    let booklist = List::new(bookitems)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    return (renderlist, booklist); 
}




fn fzf_menu(input:Vec<String>) -> String {
    // let options = SkimOptionsBuilder::default()
    //     .height(Some("50%"))
    //     .multi(true)
    //     .build()
    //     .unwrap();
    //
    // let string = input.join("\n");
    //
    // let item_reader = SkimItemReader::default();
    // let items = item_reader.of_bufread(Cursor::new(string));
    //
    // let selected_items = Skim::run_with(&options, Some(items))
    //     .map(|out| out.selected_items)
    //     .unwrap_or_else(|| Vec::new());
    //
    // let result = selected_items
    //     .into_iter()
    //     .next()
    //     .unwrap()
    //     .text()
    //     .to_string();
    //

    
    let result = Command::new("ls")
        .output()
        .expect("ls command failed to start");

    return result.status.to_string();
}

// fn main(){
//     let authors = bib::get_authors("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string());
//     let selected = fzf_menu(authors);
//     print!("{}", selected);
// }

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut list_state = ListState::default();
    list_state.select(Some(4));

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

            let prueba =   Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Detail")
                .border_type(BorderType::Plain);

            let (authors, book) = render_author(&list_state);
            f.render_widget(prueba, leftchunk[0]);
            f.render_stateful_widget(authors, leftchunk[1], &mut list_state);
            f.render_widget(book, chunks[1]);

        })?;
        match rx.recv().expect("Is working") {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('j') =>{
                    let len = bib::get_bibliography("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string()).len(); //NOT NECESARY ARG
                    if list_state.selected().unwrap() >= len-1 {
                        list_state.select(Some(0));
                    }else{
                        list_state.select(Some(list_state.selected().unwrap() + 1));
                    }
                } 
                KeyCode::Char('k') =>{
                    let len = bib::get_bibliography("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string()).len(); //NOT NECESARY ARG
                    if list_state.selected().unwrap() == 0{
                        list_state.select(Some(len-1));
                    }else{
                        list_state.select(Some(list_state.selected().unwrap() - 1));
                    }
                } 
                KeyCode::Char('1') =>{
                    disable_raw_mode()?;
                    let selected = fzf_menu(bib::get_authors("/home/caguiler/Phd/Database/Bib/karubib.bib".to_string()));
                    print!("{}", selected);
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }; // end of renKer loop
    Ok(())
}
