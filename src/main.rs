use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::task;

async fn process_chunk(chunk: &[u8], matrix: Arc<RwLock<HashMap<(usize, usize), f64>>>) {
    let mut lines = chunk.split(|&b| b == b'\n');
    // Skip any partial line at the beginning of the chunk
    lines.next();

    for line in lines {
        let entry: Vec<&[u8]> = line.split(|&b| b == b' ').collect();
        if entry.len() == 3 {
            let row = std::str::from_utf8(entry[0]).unwrap().parse::<usize>().unwrap() - 1; // 0-based index
            let col = std::str::from_utf8(entry[1]).unwrap().parse::<usize>().unwrap() - 1; // 0-based index
            let value = std::str::from_utf8(entry[2]).unwrap().parse::<f64>().unwrap();

            let mut matrix = matrix.write().unwrap();
            matrix.insert((row, col), value);
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

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [matrix-market-filename]", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    let file_size = tokio::fs::metadata(filename).await.unwrap().len() as usize;

    let buffer = read_file_chunk(filename, 0, file_size).await.unwrap_or_else(|_| {
        eprintln!("Could not read file: {}", filename);
        std::process::exit(1);
    });

    let header_end = buffer.iter().position(|&b| b == b'\n').unwrap_or(buffer.len());
    let header = &buffer[..header_end];
    let body = &buffer[header_end + 1..];

    if !contains_sequence(header, b"%%MatrixMarket") {
        eprintln!("Could not process Matrix Market banner.");
        std::process::exit(1);
    }

    if !contains_sequence(header, b"matrix") || !contains_sequence(header, b"coordinate") || !contains_sequence(header, b"real") {
        eprintln!("Unsupported Matrix Market type: [{}]", std::str::from_utf8(header).unwrap().trim());
        std::process::exit(1);
    }

    let dims_end = body.iter().position(|&b| b == b'\n').unwrap_or(body.len());
    let dims = &body[..dims_end];
    let body = &body[dims_end + 1..];

    let dims: Vec<&[u8]> = dims.split(|&b| b == b' ').collect();
    let m: usize = std::str::from_utf8(dims[0]).unwrap().parse().unwrap();
    let n: usize = std::str::from_utf8(dims[1]).unwrap().parse().unwrap();
    let nz: usize = std::str::from_utf8(dims[2]).unwrap().parse().unwrap();

    let matrix = Arc::new(RwLock::new(HashMap::new()));

    let num_threads = num_cpus::get();
    let chunk_size = body.len() / num_threads;
    let mut tasks = Vec::with_capacity(num_threads);

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = if i == num_threads - 1 { body.len() } else { start + chunk_size };
        let chunk = body[start..end].to_vec(); // Clone chunk data
        let matrix = Arc::clone(&matrix);

        tasks.push(task::spawn(async move {
            process_chunk(&chunk, matrix).await;
        }));
    }

    futures::future::join_all(tasks).await;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    writeln!(handle, "{}", std::str::from_utf8(header).unwrap().trim()).unwrap();
    writeln!(handle, "{} {} {}", m, n, matrix.read().unwrap().len()).unwrap();
    // for ((row, col), value) in matrix.read().unwrap().iter() {
    //     writeln!(handle, "{} {} {:.19}", row + 1, col + 1, value).unwrap();
    // }
}
