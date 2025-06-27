use crate::types::transaction::RskTransaction;
use crate::tui::transaction_list::TransactionList;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub struct App {
    transactions: Vec<RskTransaction>,
    should_quit: bool,
    transaction_list: TransactionList<'static>,
}

impl App {
    pub fn new(transactions: Vec<RskTransaction>) -> Self {
        let transaction_list = TransactionList::new(&transactions);
        Self {
            transactions,
            should_quit: false,
            transaction_list,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        while !self.should_quit {
            self.draw(&mut terminal)?;
            self.handle_events()?;
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();
            self.transaction_list.render(size, f);
        })?;
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Down => {
                        if let Some(selected) = self.transaction_list.selected {
                            let new_selected = (selected + 1).min(self.transactions.len().saturating_sub(1));
                            self.transaction_list.select(Some(new_selected));
                        } else if !self.transactions.is_empty() {
                            self.transaction_list.select(Some(0));
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = self.transaction_list.selected {
                            let new_selected = selected.saturating_sub(1);
                            self.transaction_list.select(Some(new_selected));
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(tx) = self.transaction_list.selected_transaction() {
                            // Copy transaction hash to clipboard
                            if let Err(e) = clipboard_win::set_clipboard_string(&tx.hash.to_string()) {
                                eprintln!("Failed to copy to clipboard: {}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
