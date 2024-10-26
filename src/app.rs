use crate::audio_visualiser;

use audio_visualiser::{get_visualiser, get_wave};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    DefaultTerminal, Frame,
};

pub struct App {
    should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_exit: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                self.should_exit = true;
            }
        }
        Ok(())
    }


    fn draw(&self, frame: &mut Frame) {
        let [title, visualiser] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(frame.area());

        let wave_data = get_wave();
        frame.render_widget("BEEP BOOP".bold().into_centered_line(), title);
        frame.render_widget(get_visualiser(&wave_data), visualiser);
    }
}
