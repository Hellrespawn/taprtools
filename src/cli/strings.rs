use log::info;
use std::path::Path;
use super::inspector::Inspector;

pub enum Strings<'a> {
    ArgparseCreatedDir(&'a Path),
    HistoryNothingToDo(&'a str),
    HistoryOnlyNToDo(u64, &'a str),
    HistoryDoingNTimes(u64, &'a str),
    Inspector(&'a Inspector<'a>),

}

impl<'a> Strings<'a> {
    fn titlecase(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn print(&self) {
        let string = match self {
            Self::ArgparseCreatedDir(p) => {
                format!(
                    "Creating configuration directory at \"{}\"",
                    p.to_string_lossy()
                )
            }
            Self::HistoryNothingToDo(s) => {
                format!("There is nothing to {}.", s)
            }
            Self::HistoryOnlyNToDo(n, s) => {
                format!("Warning: there are only {} actions to {}.", n, s)
            }
            Self::HistoryDoingNTimes(n, s) => {
                format!("{}ing {} times...", Strings::titlecase(s), n)
            },
            Self::Inspector(i) => {
                info!("Inspected script \"{}\"", i.name);
                format!("{}", i)
            }
        };

        info!("stdout: {}", string);
        println!("{}", string);
    }
}
