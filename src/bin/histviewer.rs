use anyhow::Result;
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name,
    crate_version, Arg,
};
use tfmttools::cli::history::History;
use std::path::Path;

fn main() -> Result<()> {
    let app = app_from_crate!()
        .arg(
            Arg::with_name("history-file")
                .help("Sets the input file to use")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("verbose")
                .help("Print everything.")
                .short("v")
                .long("verbose")
        );

    let matches = app.get_matches();
    let histfile = matches.value_of("history-file").expect("No file provided!");
    let verbose = matches.is_present("verbose");

    let string = histviewer(&histfile, verbose)?;

    println!("{}", string);

    Ok(())
}

fn histviewer<P: AsRef<Path>>(path: &P, verbose: bool) -> Result<String> {
    let history = History::load_from_path(true, &path.as_ref())?;

    let mut string = String::new();

    if !verbose {
        string += "Done actions:\n";
    }

    for (i, action_group) in history.done_stack.iter().enumerate() {
        if verbose {
            string += &format!("Done actions: [{}/{}]:\n", i + 1, history.done_stack.len());
            for (i, action) in action_group.iter().enumerate() {
                string += &format!("[{}/{}] {}\n", i + 1, action_group.len(), action)
            }
        } else {
            string += &format!("[{}/{}] {}\n", i + 1, history.done_stack.len(), action_group)
        }
    }

    Ok(string)
}

#[cfg(test)]
mod test {
    use super::*;
    use tfmttools::helpers::normalize_newlines;

    #[test]
    fn histviewer_verbose_test() -> Result<()> {
        let string = histviewer(&"testdata/history/test.hist", true)?;
        let reference = std::fs::read_to_string("testdata/history/verbose.hist.txt")?;

        assert_eq!(normalize_newlines(&string).trim(), normalize_newlines(&reference).trim());

        Ok(())
    }

    #[test]
    fn histviewer_normal_test() -> Result<()> {
        let string = histviewer(&"testdata/history/test.hist", false)?;
        let reference = std::fs::read_to_string("testdata/history/ref.hist.txt")?;

        assert_eq!(normalize_newlines(&string).trim(), normalize_newlines(&reference).trim());

        Ok(())

    }
}
