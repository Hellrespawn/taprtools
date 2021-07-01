use anyhow::Result;
use clap::{App, load_yaml};

pub fn main() -> Result<()> {
    let yaml = load_yaml!("tag_to_filename.yml");
    let matches = App::from_yaml(yaml).get_matches();

    println!("{:#?}", matches);

    Ok(())
}
