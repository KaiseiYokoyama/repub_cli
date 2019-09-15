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

use crate::{prelude::*, load::*, data::*, compose::*};

fn main() -> RepubResult<()> {
    let app = crate::app::app();

    let input = Input::try_from(app.get_matches())?;
    println!("{:?}", &input);

    let data = InputData::from(input);
    println!("{:?}", &data);

    let mut composer = Composer::try_from(data)?;
    composer.compose()?;

    Ok(())
}
