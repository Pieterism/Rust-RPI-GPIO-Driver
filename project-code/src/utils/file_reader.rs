use std::fs::File;
use std::io::Read;

pub fn read_ppm_file(path: &str) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[test]
fn can_read_file() {
    let result = read_ppm_file("resources/file.txt");
    assert!(result.is_ok());
    assert_eq!(result.ok().unwrap(), "Hello world!");
}

#[test]
fn can_read_file_no_such_file() {
    let result = read_ppm_file("resources/no-such-file-exists.txt");
    assert!(result.is_err());
}