// Copyright 2024 Felipe Torres González

use std::path::Path;
use std::fs::read_to_string;

/// How many stock prices are included in a raw text file.
const N_STOCKS_IN_RAW_FILE: usize = 36;
/// How many lines has a common raw text data file.
const N_LINES_PER_RAW_FILE: usize = 51;

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
            cols_to_keep_main: vec![0,5,6,1],
            cols_to_keep_stock: vec![0,7,8,1,5,6],
        }
    }

    /// Parameterized constructor for IbexParser.
    ///
    /// # Description
    ///
    /// This constructor allows parsing a slightly different text file. In brief, the key
    /// important rows are the ones containing stock prices. The rest are simply ignored.
    /// There's one exception, the 6th line, which contains data for the index. If the
    /// raw text data is collected in a different way, just modify how many lines should
    /// be ignored at the file's header and bottom. If there's no special line for the
    /// index, just assign it to the first line containing useful data.
    ///
    /// It doesn't matter whether there 35 stock rows, or 2, if the bounds of the file are
    /// properly set, and the information is structured in columns split by the character `\t`.
    ///
    /// ## Arguments
    ///
    /// - `inil` indicates the number of header lines that shall be ignored by the parser.
    /// - `idxl` indicates the line index in which the information for the index is found. Usually
    ///   this line is found inside the initial header, so the parser will ignore `inil` lines but
    ///   the one pointed by this argument.
    /// - `endl` indicates the number of bottom lines that shall be ignored by the parser.
    /// - `colsidx` shall include the column indexes that shall be parsed for the special line.
    ///   See the [examples][#Examples] of use to get more details.
    /// - `colsstock` shall include the column indexes that shall be parsed for the regular stocks.
    ///   See the [examples][#Examples] of use to get more details.
    ///
    /// # Examples of use
    ///
    /// For example if we need only the stock price and its last price, we can skip the rest of
    /// columns from the parsing this way:
    ///
    /// ```rust,ignore
    /// let parser = IbexParser::with_custom_values(11, 6, 5, vec![0,1], vec![0,1]);
    /// ```
    pub fn with_custom_values(
        inil: usize,
        idxl: usize,
        endl: usize,
        colsidx: Vec<usize>,
        colsstock: Vec<usize>
    ) -> IbexParser {
        IbexParser {
            skip_n_lines_beg: inil,
            ibex_line: idxl,
            skip_n_lines_end: endl,
            cols_to_keep_main: colsidx,
            cols_to_keep_stock: colsstock,
        }
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
    pub fn parse_file(&self, path: &Path) -> Option<StockData> {
        let raw_data = read_to_string(path).expect("Couldn't read lines from the file");
        let mut counter: usize = 0;
        let lines: Vec<&str> = raw_data.lines().collect();
        let end = lines.len() - self.skip_n_lines_end;
        let mut data: Vec<String> = Vec::with_capacity(N_STOCKS_IN_RAW_FILE);
        let mut ref_cols_to_keep = &self.cols_to_keep_main;

        if lines.len() < N_LINES_PER_RAW_FILE {
            None
        } else {

            for line in lines {
                if counter == self.ibex_line {
                    counter += 1;
                } else if counter < self.skip_n_lines_beg {
                    counter += 1;
                    continue;
                } else if counter < end {
                    counter += 1;
                    ref_cols_to_keep = &self.cols_to_keep_stock;
                } else {
                    break;
                }

                let raw_row: Vec<&str> = line.split("\t").collect();
                let mut row: String = String::from("");

                for col in ref_cols_to_keep.iter() {
                    row.push_str(raw_row[*col]);
                    row.push(';');
                }

                // Remove the last empty space.
                row.pop();
                data.push(row);
            }

            Some(data)
        }
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
    pub fn filter_file(&self, path: &Path, filter: &StockFilter) -> Option<StockData> {
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
            for f in filter {
                if item.contains(f) {
                    data.push(item.clone());
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
    use rstest::*;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[fixture]
    fn valid_data() -> Box<&'static Path> {
        Box::new(Path::new("./tests/data/data_ibex.csv"))
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
        let parser = IbexParser::new();
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
        let parser = IbexParser::new();
        let path = *non_existing_data;

        let _parsed_data = parser.parse_file(path).unwrap();
    }

    #[rstest]
    fn test_ibexparser_parse_wrongfile(wrong_data: Box<&'static Path>) {
        let parser = IbexParser::new();
        let path = *wrong_data;

        let parsed_data = parser.parse_file(path);
        assert_eq!(parsed_data, None);
    }

    #[rstest]
    fn test_ibexparser_parse_customfile(valid_data: Box<&'static Path>) {
        let parser = IbexParser::with_custom_values(
            11, 6, 5,
            vec![0,1], vec![0,1]
        );
        let path = *valid_data;

        let parsed_data = parser.parse_file(path).unwrap();
        assert_eq!(parsed_data.len(), N_STOCKS_IN_RAW_FILE);
        for item in parsed_data.iter() {
            let entry: Vec<&str> = item.split(";").collect();
            // Only 2 columns where selected at instantiation.
            assert_eq!(entry.len(), 2);
        }
    }

    #[rstest]
    fn test_ibexparser_filter_file(valid_data: Box<&'static Path>) {
        let parser = IbexParser::new();
        let path = *valid_data;
        let mut filter: StockData = vec!["AENA".to_string()];

        let mut parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data.unwrap().len(), filter.len());

        filter.push(String::from("ACS"));
        parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data.unwrap().len(), filter.len());

        // Drop the previous filter and use and empty filter to check that calling
        // `filter_file` with an empty filter yields the same result as `parse_file`.
        filter = Vec::new();
        parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data.unwrap().len(), N_STOCKS_IN_RAW_FILE-filter.len());
    }

    #[rstest]
    fn test_ibexparser_filter_wrongfile(wrong_data: Box<&'static Path>) {
        let parser = IbexParser::new();
        let path = *wrong_data;
        let filter: StockData = vec!["AENA".to_string()];

        let parsed_data = parser.filter_file(path, &filter);
        assert_eq!(parsed_data, None);
    }

}