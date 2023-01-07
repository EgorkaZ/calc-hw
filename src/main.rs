use std::io::{BufReader, BufRead};

use expr_parser::{parser, calculate, Printer};

fn main() {
    let reader = BufReader::new(std::io::stdin());

    reader.lines()
        .filter_map(|mb_line| match mb_line {
            Ok(line) => Some(line),
            Err(err) => {
                eprintln!("Couldn't read line: {err}");
                None
            },
        })
        .for_each(|line| match parser::parse(&line).collect::<Result<Vec<_>, _>>() {
            Ok(tokens) => {
                println!("{} = {}", Printer(&tokens), calculate(tokens.iter().copied()));
            },
            Err(err) => {
                eprintln!("Couldn't parse \"{line}\": {err}");
            }
        })
}
