use crate::cli::Config;
use indicatif::{
    ProgressBar as IProgressBar, ProgressDrawTarget, ProgressFinish,
    ProgressStyle,
};

pub(crate) struct AudioFileSpinner {
    spinner: IProgressBar,
}

impl AudioFileSpinner {
    pub(crate) fn new() -> AudioFileSpinner {
        let spinner = IProgressBar::new(0);

        let style = ProgressStyle::default_spinner()
            .template(
                "[{pos}/{len} audio files/total files] {wide_msg} {spinner}",
            )
            .on_finish(ProgressFinish::AtCurrentPos);

        spinner.set_style(style);
        spinner.set_draw_target(ProgressDrawTarget::stdout());
        spinner.set_message("Gathering files...");

        AudioFileSpinner { spinner }
    }

    pub(crate) fn inc_found(&self) {
        self.spinner.inc(1);
    }

    pub(crate) fn inc_total(&self) {
        // std::thread::sleep(std::time::Duration::from_millis(100));
        self.spinner.inc_length(1);
        self.spinner.tick();
    }

    pub(crate) fn finish(&self) {
        self.spinner.finish_at_current_pos();
        self.spinner.set_message("Gathered files.");
    }
}

pub(crate) fn create_progressbar(
    len: u64,
    msg: &'static str,
    finished_msg: &'static str,
    preview: bool,
) -> IProgressBar {
    let bar = IProgressBar::new(len);

    let pp = if preview { Config::PREVIEW_PREFIX } else { "" };

    let template = format!("{pp}[{{pos}}/{{len}}] {{msg}} {{wide_bar}}");

    bar.set_style(ProgressStyle::default_bar().template(&template).on_finish(
        ProgressFinish::WithMessage(std::borrow::Cow::Borrowed(finished_msg)),
    ));
    bar.set_draw_target(ProgressDrawTarget::stdout());
    bar.set_message(msg);

    bar
}
