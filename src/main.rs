use hex::encode;
use ring::digest::Digest;
use std::env::args;
use std::fs::{File, Metadata, metadata, read_dir};
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
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
fn app_exit(message: &str, code: i32) -> ! {
    eprintln!("{}", message);
    std::process::exit(code);
}

fn strip_name(name: &str) -> &str {
    // Function to strip leading asterisk and whitespace from a filename
    name.trim_start_matches('*')
        .trim_start_matches(".\\")
        .trim()
}
fn main() {
    //parsing arguments
    let arguments = args().collect::<Vec<String>>(); //read arguments
    if arguments.len() < 2 {
        print_help()
    }
    match arguments[1].as_str() {
        "-c" => {
            //if -c (calculate) given and second argument given
            if arguments.len() < 3 {
                app_exit("File name not given", 1);
            }
            let filename = &arguments[2]; // taking second argument
            let filemeta = match metadata(&filename) {
                Ok(meta) => meta,
                Err(e) => {
                    app_exit(&format!("Unable to read {filename}.\nError: {e}"), 6);
                }
            };
            if metadata(&filename).is_ok() {
                //if exist
                calculate_sha256(filename, filemeta);
            }
        }
        "-v" => {
            //if -v (verify) given with second argument
            if arguments.len() < 3 {
                app_exit("SHA256 file name not given", 1);
            }
            let sha256_file = &arguments[2]; //get filename
            if !sha256_file.ends_with(".sha256") {
                app_exit("Not a SHA256 file", 2);
            }
            let sha256_content = match read_text_file_safe(sha256_file) {
                Ok(lines) => lines,
                Err(e) => {
                    app_exit(format!("Failed to read {}: {}", sha256_file, e).as_str(), 6)
                }
            };
            if sha256_content.is_empty() {
                //if empty (will reorganize code later, as this check should be before reading
                app_exit("Empty sha256 file", 3);
            }
            verify_sha256(sha256_content);
        }
        _ => {
            print_help();
        }
    }
}

fn dir_to_vec(filename: &String) -> Vec<String> {
    //function to convert directory content into Vector of strings
    let mut files: Vec<String> = vec![]; //initiating empty array
    let dir_content = read_dir(filename).unwrap(); //reading directory content
    for entry in dir_content {
        //parsing entries
        let entry = entry.unwrap(); //unwrap (rust working in such strange way)
        let path = entry.path(); //get file or dir full path
        if metadata(&path).unwrap().is_file() {
            //if it is a file
            files.push(path.to_str().unwrap().to_string()); //push it into array
        }
    }
    files //return result
}
fn calculate_sha256(filename: &String, filemeta: Metadata) {
    match (filemeta.is_file(), filemeta.is_dir()) {
        (true, false) => {
            //if it is a file
            let sha_sum = encode(sha256_thread(filename).as_ref());
            println!("{sha_sum} *{}", strip_name(filename));
        }
        (false, true) => {
            //if it is a dir
            let files = dir_to_vec(filename);
            multithread_dir(files);
        }
        _ => {
            //neither file nor directory
            let message: String = format!("{} not a file or directory", filename);
            app_exit(message.as_str(), 4);
        }
    }
}
fn multithread_dir(file_paths: Vec<String>) {
    let max_threads: usize = match available_parallelism().unwrap().get() {
        4 => 4,
        1 => 1,
        2..4 => 2,
        n if n > 4 => 4,
        _ => 4
    };

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
                        let sha_sum = encode(sha256_thread(&path_file));
                        println!("{sha_sum} *{file_name}");
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
    const CHUNK_SIZE: usize = 1048576;
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
    println!("Calculates sha256 checksum for file or directory\r\n
Usage:\r
      -c FILENAME or Path - calculates checksum\r
      -v FILENAME.sha256 - verify checksum\r\n
Examples:\r
      sha256sum -c example.iso\r
      sha256sum -c C:\\Downloads\\\r
      sha256sum -v example.sha256\r\r\n
Author: Nikolay Artamonov. MIT License.\r
      https://github.com/helen-beauty/sha256sum");
    std::process::exit(0);
}

fn read_text_file_safe(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Try UTF-8 first
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let content_without_bom = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
            Ok(content_without_bom
                .lines() // Better than split("\n") - handles \r\n
                .map(|line| line.to_string())
                .collect())
        }
        Err(_) => {
            // File is not valid UTF-8, try lossy conversion
            let bytes = fs::read(file_path)?;
            let content = String::from_utf8_lossy(&bytes);

            let content_without_bom = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

            Ok(content_without_bom
                .lines()
                .map(|line| line.to_string())
                .collect())
        }
    }
}

fn verify_sha256(sha256_content: Vec<String>) {
    for item in sha256_content.iter() {
        //iteration to read each string
        if item.len() == 0 {
            //verify if string is not empty (sometimes it happens
            app_exit("No item retrieved from file", 5)
        }
        let sha_str: Vec<&str> = item.splitn(2, " ").collect(); //split string into parts splitn used to split exactly once
        let hash = sha_str[0].to_string().to_lowercase(); //get hash and making it lowercase
        let file_name = sha_str[1].trim_start_matches("*").trim().to_string(); //get filename and trim all unwanted characters
        if metadata(file_name.as_str()).is_ok() {
            //if a file exists and accessible
            let calculated = encode(sha256_thread(&file_name)); //calculating hash
            if calculated == hash {
                //compare hash with calculated
                println!("\"{}\" OK", file_name);
            } else {
                println!("\"{file_name}\". FAIL");
            }
        } else {
            println!("\"{file_name}\". Cannot read");
        }
    }
}
