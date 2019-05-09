mod utils;

use utils::file_reader;

pub fn main() {
    println!("Hello world");
    let file_content = file_reader::read_ppm_file("resources/file.txt").ok().unwrap();
    println!("File Content: {}", file_content);
}