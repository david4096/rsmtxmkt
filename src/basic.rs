use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process::exit;

fn main() {
    // Get the filename from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [matrix-market-filename]", args[0]);
        exit(1);
    }
    let filename = &args[1];

    // Open the file
    let file = File::open(filename).unwrap_or_else(|_| {
        eprintln!("Could not open file: {}", filename);
        exit(1);
    });
    let mut reader = io::BufReader::new(file);

    // Read the banner
    let mut banner = String::new();
    reader.read_line(&mut banner).unwrap();
    if !banner.starts_with("%%MatrixMarket") {
        eprintln!("Could not process Matrix Market banner.");
        exit(1);
    }

    // Validate banner (this is simplified; real validation needed)
    if !banner.contains("matrix") || !banner.contains("coordinate") || !banner.contains("real") {
        eprintln!("Unsupported Matrix Market type: [{}]", banner.trim());
        exit(1);
    }

    // Read matrix dimensions
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let dims: Vec<&str> = line.trim().split_whitespace().collect();
    if dims.len() != 3 {
        eprintln!("Error reading matrix dimensions");
        exit(1);
    }
    let m: usize = dims[0].parse().unwrap();
    let n: usize = dims[1].parse().unwrap();
    let nz: usize = dims[2].parse().unwrap();

    // Allocate memory for matrix
    let mut rows = vec![0; nz];
    let mut cols = vec![0; nz];
    let mut values = vec![0.0; nz];

    // Read matrix entries
    for i in 0..nz {
        line.clear();
        reader.read_line(&mut line).unwrap();
        let entry: Vec<&str> = line.trim().split_whitespace().collect();
        if entry.len() != 3 {
            eprintln!("Error reading matrix entry");
            exit(1);
        }
        rows[i] = entry[0].parse::<usize>().unwrap() - 1; // 0-based index
        cols[i] = entry[1].parse::<usize>().unwrap() - 1; // 0-based index
        values[i] = entry[2].parse::<f64>().unwrap();
    }

    // Write out matrix
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    writeln!(handle, "{}", banner.trim()).unwrap();
    writeln!(handle, "{} {} {}", m, n, nz).unwrap();
    for i in 0..nz {
        writeln!(handle, "{} {} {:.19}", rows[i] + 1, cols[i] + 1, values[i]).unwrap();
    }
}
