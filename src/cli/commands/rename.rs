use crate::cli::validate::validate;
use crate::cli::Filesystem;
use crate::file::AudioFile;
use anyhow::Result;
use file_history::{Action, History};
use tfmt::{Interpreter, Script};

pub(crate) struct Rename;

impl Rename {
    pub(crate) fn run(
        preview: bool,
        recursion_depth: usize,
        name: &str,
        arguments: &[String],
    ) -> Result<()> {
        let history_path = Filesystem::get_history_path()?;
        let mut history = History::load(&history_path)?;

        let script = Filesystem::get_script(name)?;

        let files = Rename::gather_files(recursion_depth)?;

        let actions = Rename::interpret_files(script, arguments, &files)?;

        Rename::validate_actions(&actions)?;

        let (actions, _filtered_actions) = Rename::partition_actions(actions);

        let result: Result<()> = {
            for action in actions {
                if !preview {
                    history.apply(action)?;
                }
            }

            Ok(())
        };

        // FIXME Handle nested error somehow.
        if result.is_err() {
            history.rollback()?;
        } else {
            history.save()?;
        }

        result
    }

    fn gather_files(recursion_depth: usize) -> Result<Vec<AudioFile>> {
        Filesystem::get_audiofiles(recursion_depth)
    }

    fn interpret_files(
        script: Script,
        arguments: &[String],
        files: &[AudioFile],
    ) -> Result<Vec<Action>> {
        let mut interpreter = Interpreter::new(script, arguments.to_vec())?;

        let actions: Result<Vec<Action>> = files
            .iter()
            .map(|audiofile| {
                Rename::action_from_file(&mut interpreter, audiofile)
            })
            .collect();

        actions
    }

    fn action_from_file(
        interpreter: &mut Interpreter,
        audiofile: &AudioFile,
    ) -> Result<Action> {
        let string = interpreter.interpret(audiofile.tags())?;

        let source = audiofile.path().to_path_buf();

        // We already know this is a file with either an "mp3" or "ogg"
        // extension, so we unwrap safely.
        let extension = source.extension().unwrap();

        // We already know this is a file, so it should have a parent directory.
        let target = source
            .parent()
            .unwrap()
            .join(string)
            .with_extension(extension);

        let action = Action::Move { source, target };

        Ok(action)
    }

    fn validate_actions(actions: &[Action]) -> Result<()> {
        validate(actions)
    }

    fn partition_actions(actions: Vec<Action>) -> (Vec<Action>, Vec<Action>) {
        actions.into_iter().partition(|action| {
            let (source, target) = action.get_src_tgt_unchecked();
            source != target
        })
    }
}
