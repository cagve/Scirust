use std::fs;
use biblatex::{Bibliography, ChunksExt, Entry};


pub struct Author {
    pub name: String,
    pub books: Vec<Entry>,
}

impl Author {
    pub fn new(name: String, books: Vec<Entry>) -> Author {
        return Author { name, books };
    }
    
    pub fn get_name(&self) -> &String{
        return &self.name;
    }

    pub fn books_to_string(&self) -> String{
        let book_string:Vec<String> = self.books
            .iter()
            .map(|x| x.title().unwrap().to_biblatex_string(true))
            .collect();

        let str = book_string.join("-");
        return str;
    }
}

pub fn get_authors(bibfile: String) -> Vec<Author> {
    let file = bibfile.clone();
    let bib = get_bibliography(file);
    let mut authors: Vec<Author> = Vec::new();
    bib.iter().for_each(|x| {
        x.author()
            .unwrap()
            .iter()
            .for_each(|y| {
                let entries = get_entries_by_author(bibfile.clone(), y.to_string());
                let author = Author::new(y.to_string(), entries);
                authors.push(author);
            })
    });
    return authors;
}

pub fn get_entries_by_author(bibfile: String, author: String) -> Vec<Entry> {
    // FIX THIS
    let bib = get_bibliography(bibfile);
    let filtered = bib
        .iter()
        .filter(|x| {
            return x
                .author()
                .unwrap()
                .iter()
                .filter(|f| {
                    return author.contains(&f.name);
                })
                .collect::<Vec<_>>()
                .len()
                > 0;
        })
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    // let filtered = bib.iter().map(|x| x.to_owned()).collect::<Vec<_>>();
    return filtered;
}

pub fn get_bibliography(bibfile: String) -> Vec<biblatex::Entry> {
    let bib = fs::read_to_string(bibfile).unwrap(); // >  Read file
                                                    // let src = bib.split_once("@comment END").unwrap().0;        // > Delete comments about groups
    let src = bib.split_once("@Comment").unwrap().0; // > Delete comments about groups
    let bibliography = Bibliography::parse(src).unwrap();
    return bibliography.into_vec();
}
