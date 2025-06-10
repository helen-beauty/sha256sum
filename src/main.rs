use hex::encode;
use ring::digest::Digest;
use std::env::args;
use std::fs::{File, metadata, read_dir};
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use std::time::Instant;
use std::{fs, io, thread};

struct ChunkStream {
    reader: BufReader<File>,
    buffer: Vec<u8>,
    //    chunk_size: usize,
}

impl ChunkStream {
    fn new(filename: &str, chunk_size: usize) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(ChunkStream {
            reader,
            buffer: vec![0; chunk_size],
            //            chunk_size,
        })
    }
}

impl Iterator for ChunkStream {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::Read;

        match self.reader.read(&mut self.buffer) {
            Ok(0) => None, // EOF
            Ok(n) => Some(Ok(self.buffer[..n].to_vec())),
            Err(e) => Some(Err(e)),
        }
    }
}

fn main() {
    //if no arguments, print help and exit
    if args().len() < 2 {
        print_help();
    }
    //arguments given
    else if args().len() > 2 {
        let arguments = args().collect::<Vec<String>>(); //read arguments
        //checking first argument
        if arguments[1] == String::from("-c") {
            //if -c (calculate) given and second argument given
            let filename = &arguments[2]; // taking second argument
            let filemeta = metadata(&filename).unwrap();
            if metadata(&filename).is_ok() {
                //if exist
                if filemeta.is_file() {
                    //if it is a file
                    let start = Instant::now();
                    let sha_sum = encode(sha256_thread(filename).as_ref());
                    println!("{sha_sum} *{filename}");
                    //print profiling data
                    println!("Elapsed: {:?}", start.elapsed());
                } else if filemeta.is_dir() {
                    let start = Instant::now();
                    let files = dir_to_vec(filename);
                    multithread_dir(files);
                    println!("Elapsed: {:?}", start.elapsed());
                } else {
                    exit_message("Not a file");
                }
            } else {
                exit_message("File not found or not accessible");
            }
        } else if arguments[1] == String::from("-v") {
            //if -v (verify) given with second argument
            let sha256_file = &arguments[2];
            if sha256_file.ends_with(".sha256") {
                let sha256_content: Vec<String> = fs::read_to_string(sha256_file)
                    .expect("Failed to read input")
                    .split("\n")
                    .map(|line| line.to_string())
                    .collect();
                if sha256_content.is_empty() {
                    exit_message("Empty sha256 file");
                }

                for item in sha256_content.iter() {
                    if item.len() > 0 {
                        let sha_str: Vec<&str> = item.splitn(2, " ").collect();
                        let hash = sha_str[0].trim();
                        let file_name = sha_str[1].trim_start_matches("*").trim().to_string();
                        let calculated = encode(sha256_thread(&file_name).as_ref());
                        if calculated == hash {
                            println!("OK")
                        } else {
                            println!("FAIL")
                        }
                    }
                }
            } else {
                exit_message("Not a SHA256 file");
            }
        } else {
            print_help();
        }
    } else {
        print_help();
    }
}

fn dir_to_vec(filename: &String) -> Vec<String> {
    let mut files: Vec<String> = vec![];
    let dir_content = read_dir(filename).unwrap();
    for entry in dir_content {
        let entry = entry.unwrap();
        let path = entry.path();
        if metadata(&path).unwrap().is_file() {
            files.push(path.to_str().unwrap().to_string());
        }
    }
    files
}

fn multithread_dir(file_paths: Vec<String>) {
    let max_threads: usize = available_parallelism().unwrap().get();
    println!("Threads: {}", max_threads);

    // Create shared iterator from the vector of file paths
    let files = Arc::new(Mutex::new(file_paths.into_iter()));
    let mut handles = Vec::new();

    for _ in 0..max_threads {
        let files = Arc::clone(&files);

        let handle = thread::spawn(move || {
            loop {
                let file_path = {
                    let mut files_guard = files.lock().unwrap();
                    files_guard.next()
                };

                match file_path {
                    Some(path_file) => {
                        // Extract just the filename for display
                        let file_name = std::path::Path::new(&path_file)
                            .file_name()
                            .unwrap()
                            .to_string_lossy();

                        // Check if it's a file before processing
                        //let file_meta_data = metadata(&path_file).unwrap();
                        //if file_meta_data.is_file() {
                        let sha_sum = encode(sha256_thread(&path_file));
                        println!("{sha_sum} *{file_name}");
                        //}
                    }
                    None => break, // No more files to process
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

fn sha256_thread(filename: &String) -> Digest {
    const CHUNK_SIZE: usize = 8192;
    let file_stream = ChunkStream::new(filename, CHUNK_SIZE);
    let mut context = ring::digest::Context::new(&ring::digest::SHA256);
    for (i, chunk_result) in file_stream.unwrap().enumerate() {
        match chunk_result {
            Ok(chunk) => {
                context.update(&chunk);
            }
            Err(e) => eprintln!("Error reading chunk {}: {}", i + 1, e),
        }
    }
    let sha_sum = context.finish();
    sha_sum
}

fn print_help() {
    println!(
        "Calculates sha256 checksum for file\n
        Usage:\n
        -c FILENAME  calculates checksum\n
        -v FILENAME.sha256 - verify checksum\n"
    );
    std::process::exit(0);
}

fn exit_message(message: &str) {
    println!("Program exited. {message}\n");
    print_help();
}
