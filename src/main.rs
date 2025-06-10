use std::env::args;
use std::fs::{metadata, File};
use std::{fs, io, thread};
use std::io::{BufReader};
use std::time::Instant;
use ring::digest::Digest;
use std::thread::{available_parallelism};
use std::sync::{Arc, Mutex};
use hex;
use hex::encode;

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
    if  args().len() < 2 {
        print_help();
    }
    //arguments given
    else if  args().len() > 2 {
        let arguments = args().collect::<Vec<String>>(); //read arguments
        //checking first argument
        if arguments[1] == String::from("-c") { //if -c (calculate) given and second argument given
            let filename = &arguments[2]; // taking second argument
            let filemeta = metadata(&filename).unwrap();
            if   metadata(&filename).is_ok() { //if exist
                if filemeta.is_file() { //if it is a file
                        //string for profiling
                        let start = Instant::now();
                        let sha_sum = encode(sha256_thread(filename).as_ref());
                        println!("{}", sha_sum);
                        //print profiling data
                        println!("Elapsed: {:?}", start.elapsed());
                }
                else if filemeta.is_dir() {
                    let start = Instant::now();
                    multithread_dir(&filename);
                    println!("Elapsed: {:?}", start.elapsed());
                }
                else {
                    exit_message("Not a file");
                }

            }
            else {
                exit_message("File not found or not accessible");
            }
        }
        else {
            print_help();
        }

    }
    else {
        print_help();
    }
}

fn multithread_dir(filename: &&String) {
    let max_threads: usize = available_parallelism().unwrap().get();
    println!("Threads: {}", max_threads);
    let dir_content: Vec<_> = fs::read_dir(filename).unwrap()
        .map(|entry| entry.unwrap())
        .collect();

    let files = Arc::new(Mutex::new(dir_content.into_iter()));
    let mut handles = Vec::new();

    for _ in 0..max_threads {
        let files = Arc::clone(&files);
        let filename = filename.to_string(); // Clone filename for thread

        let handle = thread::spawn(move || {
            loop {
                let entry = {
                    let mut files_guard = files.lock().unwrap();
                    files_guard.next()
                };

                match entry {
                    Some(entry) => {
                        let file_name = entry.file_name().into_string().unwrap();
                        let mut path_file = String::from(&filename);
                        path_file.push('\\');
                        path_file.push_str(&file_name);

                        let file_meta_data = metadata(&path_file).unwrap();
                        if file_meta_data.is_file() {
                            let sha_sum = encode(sha256_thread(&path_file));
                            println!("{} {}", sha_sum, file_name);
                        }
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
            Ok(chunk) => { context.update(&chunk); }
            Err(e) => eprintln!("Error reading chunk {}: {}", i + 1, e),
        }
    }
    let sha_sum = context.finish();
    sha_sum
}

fn print_help() {
    println!("Calculates sha256 checksum for file\n
        Usage:\n
        -c FILENAME  calculates checksum\n
        -v FILENAME.sha256 - verify checksum\n");
    std::process::exit(0);
}

fn exit_message(message: &str) {
    println!("Program exited. {message}\n");
}
