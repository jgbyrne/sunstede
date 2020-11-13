use std::fs;
use std::io;
use std::path;
use crate::config::Site;

pub struct GenError {
    pub msg: String,
}

impl From<io::Error> for GenError {
    fn from(error: io::Error) -> Self {
        GenError {
           msg: format!("IO Error: {}", error),
        }
    }
}

pub fn generate(site: Site) -> Result<(), GenError> {
    let site_dir = path::Path::new("sites").join(site.key);
    for entry in fs::read_dir(site_dir)? {
        let entry = entry?;
        if entry.path().is_file() {
            let buf = fs::read_to_string(entry.path());
            println!("{:?}", buf);
        }
    }

    Ok(())
}
