mod app;
mod model;
mod storage;
mod ui;

use app::App;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::{error::Error, io, time::Duration};

const DATA_FILE: &str = "./data/todos.csv";
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(DATA_FILE)?;

    ratatui::run(|terminal| run_app(terminal, &mut app))?;

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_exit() {
        terminal.draw(|frame| {
            ui::draw(frame, app);
        })?;

        if !event::poll(EVENT_POLL_INTERVAL)? {
            continue;
        }

        let event = event::read()?;

        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key);
            }
        }
    }

    Ok(())
}
