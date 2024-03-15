// Copyright 2024 Felipe Torres González

use log::info;
use std::fs::read_to_string;
use std::{collections::HashMap, path::Path};

/// How many stock prices are included in a raw text file.
const N_STOCKS_IN_RAW_FILE: usize = 36;
/// How many lines has a common raw text data file.
const N_LINES_PER_RAW_FILE: usize = 51;
/// Separator for dates.
const DATE_SEPARATOR: char = '/';
/// Index of the date for the Ibex data entry in the raw text file.
const DATE_COLUMN_INDEX: usize = 5;
/// CSV separator
const CSV_SEPARATOR: char = ';';
/// Column index of the time stamp for a stock price entry.
const TS_COL_IDX: usize = 2;

/// A custom type that identifies an array of strings that will be used to filter results.
type StockFilter = Vec<String>;

/// A custom type that identifies an array that includes stock data.
type StockData = Vec<String>;

/// An object providing a parser for the Ibex index and its associated stocks.
///
/// # Description
///
/// An object that encapsulates the functionality to parse a raw text file and extract
/// a timestamped stock price data and accumulated volume.
///
/// The raw file must have a structure alike to the table found in [here][ibex35_data].
/// Just extract the entire HTML div with the `id=root` and save it as a text file.
/// An example of the data file that results of it is found inside the test folder.
///
/// If the raw text data file has slightly different information, this parser could still
/// be used. Consider using `with_custom_values` instead of `new` to specify some critical
/// information about the structure of the text file.
///
/// ## Example of use
///
/// The following example shows how to parse a text file and print the result to the
/// console:
///
/// ```rust
/// use ibex_parser::discover;
/// use ibex_parser::parser_ibex::IbexParser;
/// use std::path::Path;
///
/// fn main() {
///     let path = Path::new("./tests/data/data_ibex.csv");
///     let mut parser = IbexParser::new();
///     let mut data = parser.parse_file(path).unwrap();
///
///     for line in data {
///         println!("{:?}", line);
///     }
/// }
/// ```
/// [ibex35_data]: https://www.bolsasymercados.es/bme-exchange/es/Mercados-y-Cotizaciones/Acciones/Mercado-Continuo/Precios/ibex-35-ES0SI0000005
pub struct IbexParser {
    skip_n_lines_beg: usize,
    ibex_line: usize,
    skip_n_lines_end: usize,
    cols_to_keep_main: Vec<usize>,
    cols_to_keep_stock: Vec<usize>,
    target_date: Option<String>,
    col_date: usize,
    ts_table: HashMap<String, i32>,
}

impl IbexParser {
    /// Default constructor for IbexParser.
    ///
    /// # Description
    ///
    /// This constructor builds an instance of IbexParser following the format expected
    /// for a file alike to the one found under `tests/data/data_ibex.csv`.
    ///
    /// The first 11 lines should be skipped, but the 6th, which contains the information
    /// for the index itself.
    /// The last 5 lines contain no useful data either, and are skipped.
    ///
    /// Only some columns are parsed: "Nombre", "Fecha", "Hora", "Último", "Volumen" and
    /// "Efectivo (miles €)". For a different parsing schema, use `with_custom_values`.
    pub fn new() -> IbexParser {
        IbexParser {
            skip_n_lines_beg: 11,
            ibex_line: 6,
            skip_n_lines_end: 5,
            cols_to_keep_main: vec![0, 5, 6, 1],
            cols_to_keep_stock: vec![0, 7, 8, 1, 5, 6],
            target_date: None,
            col_date: DATE_COLUMN_INDEX,
            ts_table: HashMap::new(),
        }
    }

    /// Set a target date to filter out entries with a different date.
    ///
    /// # Description
    ///
    /// This method sets a target day that will be used to filter out parsed entries from
    /// different days. Filtering is only enabled for days, so if a month/year are included
    /// in the argument, those will be ignored.
    ///
    /// If `None` is passed as argument, the method will retrieve the current target day.
    ///
    /// ## Arguments
    ///
    /// - `date` specifies a target day. It can be passed as a single day, like `"21"` or
    ///   as part of a full date, like `"21/01/2023"`, in which the month and year values
    ///   will be just ignored.
    ///
    /// ## Returns
    ///
    /// The current set day as target. If no valid date was set before, `None` is returned.
    /// The format of the date is a bare integer number as string indicating the day of the
    /// month.
    pub fn target_date(&mut self, date: Option<&str>) -> Option<&str> {
        if let Some(d) = date {
            let mut parsed_date: Vec<&str> = Vec::new();

            if d.contains('/') {
                parsed_date = d.split(DATE_SEPARATOR).collect();
            } else {
                parsed_date.push(d);
            }
            info!("Target day set to {:?}", self.target_date);

            self.target_date = Some(String::from(parsed_date[0]))
        }
        info!("Target day set to {:?}", self.target_date);

        self.target_date.as_ref().map(|x| x.as_ref())
    }

    /// Internal method that builds a HashMap to keep track of the time stamps
    /// of the stock entries.
    fn init_ts_table(&mut self, filters: &StockFilter) {
        // Only build this table when there are filters and the table was empty.
        if filters.len() > 0 && self.ts_table.len() < 1 {
            for f in filters {
                self.ts_table.insert(f.clone(), -1);
            }
        }
    }

    /// Internal method that parses the content of a raw text file.
    ///
    /// # Description
    ///
    /// This method reads the input text file and parses stock price entries row by row.
    /// It supports using target dates to skip data that it's not interesting. There
    /// are no more filtering features applied by this method. More advanced filters can
    /// be applied by the following stages of the parsing chain.
    ///
    /// ## Arguments
    ///
    /// An instance of a `Path` struct that points to a file that contains a raw text
    /// file with the structure alike to one the found in [here][ibex35_data].
    ///
    /// ## Returns
    ///
    /// A `StockData` type that contains the parsed data. Each row is pushed to a single
    /// position of the `StockData` collection.
    fn load_data_file(&self, path: &Path) -> Option<StockData> {
        let raw_data = read_to_string(path).expect("Couldn't read lines from the file");
        // Line counter.
        let mut counter: usize = 0;
        // Vector that contains each line of the raw text file.
        let lines: Vec<&str> = raw_data.lines().collect();
        // The end of the zone that contains useful data in the raw text file.
        let end = lines.len() - self.skip_n_lines_end;
        // The stock data container, 35 companies + a value for the index.
        let mut data: Vec<String> = Vec::with_capacity(N_STOCKS_IN_RAW_FILE);
        // Pointer to the columns vector that defines the order in which the parsed columns should be saved.
        let mut ref_cols_to_keep = &self.cols_to_keep_main;
        // Flag that allows stopping the parser.
        let mut stop_parsing = false;
        // Flag that gets asserted once the date for the file is parsed.
        let mut file_date_parsed: bool = false;

        // Closure that extracts the day from a date. Useful to check if the date within the time stamp of the file
        // matches the target date (when given).
        let extract_day = |date: &str| {
            // A full date was given, extract the first component: the day.
            if date.contains('/') {
                String::from(date.split(DATE_SEPARATOR).collect::<Vec<&str>>()[0])
            // Only the day was given, nothing to do.
            } else {
                String::from(date)
            }
        };

        // Avoid a parsing attempt over junk files. This would only cause troubles, and we know a valid file
        // at a bare minimum need to contain info for 35 stocks + the index.
        if lines.len() < N_LINES_PER_RAW_FILE {
            info!("The current file contains no valid data to be parsed.");
            None
        } else {
            for line in lines {
                // This condition asserts when the input file includes data for a day different from the target day.
                if stop_parsing {
                    break;
                // Parsing the line with the data for the Ibex Index.
                } else if counter == self.ibex_line {
                    counter += 1;
                // Parsing garbage data between the beginning of the file and the actual data.
                } else if counter < self.skip_n_lines_beg {
                    counter += 1;
                    continue;
                // Parsing stock data.
                } else if counter < end {
                    counter += 1;
                    ref_cols_to_keep = &self.cols_to_keep_stock;
                // Reached the end of the stock data, whatever is found after that, is not useful.
                } else {
                    break;
                }

                // Vectorize the line to access easily to each column.
                let raw_row: Vec<&str> = line.split("\t").collect();

                // If a target date was given, we check that the input file matches the same date, otherwise skip
                // parsing this file.
                if self.target_date.is_some() && !file_date_parsed {
                    if extract_day(raw_row[self.col_date]) != self.target_date.as_deref().unwrap() {
                        stop_parsing = true;
                        continue;
                    } else {
                        file_date_parsed = true;
                    }
                }

                let mut row: String = String::from("");
                // Build the new row with the parsed information that is useful.
                for col in ref_cols_to_keep.iter() {
                    row.push_str(raw_row[*col]);
                    row.push(CSV_SEPARATOR);
                }

                // Remove the last empty space.
                row.pop();
                data.push(row);
            }

            Some(data)
        }
    }

    /// Internal method that analyses a `StockData` collection to extract stock names.
    ///
    /// # Description
    ///
    /// This method is used to build a map that keeps track of the time stamp entries
    /// for each stock for the input data files.
    /// Rather than writing by hand the names of the 35 stocks + the Index name, this
    /// method extracts those from the input data. This way, the code will be valid
    /// when any stock leaves the index and another one is included in it.
    ///
    /// ## Arguments
    ///
    /// A `StockData` collection as reference that results after parsing a input
    /// data file.
    ///
    /// ## Preconditions
    ///
    /// - The input argument `stock_vec` must contain stock values, i.e. `stock_vec.len() > 0`.
    /// - The stocks included in the input data files must be always present for all the input
    ///   files that are fed to the parser at once.
    fn extract_stocks(&self, stock_vec: &StockData) -> StockData {
        let mut filters = Vec::new();

        for item in stock_vec {
            let values: StockData = item.split(";").map(|e| e.to_string()).collect();
            filters.push(values[0].clone());
        }

        filters
    }

    /// Parse a text file that contains stock prices.
    ///
    /// # Description
    ///
    /// This method reads a text file by lines and parses the information to extract
    /// stock prices and other information. The structure of the text file is alike
    /// to the table found [here][ibex35_data].
    ///
    /// Briefly, there is a line at line 7 that contains the information for the index.
    /// Then, at line 11, there are 35 lines in which each line includes the information
    /// for a stock of the index.
    ///
    /// Some values are discarded as I find them of little relevance. The following
    /// values are parsed:
    /// - Stock name at column 0.
    /// - Timestamp of the values at columns 7 (date) and 8 (time).
    /// - Last negotiated price at column 1.
    /// - Accumulated volume at column 5.
    /// - Accumulated volume in thousands of Euro at column 6.
    ///
    /// **The values are returned in that order** for each stock entry.
    ///
    /// The parser analyses the time stamp for each stock price entry. If an entry contains
    /// a time stamp older in time than the previous parsed, the entry is skipped. Also, when
    /// "Cierre" is parsed from the time stamp column, the parsing is stopped because it that
    /// value means no more stock price entries with valid data will be read until a new
    /// trading day.
    ///
    /// ## Arguments
    ///
    /// An instance of a `Path` struct that points to a file that contains a raw text
    /// file with the structure alike to one the found in [here][ibex35_data].
    ///
    /// ### Preconditions
    ///
    /// The file pointed by `path` must exist and the owner of the process running this
    /// code must have permissions to read such file.
    ///
    /// ## Returns
    ///
    /// A wrapped vector in which each position contains a `String` with the values for a
    /// stock. An example of one entry:
    /// ```text
    /// "B.SANTANDER 06/02/2024 15:19:51 3,7420 12.825.738 47.876,71"
    /// ```
    ///
    /// If valid data could not be parsed, `None` is returned.
    ///
    /// That line could be modified using `with_custom_values`, see its documentation to
    /// get more details.
    ///
    /// [ibex35_data]: https://www.bolsasymercados.es/bme-exchange/es/Mercados-y-Cotizaciones/Acciones/Mercado-Continuo/Precios/ibex-35-ES0SI0000005
    pub fn parse_file(&mut self, path: &Path) -> Option<StockData> {
        // If None is received, return it to the caller.
        let raw_data = self.load_data_file(path)?;
        let mut data = Vec::new();
        // This method receives no filters, thus the time stamp map shall contain all the
        // stocks as filters.
        let filter = self.extract_stocks(&raw_data);
        // Now, build the time stamp map for all the stocks.
        self.init_ts_table(&filter);

        for item in raw_data.iter() {
            let col_split = item.split(";").collect::<Vec<&str>>();
            // Extract the time of the entry.
            let mut str_ts = col_split[TS_COL_IDX].to_string();
            let stock = col_split[0];

            // When "Cierre" is read, there won't be any more new entries until the
            // next trading day for this stock, using an impossible time stamp to
            // signal such scenario.
            if str_ts == "Cierre" {
                self.ts_table.insert(stock.to_string(), <i32>::default());
                continue;
            }

            // And make it planar so we can do simple maths with it.
            str_ts.retain(|c| c != ':');

            // Parse the time stamp for the current entry price, if it is more recent than the
            // previous one, store it. Omit it otherwise. This way, if data files after the closing of
            // the session are present, the values will be safely omitted.
            let current_ts = str_ts.parse::<i32>().unwrap_or_default();

            if *self.ts_table.get(stock).unwrap() != current_ts {
                data.push(item.clone());
                self.ts_table.insert(stock.to_string(), current_ts);
                continue;
            }
        }

        Some(data)
    }

    /// Parse and filter a text file that contains stock prices.
    ///
    /// # Description
    ///
    /// This method performs a parsing of a text data file in the same way as `parse_file`
    /// does, but it also filters out stock entries that are not included in the argument
    /// `filter`. When using an empty filter, calling this method yields the same result
    /// as calling `parse_file`.
    ///
    /// ## Arguments
    ///
    /// - An instance of a `Path` struct that points to a file that contains a raw text
    ///   file with the structure alike to one the found in [here][ibex35_data].
    /// - A `StockFilter` instance that includes none or some strings that will be used
    ///   to filter what stock's data is the user interested about.
    ///
    /// ### Preconditions
    ///
    /// The file pointed by `path` must exist and the owner of the process running this
    /// code must have permissions to read such file.
    ///
    /// ## Returns
    ///
    /// A wrapped vector in which each position contains a `String` with the values for a
    /// stock. An example of one entry:
    /// ```text
    /// "B.SANTANDER 06/02/2024 15:19:51 3,7420 12.825.738 47.876,71"
    /// ```
    ///
    /// If valid data could not be parsed, `None` is returned.
    ///
    /// That line could be modified using `with_custom_values`, see its documentation to
    /// get more details.
    ///
    /// [ibex35_data]: https://www.bolsasymercados.es/bme-exchange/es/Mercados-y-Cotizaciones/Acciones/Mercado-Continuo/Precios/ibex-35-ES0SI0000005
    pub fn filter_file(&mut self, path: &Path, filter: &StockFilter) -> Option<StockData> {
        let raw_data = self.parse_file(path);

        if raw_data == None {
            return None;
        }

        // Allow using this method as a regular `parse_file` when no filters are given.
        if filter.len() == 0 {
            return raw_data;
        }

        let mut data: StockData = Vec::new();

        for item in raw_data.unwrap().iter() {
            // Push the data only if it belongs to the filters (if any was given).
            for f in filter {
                if item.contains(f) {
                    data.push(item.clone());
                    // Update the time stamp of the last pushed value.
                    break;
                }
            }
        }

        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use std::path::Path;

    #[fixture]
    fn valid_data_cierre() -> Box<&'static Path> {
        Box::new(Path::new("./tests/data/data_ibex.csv"))
    }

    #[fixture]
    fn valid_data() -> Box<&'static Path> {
        Box::new(Path::new("./tests/data/data_ibex(1).csv"))
    }

    #[fixture]
    fn non_existing_data() -> Box<&'static Path> {
        Box::new(Path::new("./tests/data/dato_ibex.csv"))
    }

    #[fixture]
    fn wrong_data() -> Box<&'static Path> {
        Box::new(Path::new("./tests/data/wdata_ibex.csv"))
    }

    // Check that we can parse a file with data.
    #[rstest]
    fn test_ibexparser_parse_file(valid_data: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *valid_data;

        let parsed_data = parser.parse_file(path).unwrap();
        assert_eq!(parsed_data.len(), N_STOCKS_IN_RAW_FILE);
        let mut first_parsed: bool = false;
        for item in parsed_data.iter() {
            let entry: Vec<&str> = item.split(";").collect();
            if !first_parsed {
                // 4 columns selected for the Ibex's entry (see new).
                assert_eq!(entry.len(), 4);
                first_parsed = !first_parsed;
            } else {
                // 6 columns selected for the stocks's entry (see new).
                assert_eq!(entry.len(), 6);
            }
        }
    }

    // Check that the parser fails to parse a non existing file.
    #[rstest]
    #[should_panic]
    fn test_ibexparser_parse_nofile(non_existing_data: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *non_existing_data;

        let _parsed_data = parser.parse_file(path).unwrap();
    }

    #[rstest]
    fn test_ibexparser_parse_wrongfile(wrong_data: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *wrong_data;

        let parsed_data = parser.parse_file(path);
        assert_eq!(parsed_data, None);
    }

    #[rstest]
    fn test_ibexparser_filter_file_cierre(valid_data_cierre: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *valid_data_cierre;
        let mut filter: StockData = vec!["AENA".to_string()];

        let mut parsed_data = parser.filter_file(path, &filter);
        // The entry for AENA has a "Cierre" timestamp.
        assert_eq!(parsed_data.unwrap().len(), 0);

        filter.push(String::from("ACS"));
        parsed_data = parser.filter_file(path, &filter);
        // The entry for ACS has a "Cierre" timestamp.
        assert_eq!(parsed_data.unwrap().len(), 0);

        // Drop the previous filter and use and empty filter to check that calling
        // `filter_file` with an empty filter yields the same result as `parse_file`.
        filter = Vec::new();
        parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data.unwrap().len(), 0);
    }

    #[rstest]
    fn test_ibexparser_filter_file(valid_data: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *valid_data;
        let filter: StockData = vec!["AENA".to_string(), "ACS".to_string()];

        let parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data.unwrap().len(), filter.len());
    }

    #[rstest]
    fn test_ibexparser_filter_wrongfile(wrong_data: Box<&'static Path>) {
        let mut parser = IbexParser::new();
        let path = *wrong_data;
        let filter: StockData = vec!["AENA".to_string()];

        let parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data, None);
    }

    #[rstest]
    fn test_ibexparser_target_date() {
        let mut parser = IbexParser::new();
        let set_date = "23";
        let set_full_date = "23/01/2023";

        assert_eq!(parser.target_date(None), None);
        assert_eq!(parser.target_date(Some(set_date)).unwrap(), set_date);
        assert_eq!(parser.target_date(Some(set_full_date)).unwrap(), set_date);
    }

    #[rstest]
    fn test1_ibexparser_extract_stocks(valid_data: Box<&'static Path>) {
        let parser = IbexParser::new();
        let path = *valid_data;
        let raw_data = parser.load_data_file(path).unwrap();

        let parsed_data = parser.extract_stocks(&raw_data);
        assert_eq!(parsed_data.len(), N_STOCKS_IN_RAW_FILE);
    }
}
