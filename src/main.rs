use app::App;
use std::{io, sync::{Arc, Mutex}};
use net::Results;
mod app;
mod ui;
mod scan;
mod net;

#[derive(Default)]
struct Progress {
    total: usize,
    finished: usize,
}

fn main() -> io::Result<()> {
    
    let shared_state = Arc::new(Mutex::new(Results::new()));

    let mut terminal = ratatui::init();
    let app_result = App::init(shared_state.clone()).run(&mut terminal);
    ratatui::restore();
    app_result
}
