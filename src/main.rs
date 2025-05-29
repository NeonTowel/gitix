#![allow(warnings)]
mod app;
mod config;
mod files;
mod git;
mod tui;

fn main() {
    let mut state = app::AppState::default();
    tui::start_tui(&mut state);
}
