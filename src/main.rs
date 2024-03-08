// Copyright 2024 Felipe Torres González

use clap::Parser;
use ibex_parser::discover;
use ibex_parser::parser_ibex::IbexParser;
use std::path::Path;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

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
    filter: Option<Vec<String>>,
    /// Name of the data files.
    #[arg(long)]
    file_stem: Option<String>,
    /// Extension of the data files.
    #[arg(long)]
    file_ext: Option<String>,
    /// Target day for parsing data.
    #[arg(long)]
    target_date: Option<String>,
}

fn main() {
    pretty_env_logger::init();
    let args = Args::parse();
    // Check whether the stock list should be filtered to show only one stock entry.
    let mut filter: Vec<String> = Vec::new();

    if let Some(x) = args.filter.as_deref() {
        filter.extend_from_slice(x);
    }

    let path = Path::new(&args.path);
    // Call discover to build a list of data files that can be parsed later.
    let files = discover(path, args.file_stem.as_deref(), args.file_ext.as_deref());
    debug!("List of files to be parsed:");
    debug!("{:?}", files);

    // Instance the parser and attempt to parse all the discovered files.
    let mut parser = IbexParser::new();
    // Pass the wrapped target date.
    parser.target_date(args.target_date.as_deref());
    if let Some(x) = parser.target_date(None) {
        info!("Files that contain data for day {x} will be parsed.");
    }

    for file in files {
        let file_string = format!("{}/{}", &args.path, file.as_str());
        let path = Path::new(&file_string);

        // Avoid passing empty files to the parser.
        if path.metadata().unwrap().len() < MIN_BYTES_X_FILE {
            continue;
        }
        debug!("Parsing {file_string} using filters: {:?}", filter);
        let data = parser.filter_file(path, &filter);

        match data {
            Some(x) => {
                for line in x {
                    println!("{}", line);
                }
            }
            None => warn!("File {file} doesn't contain valid data."),
        }
    }
}
