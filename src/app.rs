use crate::song_details::SongMetaData;
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::style::Stylize;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::{io, time::Duration};

pub struct App {
    metadata: SongMetaData,
    should_quit: bool,
}

impl App {
    pub fn new(metadata: SongMetaData) -> Self {
        Self {
            metadata,
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        self.should_quit = true;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(8), // Metadata
                Constraint::Min(5),    // Lyrics
            ])
            .margin(1)
            .split(f.size());

        // Title
        let title = Paragraph::new("MP3 Player")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Metadata
        let minutes = self.metadata.duration.as_secs() / 60;
        let seconds = self.metadata.duration.as_secs() % 60;

        let metadata_text = vec![
            Line::from(vec![
                "Title: ".fg(Color::Yellow),
                self.metadata
                    .title
                    .clone()
                    .unwrap_or_default()
                    .fg(Color::White),
            ]),
            Line::from(vec![
                "Artist: ".fg(Color::Yellow),
                self.metadata
                    .artist
                    .clone()
                    .unwrap_or_default()
                    .fg(Color::White),
            ]),
            Line::from(vec![
                "Album: ".fg(Color::Yellow),
                self.metadata
                    .album
                    .clone()
                    .unwrap_or_default()
                    .fg(Color::White),
            ]),
            Line::from(vec![
                "Duration: ".fg(Color::Yellow),
                format!("{:02}:{:02}", minutes, seconds).fg(Color::White),
            ]),
        ];

        let metadata = Paragraph::new(metadata_text)
            .block(Block::default().borders(Borders::ALL).title("Metadata"));
        f.render_widget(metadata, chunks[1]);

        // Lyrics
        let lyrics_text = self.metadata.lyrics.clone().unwrap_or_default();
        let lyrics = Paragraph::new(lyrics_text)
            .block(Block::default().borders(Borders::ALL).title("Lyrics"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(lyrics, chunks[2]);
    }
}
