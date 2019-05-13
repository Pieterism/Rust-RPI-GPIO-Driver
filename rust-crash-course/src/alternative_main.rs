use utils::file_reader;
use std::path::Path;

pub fn main() {
    let path = Path::new("/Users/wdeceuninck/IdeaProjects/veiligesoftware-20182019-groep7/project-code/resources/testbeeld.ppm");
    file_reader::read_ppm_file(&path);
}