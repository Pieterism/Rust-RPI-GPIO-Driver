mod utils;

use utils::file_reader;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use std::error::Error;

fn main()
{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Syntax: {} <filename>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path)    {
        Err(why) => panic!("Could not open file: {} (Reason: {})",
                           display, why.description()),
        Ok(file) => file
    };

    // read the full file into memory. panic on failure
    let mut raw_file = Vec::new();
    file.read_to_end(&mut raw_file).unwrap();

    // construct a cursor so we can seek in the raw buffer
    let mut cursor = Cursor::new(raw_file);
    let mut image = match file_reader::decode_ppm_image(&mut cursor) {
        Ok(img) => img,
        Err(why) => panic!("Could not parse PPM file - Desc: {}", why.description()),
    };

    file_reader::show_image(&image);
}