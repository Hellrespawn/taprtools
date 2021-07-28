use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn init_logger() {
    tfmttools::cli::logging::setup_logger(5, "tfmttools-test")
        .expect("Error in setup_logger");
}

#[allow(dead_code)]
pub fn get_script(filename: &str) -> Result<String> {
    let mut path = PathBuf::from("testdata/script");
    path.push(filename);

    Ok(fs::read_to_string(path)?)
}
