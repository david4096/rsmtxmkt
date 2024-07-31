// https://math.nist.gov/MatrixMarket/mmio-c.html
use std::env;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::task;
use sprs::{CsMat, CsMatBase};

async fn process_chunk(chunk: &[u8], rows: &mut Vec<usize>, cols: &mut Vec<usize>, values: &mut Vec<f64>) {
    let mut lines = chunk.split(|&b| b == b'\n');
    // Skip any partial line at the beginning of the chunk
    lines.next();

    for line in lines {
        let entry: Vec<&[u8]> = line.split(|&b| b == b' ').collect();
        if entry.len() == 3 {
            let row = std::str::from_utf8(entry[0]).unwrap().parse::<usize>().unwrap() - 1; // 0-based index
            let col = std::str::from_utf8(entry[1]).unwrap().parse::<usize>().unwrap() - 1; // 0-based index
            let value = std::str::from_utf8(entry[2]).unwrap().parse::<f64>().unwrap();

            rows.push(row);
            cols.push(col);
            values.push(value);
        }
    }
}

async fn read_file_chunk(filename: &str, start: usize, end: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(filename).await?;
    file.seek(tokio::io::SeekFrom::Start(start as u64)).await?;
    let mut buffer = vec![0; end - start];
    file.read_exact(&mut buffer).await?;
    Ok(buffer)
}

fn contains_sequence(haystack: &[u8], needle: &[u8]) -> bool {
    needle.len() <= haystack.len() && haystack.windows(needle.len()).any(|window| window == needle)
}

async fn load_matrix_market(filename: &str) -> Result<CsMat<f64>, std::io::Error> {
    let file_size = tokio::fs::metadata(filename).await?.len() as usize;

    let buffer = read_file_chunk(filename, 0, file_size).await?;

    let header_end = buffer.iter().position(|&b| b == b'\n').unwrap_or(buffer.len());
    let header = &buffer[..header_end];
    let body = &buffer[header_end + 1..];

    if !contains_sequence(header, b"%%MatrixMarket") {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Matrix Market banner"));
    }

    if !contains_sequence(header, b"matrix") || !contains_sequence(header, b"coordinate") || !contains_sequence(header, b"real") {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported Matrix Market type"));
    }

    let dims_end = body.iter().position(|&b| b == b'\n').unwrap_or(body.len());
    let dims = &body[..dims_end];
    let body = &body[dims_end + 1..];

    let dims: Vec<&[u8]> = dims.split(|&b| b == b' ').collect();
    let m: usize = std::str::from_utf8(dims[0]).unwrap().parse().unwrap();
    let n: usize = std::str::from_utf8(dims[1]).unwrap().parse().unwrap();
    let nz: usize = std::str::from_utf8(dims[2]).unwrap().parse().unwrap();

    let matrix = Arc::new(RwLock::new((Vec::new(), Vec::new(), Vec::new())));

    let num_threads = num_cpus::get();
    let chunk_size = body.len() / num_threads;
    let mut tasks = Vec::with_capacity(num_threads);

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = if i == num_threads - 1 { body.len() } else { start + chunk_size };
        let chunk = body[start..end].to_vec();
        let matrix = Arc::clone(&matrix);

        tasks.push(task::spawn(async move {
            let mut rows = Vec::new();
            let mut cols = Vec::new();
            let mut values = Vec::new();
            process_chunk(&chunk, &mut rows, &mut cols, &mut values).await;

            let mut matrix = matrix.write().unwrap();
            matrix.0.extend(rows);
            matrix.1.extend(cols);
            matrix.2.extend(values);
        }));
    }

    futures::future::join_all(tasks).await;

    let (rows, cols, values) = {
        let matrix = matrix.read().unwrap();
        (matrix.0.clone(), matrix.1.clone(), matrix.2.clone())
    };

    let mut entries: Vec<(usize, usize, f64)> = rows.into_iter()
        .zip(cols.into_iter())
        .zip(values.into_iter())
        .map(|((row, col), value)| (row, col, value))
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let (sorted_rows, sorted_cols, sorted_values): (Vec<_>, Vec<_>, Vec<_>) = entries.into_iter()
        .map(|(row, col, value)| (row, col, value))
        .fold((Vec::new(), Vec::new(), Vec::new()), |(mut rows, mut cols, mut values), (row, col, value)| {
            rows.push(row);
            cols.push(col);
            values.push(value);
            (rows, cols, values)
        });

    let mut indptr = vec![0; m + 1];
    let mut current_row = 0;
    let mut count = 0;
    for row in sorted_rows {
        while current_row < row {
            indptr[current_row + 1] = count;
            current_row += 1;
        }
        count += 1;
    }
    indptr[current_row + 1] = count;

    Ok(CsMat::new(
        (m, n),
        indptr,
        sorted_cols,
        sorted_values
    ))
}


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [matrix-market-filename]", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    match load_matrix_market(filename).await {
        Ok(cs_matrix) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{} {} {}", cs_matrix.rows(), cs_matrix.cols(), cs_matrix.nnz()).unwrap();
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

