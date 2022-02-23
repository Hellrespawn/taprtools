use crate::cli::validate::validate;
use crate::cli::Config;
use crate::file::AudioFile;
use anyhow::Result;
use file_history::{Action, History};
use tfmt::{Interpreter, Script};

pub(crate) fn rename(
    preview: bool,
    config: &Config,
    recursion_depth: usize,
    name: &str,
    arguments: &[String],
) -> Result<()> {
    let history_path = config.get_history_path();
    let mut history = History::load(&history_path)?;

    let script = config.get_script(name)?;

    let files = gather_files(recursion_depth)?;

    println!("Gathered {} audio files.", files.len());

    let actions = interpret_files(script, arguments, &files)?;

    validate_actions(&actions)?;

    let (actions, _filtered_actions) = partition_actions(actions);

    let pp = if preview { Config::PREVIEW_PREFIX } else { "" };
    println!("{pp}Moving {} audio files.", actions.len());

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
    Config::get_audiofiles(recursion_depth)
}

fn interpret_files(
    script: Script,
    arguments: &[String],
    files: &[AudioFile],
) -> Result<Vec<Action>> {
    let mut interpreter = Interpreter::new(script, arguments.to_vec())?;

    let actions: Result<Vec<Action>> = files
        .iter()
        .map(|audiofile| action_from_file(&mut interpreter, audiofile))
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
