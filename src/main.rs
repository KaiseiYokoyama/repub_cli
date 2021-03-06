mod prelude;
mod app;
mod load;
mod data;
mod tmpfile;
mod toc;
mod compose;
mod test;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate log;

use crate::{prelude::*, load::*, data::*, compose::*};

fn main() {
    if let Err(e) = run() {
        if cfg!(debug_assertions) {
            RepubError(format!("{:?}", &e)).print();
        } else {
            RepubError(format!("{}", &e)).print();
        }
    }
}

fn run() -> RepubResult<()> {
    let app = crate::app::app();

    let input = Input::try_from(app.get_matches())?;

    let data = InputData::from(input);

    let mut composer = Composer::try_from(data)?;
    composer.compose()?;

    Ok(())
}