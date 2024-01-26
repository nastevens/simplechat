/// Widget for displaying received chat messages
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListDirection, Padding, Widget},
};
use simplechat_protocol::ReceivedMessage;

/// Display messages in a window that scrolls up as new messages are received
#[derive(Debug)]
pub struct ChatHistory<'a> {
    history: Vec<Text<'a>>,
    list: List<'a>,
}

impl<'a> Default for ChatHistory<'a> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            list: Self::list(),
        }
    }
}

impl Widget for ChatHistory<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf)
    }
}

impl Widget for &ChatHistory<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut items = self.history.clone();
        items.reverse();
        self.list.clone().items(items).render(area, buf);
    }
}

impl<'a> ChatHistory<'a> {
    /// Add a received message to history
    pub fn push_received(&mut self, msg: ReceivedMessage) {
        self.history.push(decorate_received(msg));
    }

    /// Add a self-sent message to history
    pub fn push_self(&mut self, msg: impl Into<String>) {
        self.history.push(decorate_self(msg.into()));
    }

    /// Delete all chat history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    fn list() -> List<'a> {
        List::default().direction(ListDirection::BottomToTop).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1)),
        )
    }
}

fn decorate_received<'a>(msg: ReceivedMessage) -> Text<'a> {
    Text::from(vec![
        Line::styled(
            format!("{}", msg.author),
            Style::default().fg(Color::Green),
        ),
        Span::raw(msg.text).into(),
        Line::default(),
    ])
}

fn decorate_self<'a>(text: String) -> Text<'a> {
    Text::from(vec![
        Line::styled("You", Style::default().fg(Color::Blue)),
        Line::raw(text),
        Line::default(),
    ])
}
