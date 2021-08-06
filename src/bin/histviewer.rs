use anyhow::Result;
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name,
    crate_version, Arg,
};
use std::path::Path;
use tfmttools::cli::history::{Action, History, Stack};

fn main() -> Result<()> {
    let app = app_from_crate!()
        .arg(
            Arg::with_name("history-file")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Print everything.")
                .short("v")
                .long("verbose"),
        );

    let matches = app.get_matches();
    let histfile = matches.value_of("history-file").expect("No file provided!");
    let verbose = matches.is_present("verbose");

    let string = histviewer(&histfile, verbose)?;

    print!("{}", string);

    Ok(())
}

fn histviewer<P: AsRef<Path>>(path: &P, verbose: bool) -> Result<String> {
    let history = History::load_from_path(true, &path.as_ref())?;

    let mut string = String::new();

    string += &stack_to_string(history.done_stack, "Done", verbose);
    string += &stack_to_string(history.undone_stack, "Undone", verbose);

    Ok(string)
}

fn stack_to_string(stack: Stack, name: &str, verbose: bool) -> String {
    let mut string = String::new();

    if !verbose && !stack.is_empty() {
        string += &format!("{} actions:\n", name);
    }

    for (i, action_group) in stack.iter().enumerate() {
        if verbose {
            string +=
                &format!("{} actions: [{}/{}]:\n", name, i + 1, stack.len());
            for (i, action) in action_group.iter().enumerate() {
                string +=
                    &format!("[{}/{}] {}\n", i + 1, action_group.len(), action)
            }
        } else {
            string += &format!(
                "[{}/{}] {}\n",
                i + 1,
                stack.len(),
                action_group_to_string(action_group)
            )
        }
    }

    string
}

fn action_group_to_string(action_group: &[Action]) -> String {
    let (mut create, mut remove, mut rename) = (0, 0, 0);

    for action in action_group {
        match action {
            Action::CreateDir { .. } => create += 1,
            Action::RemoveDir { .. } => remove += 1,
            Action::Move { .. } => rename += 1,
        }
    }

    format!(
        "ActionGroup: [{}: {} cr, {} rn, {} rm]",
        action_group.len(),
        create,
        rename,
        remove
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use tfmttools::helpers::normalize_newlines;

    #[test]
    fn histviewer_verbose_test() -> Result<()> {
        let string = histviewer(&"testdata/history/test.hist", true)?;
        let reference =
            std::fs::read_to_string("testdata/history/verbose.hist.txt")?;

        println!("{}", string);

        assert_eq!(
            normalize_newlines(&string).trim(),
            normalize_newlines(&reference).trim()
        );

        Ok(())
    }

    #[test]
    fn histviewer_normal_test() -> Result<()> {
        let string = histviewer(&"testdata/history/test.hist", false)?;
        let reference =
            std::fs::read_to_string("testdata/history/ref.hist.txt")?;

        println!("{}", string);

        assert_eq!(
            normalize_newlines(&string).trim(),
            normalize_newlines(&reference).trim()
        );

        Ok(())
    }
}
