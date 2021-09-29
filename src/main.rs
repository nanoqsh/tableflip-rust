mod lexeme;
mod parser;
mod rows;
mod table;

pub use lexeme::Lexeme;
use parser::Parser;
use rows::Rows;
use std::{
    io::{self, Read},
    process::exit,
};
use table::Table;

fn parse_error(at: usize) -> ! {
    eprintln!("parse error at {}", at);
    exit(1);
}

fn main() {
    // Read all input to string
    // since we still need to calculate
    // the table column width
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Invalid input");

    let parser = Parser::new(&input).map(|res| match res {
        Ok(lex) => lex,
        Err(at) => parse_error(at),
    });

    let (head, mut tail) = Rows::new(parser).split();
    let mut table = Table::new().head(head);

    while let Some(row) = tail.row() {
        table = table.tail(row);
    }

    print!("{}", table);
}
