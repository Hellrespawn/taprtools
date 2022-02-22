use crate::cli::validate::validate;
use crate::cli::Filesystem;
use crate::file::AudioFile;
use anyhow::Result;
use file_history::{Action, History};
use std::path::PathBuf;
use tfmt::{Interpreter, Script};

pub(crate) struct SrcTgtPair {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf,
}

pub(crate) struct Rename;

impl Rename {
    pub(crate) fn run(
        preview: bool,
        recursion_depth: usize,
        name: &str,
        arguments: &[String],
    ) -> Result<()> {
        let script = Filesystem::get_script(name)?;

        let files = Rename::gather_files(recursion_depth)?;

        let pairs = Rename::interpret_files(script, arguments, &files)?;

        let (pairs, _filtered_pairs) = Rename::partition_src_tgt_pair(pairs);

        Rename::validate_src_tgt_pairs(&pairs)?;

        let history_path = Filesystem::get_history_path()?;
        let mut history = History::load(&history_path)?;

        let actions: Vec<Action> = pairs
            .into_iter()
            .map(|SrcTgtPair { source, target }| Action::Move {
                source,
                target,
            })
            .collect();

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
    ) -> Result<Vec<SrcTgtPair>> {
        let mut interpreter = Interpreter::new(script, arguments.to_vec())?;

        let actions: Result<Vec<SrcTgtPair>> = files
            .iter()
            .map(|audiofile| {
                Rename::src_target_pair_from_file(&mut interpreter, audiofile)
            })
            .collect();

        actions
    }

    fn src_target_pair_from_file(
        interpreter: &mut Interpreter,
        audiofile: &AudioFile,
    ) -> Result<SrcTgtPair> {
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

        let pair = SrcTgtPair { source, target };

        Ok(pair)
    }

    fn partition_src_tgt_pair(
        actions: Vec<SrcTgtPair>,
    ) -> (Vec<SrcTgtPair>, Vec<SrcTgtPair>) {
        actions
            .into_iter()
            .partition(|pair| pair.source != pair.target)
    }

    fn validate_src_tgt_pairs(pairs: &[SrcTgtPair]) -> Result<()> {
        validate(pairs)
    }
}
