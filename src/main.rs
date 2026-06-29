use std::error::Error;

mod app;
mod bots;
mod event;
mod map;
mod state;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = app::App::new();

    app.run()?;

    Ok(())
}
