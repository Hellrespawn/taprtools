use super::inspector::Inspector;
use std::path::Path;
pub enum Strings<'a> {
    ArgparseCreatedDir(&'a Path),
    ClearHistoryCantFindHistory,
    ClearHistoryError(&'a str),
    HistoryNothingToDo(&'a str),
    HistoryOnlyNToDo(u64, &'a str),
    HistoryDoingNTimes(u64, &'a str),
    Inspector(&'a Inspector<'a>),
    ListScripts,
    RenameIgnoringOutputFolder(&'a Path),
}

impl<'a> Strings<'a> {
    pub fn eprint(&self) {
        self.print(log::Level::Error)
    }

    pub fn wprint(&self) {
        self.print(log::Level::Warn)
    }

    pub fn iprint(&self) {
        self.print(log::Level::Info)
    }

    pub fn dprint(&self) {
        self.print(log::Level::Debug)
    }

    pub fn tprint(&self) {
        self.print(log::Level::Trace)
    }

    fn print(&self, level: log::Level) {
        let string = match self {
            Self::ArgparseCreatedDir(p) => {
                format!(
                    "Creating configuration directory at \"{}\"",
                    p.to_string_lossy()
                )
            }

            Self::ClearHistoryCantFindHistory => {
                "Can't find history file to clear!".to_string()
            }

            Self::ClearHistoryError(e) => {
                format!("Error while trying to clear history!\n{}", e)
            }

            Self::HistoryNothingToDo(s) => {
                format!("There is nothing to {}.", s)
            }
            Self::HistoryOnlyNToDo(n, s) => {
                format!("Warning: there are only {} actions to {}.", n, s)
            }
            Self::HistoryDoingNTimes(n, s) => {
                format!("{}ing {} times...", Strings::titlecase(s), n)
            }
            Self::Inspector(i) => {
                // FIXME info for inspected, debug for output?
                log::log!(level, "Inspected script \"{}\"", i.name);
                format!("{}", i)
            }

            Self::ListScripts => "Couldn't find any scripts.".to_string(),

            Self::RenameIgnoringOutputFolder(p) => {
                format!(
                    "Absolute path found, ignoring --output-folder {}",
                    p.to_string_lossy()
                )
            }
        };

        log::log!(level, "stdout: {}", string);
        println!("{}", string);
    }

    fn titlecase(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
