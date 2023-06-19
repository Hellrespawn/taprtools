#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_wraps)]

use crate::{file::AudioFile, tags::Tags};
use conv::ConvAsUtil;
use std::sync::Arc;
use tapr::{
    Arguments, Callable, CallableType, Environment, Interpreter,
    NativeFunction, Parameters, TaprErrorKind, TaprResult, Value,
};

pub(crate) fn get_tapr_environment(audiofile: AudioFile) -> Environment {
    let mut env = Environment::new();

    let tag_function =
        Value::Callable(Arc::new(TagsFunction(Box::new(audiofile))));

    env.insert("tags".to_owned(), tag_function).unwrap();

    env.insert(
        "zero-pad".to_owned(),
        NativeFunction::new(
            "zero-pad",
            align,
            "width:number s:string".try_into().unwrap(),
        )
        .into(),
    )
    .unwrap();

    env.insert(
        "parse-tag-date".to_owned(),
        NativeFunction::new(
            "parse-date",
            parse_date,
            "s:string".try_into().unwrap(),
        )
        .into(),
    )
    .unwrap();

    env
}

struct TagsFunction(pub Box<dyn Tags>);

impl Callable for TagsFunction {
    fn call(
        &self,
        _: &mut Interpreter,
        arguments: Arguments,
    ) -> TaprResult<Value> {
        let keyword = arguments.unwrap_keyword(0);

        let string = match keyword.as_str() {
            "album" => self.0.album().unwrap_or(""),
            "album_artist" | "albumartist" => {
                self.0.album_artist().unwrap_or("")
            }
            "album_sort" | "albumsort" => self.0.albumsort().unwrap_or(""),
            "artist" => self.0.artist().unwrap_or(""),
            "genre" => self.0.genre().unwrap_or(""),
            "title" => self.0.title().unwrap_or(""),
            "year" => self.0.year().unwrap_or(""),
            "date" => self.0.date().unwrap_or(""),
            "track_number" | "tracknumber" => {
                self.0.track_number().unwrap_or("")
            }
            "disc_number" | "discnumber" | "disk_number" | "disknumber" => {
                self.0.disc_number().unwrap_or("")
            }
            _ => "",
        };

        Ok(if string.is_empty() { Value::Nil } else { string.trim().into() })
    }

    fn arity(&self) -> usize {
        1
    }

    fn callable_type(&self) -> CallableType {
        CallableType::Native
    }

    fn parameters(&self) -> Parameters {
        "k:keyword".try_into().unwrap()
    }
}

fn align(_: &mut Interpreter, arguments: Arguments) -> TaprResult<Value> {
    let f_width = arguments.unwrap_number(0);
    let string = arguments.unwrap_string(1);

    if f_width.fract() != 0.0 {
        return Err(TaprErrorKind::InvalidInteger(f_width).into());
    }

    let width: usize = f_width
        .round()
        .approx()
        .map_err(|_| TaprErrorKind::InvalidInteger(f_width))?;

    Ok(format!("{string:0>width$}").into())
}

fn parse_date(_: &mut Interpreter, arguments: Arguments) -> TaprResult<Value> {
    let date_string = arguments.unwrap_string(0);

    Ok(date_string.split('-').next().unwrap().into())
}
