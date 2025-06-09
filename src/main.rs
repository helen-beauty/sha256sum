use std::env::args;
//use std::fmt::Debug;
use std::fs::{metadata, File};
use std::io;
use std::io::{BufReader};
use std::time::Instant;
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
                if filemeta.is_file() { //if it is a file
                    println!("{filename} size {} bytes", filemeta.len()); //print some intro;
                    let file_stream = ChunkStream::new(filename, 8192);
                    let mut context = ring::digest::Context::new(&ring::digest::SHA256);
                    //string for profiling
                    let start = Instant::now();                    

                    for (i, chunk_result) in file_stream.unwrap().enumerate() {
                        match chunk_result {
                            Ok(chunk) => { context.update(&chunk); }
                            Err(e) => eprintln!("Error reading chunk {}: {}", i + 1, e),
                        }
                    }
                    let sha_sum = context.finish();
                    
                    //print profiling data
                    println!("Elapsed: {:?}", start.elapsed());
                    println!("{:?}", sha_sum);
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
