mod prelude;
mod app;
mod load;
mod data;
mod tmpfile;
mod toc;
mod test;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use crate::{prelude::*, load::*, data::*};

fn main() -> RepubResult<()> {
    let app = crate::app::app();

    let input = Input::try_from(app.get_matches())?;
    println!("{:?}", &input);

    let data = Data::from(input);
    println!("{:?}", &data);

    Ok(())
}
