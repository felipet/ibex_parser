// Copyright 2024 Felipe Torres González

use ibex_parser::discover;
use ibex_parser::parser_ibex::IbexParser;
use std::path::Path;

fn main() {
    let dir = "./tests/data";
    let path = Path::new(dir);
    let files = discover(path, None, None);
    let mut parser = IbexParser::new();
    let mut data: Vec<String>;

    for file in files {
        println!("Parsing file: {:?}", file);
        let filepath = format!("{}/{}",dir,file.as_str());
        data = parser.parse_file(Path::new(&filepath));

        for line in data {
            println!("{:?}", line);
        }
    }

}
