use crate::types::transaction::{RskTransaction, TransactionStatus};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

pub struct TransactionList<'a> {
    transactions: &'a [RskTransaction],
    selected: Option<usize>,
}

impl<'a> TransactionList<'a> {
    pub fn new(transactions: &'a [RskTransaction]) -> Self {
        Self {
            transactions,
            selected: None,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn selected_transaction(&self) -> Option<&RskTransaction> {
        self.selected
            .and_then(|i| self.transactions.get(i))
    }

    pub fn render(&self, area: ratatui::prelude::Rect, frame: &mut ratatui::prelude::Frame) {
        let header = Row::new(vec![
            Cell::from("#"),
            Cell::from("Hash"),
            Cell::from("From"),
            Cell::from("To"),
            Cell::from("Value (RBTC)"),
            Cell::from("Status"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

        let rows = self.transactions.iter().enumerate().map(|(i, tx)| {
            let status_style = match tx.status {
                TransactionStatus::Success => Style::default().fg(Color::Green),
                TransactionStatus::Failed => Style::default().fg(Color::Red),
                TransactionStatus::Pending => Style::default().fg(Color::Yellow),
                TransactionStatus::Unknown => Style::default().fg(Color::Gray),
            };

            let is_selected = self.selected == Some(i);
            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from((i + 1).to_string()),
                Cell::from(tx.hash.to_string()),
                Cell::from(tx.from.to_string()),
                Cell::from(tx.to.map(|a| a.to_string()).unwrap_or_else(|| "-".into())),
                Cell::from(ethers::utils::format_units(tx.value, 18).unwrap_or_else(|_| "N/A".into())),
                Cell::from(tx.status.to_string()).style(status_style),
            ])
            .style(style)
        });

        let table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Transactions"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
            .widths(&[
                ratatui::layout::Constraint::Length(4),   // #
                ratatui::layout::Constraint::Length(66), // Hash
                ratatui::layout::Constraint::Length(42), // From
                ratatui::layout::Constraint::Length(42), // To
                ratatui::layout::Constraint::Length(15), // Value
                ratatui::layout::Constraint::Length(10), // Status
            ]);

        frame.render_stateful_widget(
            table,
            area,
            &mut self.selected.unwrap_or(0).into(),
        );

        // Show transaction details if one is selected
        if let Some(tx) = self.selected_transaction() {
            self.render_transaction_details(tx, area, frame);
        }
    }

    fn render_transaction_details(
        &self,
        tx: &RskTransaction,
        area: ratatui::prelude::Rect,
        frame: &mut ratatui::prelude::Frame,
    ) {
        let details = vec![
            Line::from(vec![
                Span::styled("Hash: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(tx.hash.to_string()),
            ]),
            Line::from(vec![
                Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(tx.from.to_string()),
            ]),
            Line::from(vec![
                Span::styled("To: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(tx.to.map(|a| a.to_string()).unwrap_or_else(|| "-".into())),
            ]),
            Line::from(vec![
                Span::styled("Value: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(ethers::utils::format_units(tx.value, 18).unwrap_or_else(|_| "N/A".into())),
                Span::raw(" RBTC"),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    tx.status.to_string(),
                    match tx.status {
                        TransactionStatus::Success => Style::default().fg(Color::Green),
                        TransactionStatus::Failed => Style::default().fg(Color::Red),
                        TransactionStatus::Pending => Style::default().fg(Color::Yellow),
                        TransactionStatus::Unknown => Style::default().fg(Color::Gray),
                    },
                ),
            ]),
        ];

        let details_block = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Transaction Details"))
            .wrap(Wrap { trim: true });

        // Position the details to the right of the transactions
        let details_area = ratatui::layout::Rect {
            x: area.x + area.width / 2,
            y: area.y,
            width: area.width / 2,
            height: area.height,
        };

        frame.render_widget(details_block, details_area);
    }
}
