mod prelude;
mod app;
mod load;
mod test;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;

use crate::prelude::*;

fn main() -> RepubResult<()> {
    use crate::load::load::Input;

    let app = crate::app::app();
    let input = Input::try_from(app.get_matches())?;

    println!("{:?}", input);
    Ok(())
}
