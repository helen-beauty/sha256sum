use std::env::args;
use std::fs::{metadata, File};
use std::{fs, io};
use std::io::{BufReader};
use std::time::Instant;
use ring::digest::Digest;
use std::thread::{available_parallelism};

//use ring::digest::{digest, SHA256};

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
                let default_parallelism_approx = available_parallelism().unwrap().get();
                println!("Default parallelism: {}", default_parallelism_approx);
                if filemeta.is_file() { //if it is a file
                    println!("{filename} size {} bytes", filemeta.len()); //print some intro;
                        //string for profiling
                        let start = Instant::now();
                        let sha_sum = sha256_thread(filename);
                        println!("{:?}", sha_sum);
                        //print profiling data
                        println!("Elapsed: {:?}", start.elapsed());
                }
                else if filemeta.is_dir() {
                    let dir_content =  fs::read_dir(filename).unwrap();
                    for entry in dir_content {
                        let file_name = entry.unwrap().file_name().into_string().unwrap();
                        let path_file: &mut String =  &mut String::from(filename);
                        path_file.push('\\');
                        path_file.push_str(file_name.as_str());
                        let sha_sum = sha256_thread(path_file);
                        println!("{:?} {}", sha_sum, file_name);
                    }
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
