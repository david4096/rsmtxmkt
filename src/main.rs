use clap::{Arg, Command};
use std::io::{self, Write};
use rsmtxmkt::load_matrix_market; // Update `your_crate_name` with your actual crate name

/// Main function to execute the program, which loads a Matrix Market file and outputs matrix dimensions and non-zero count.
#[tokio::main]
async fn main() {
    let matches = Command::new("Matrix Market Loader")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Loads a Matrix Market file and outputs matrix dimensions and non-zero count.")
        .arg(
            Arg::new("filename")
                .help("The path to the Matrix Market file")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Extract filename and handle potential errors
    // Extract the filename argument
    let filename = matches.get_one::<String>("filename").unwrap_or_else(|| {
        eprintln!("Filename argument is required.");
        std::process::exit(1);
    });

    match load_matrix_market(filename).await {
        Ok(cs_matrix) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{} {} {}", cs_matrix.rows(), cs_matrix.cols(), cs_matrix.nnz()).unwrap();
            // Uncomment the following block to output the matrix entries
            // for (value, (row, col)) in cs_matrix.iter() {
            //     writeln!(handle, "{} {} {:.19}", row + 1, col + 1, value).unwrap();
            // }
        }
        Err(e) => {
            eprintln!("Failed to load matrix: {}", e);
            std::process::exit(1);
        }
    }
}
