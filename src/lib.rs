// Copyright 2024 Felipe Torres González

pub mod parser_ibex;

use std::path::{
    Path,
    PathBuf
};

/// Discover files that contain raw data for the stock prices of the Ibex 35.
///
/// # Description
///
/// This function scans a directory non-recursively and builds a vector that contains the
/// file names (stem + extension) of the files that contain stock data.
///
/// This function **does not** analyzes the content of the files, it rather filters using:
/// - First, a string that indicates the extension that the files must have.
/// - Second, a string that indicates the beginning of the file names.
///
/// ## Arguments
///
/// - `path` an instance of the struct `Path` that points to the directory that needs to be
///    analysed.
/// - `filter` a wrapped string slice that can contain the constant part of the files that
///    should be marked. For example, if the data files have this naming schema: `name(N).ext`,
///    the part `name` should be used as filter. If `None` is passed, the default filter will
///    be used: `data_ibex`.
/// - `format` a wrapped string slice that indicates the extension of the files that should be
///    marked. For example, if the data files have this naming schema: `name(N).ext`,
///    the part `ext` should be used as format. If `None` is passed, the default filter will
///    be used: `csv`.
///
/// ## Preconditions
///
/// `path` must be initialized to a valid directory in which the user running the application
/// has permissions for reading.
///
/// ## Return
///
/// A vector of strings is returned containing the entire file names of the files found that
/// satisfy the given filters (filter and format).
///
/// # Example of use
///
/// For example, let's consider a directory containing the following items:
///
/// ```text
/// ➜ ll
/// total 24K
/// -rw-r--r--. 1 felipe felipe  155 Feb  6 17:38  Cargo.lock
/// -rw-r--r--. 1 felipe felipe  180 Feb  6 17:22  Cargo.toml
/// -rw-r--r--. 1 felipe felipe 3.3K Feb  6 15:24 'data_ibex(1).csv'
/// -rw-r--r--. 1 felipe felipe 3.3K Feb  6 15:29 'data_ibex(2).csv'
/// -rw-r--r--. 1 felipe felipe 3.3K Feb  6 15:35 'data_ibex(3).csv'
/// -rw-r--r--. 1 felipe felipe 3.3K Feb  6 15:18  data_ibex.csv
/// drwxr-xr-x. 1 felipe felipe   26 Feb  6 17:34  src
/// drwxr-xr-x. 1 felipe felipe  114 Feb  7 09:11  target
/// ```
///
/// Files named `data_ibex` contain raw data for the stocks that are included in the index.
/// We aim to curate a list that only contains those files. To do so, we can run `discover`
/// over that path:
///
/// ```rust
/// use ibex_parser::discover;
/// use std::path::Path;
///
/// let path = Path::new("./");
/// let files = discover(path, None, None);
/// println!("{:?}", files);
/// ```
///
/// As those files use the default filter and extension, we have no need to specify those
/// when calling the function `discover`.
pub fn discover(path: &Path, filter: Option<&str>, format: Option<&str>) -> Vec<String> {
    let filter = if let Some(x) = filter {
        String::from(x)
    } else {
        String::from("data_ibex")
    };

    let file_format = if let Some(x) = format {
        String::from(x)
    } else {
        String::from("csv")
    };

    let mut files: Vec<String> = Vec::new();

    for entry in path.read_dir().expect("Can't read the directory") {
        if let Ok(entry) = entry {
            if entry.metadata().unwrap().is_file() {
                // An owned version of a Path.
                let cur_file: PathBuf = entry.path();

                // Avoid panicking when a file without format is found.
                let extension = if let Some(x) = cur_file.extension() {
                    x.to_str().unwrap()
                } else {
                    "_"
                };

                if extension == file_format &&
                   filter == cur_file.file_stem().unwrap().to_str().unwrap()[..filter.len()] {
                    files.push(
                        String::from(cur_file.file_name().unwrap().to_str().unwrap())
                    );
                } else {
                    continue;
                }
            }
        }
    }

    files
}