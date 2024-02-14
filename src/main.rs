// Copyright 2024 Felipe Torres González

use clap::Parser;
use ibex_parser::discover;
use ibex_parser::parser_ibex::IbexParser;
use std::path::Path;

// The minium size of a text file that might contain stock data. Files with less than this size are omitted.
const MIN_BYTES_X_FILE: u64 = 560;

#[derive(Parser, Debug)]
#[command(name = "IbexParser")]
#[command(version = "0.1.0")]
#[command(
    about = "Parser for Ibex35 stock data",
    long_about = r#"
Ibex35 Data Parsing Tool: This tool parses stock prices and other data as is offered by BME's web.
Data is parsed from raw text files and output in CSV format for a later import into some analysis tool
or graph tool.

Raw text files shall keep the same data organization as BME's web does. For example, select all the content
of the page and paste it into a text file. That file is ready to be used by this parser.
"#
)]
struct Args {
    /// Directory to search for text data files.
    path: String,
    /// Company to filter the results.
    filter: Option<String>,
}

fn main() {
    let args = Args::parse();
    let filter: Vec<String> = if let Some(filter) = args.filter.as_deref() {
        vec![String::from(filter)]
    } else {
        Vec::new()
    };

    let path = Path::new(&args.path);
    let files = discover(path, None, None);
    let parser = IbexParser::new();

    for file in files {
        let file_string = format!("{}/{}", &args.path, file.as_str());
        let path = Path::new(&file_string);

        // Avoid passing empty files to the parser.
        if path.metadata().unwrap().len() < MIN_BYTES_X_FILE {
            continue;
        }
        let data = parser.filter_file(path, &filter);

        match data {
            Some(x) => {
                for line in x {
                    println!("{}", line);
                }
            }
            None => println!("File {file} doesn't contain valid data."),
        }
    }
}
