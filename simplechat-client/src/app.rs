/// Main simple chat client app
use crate::{
    components::{
        chat_history::ChatHistory,
        text_input::{TextInput, TextInputAction},
    },
    tui::{Event, Tui},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use futures::{SinkExt, StreamExt};
use ratatui::prelude::{Constraint, Direction, Layout};
use simplechat_protocol::{
    ClientFrame, ClientFrameCodec, SentMessage, ServerFrame, ServerFrameCodec,
};
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
};
use tokio_util::codec::{FramedRead, FramedWrite};

/// Actions taken in response to events
#[derive(Debug)]
pub(crate) enum Action {
    Input(TextInputAction),
    Send,
    Quit,
}

/// Control logic for the application - receives events, translates them into
/// actions, adjusts state, and then renders that state
#[derive(Debug)]
pub(crate) struct App<'a> {
    history: ChatHistory<'a>,
    input: TextInput,
    quit: bool,
    reader: FramedRead<ReadHalf<TcpStream>, ServerFrameCodec>,
    writer: FramedWrite<WriteHalf<TcpStream>, ClientFrameCodec>,
    user: String,
}

impl<'a> App<'a> {
    pub async fn connect(addr: impl ToSocketAddrs, user: impl Into<String>) -> Result<Self> {
        let (rx, tx) = tokio::io::split(TcpStream::connect(addr).await?);
        let reader = FramedRead::new(rx, ServerFrameCodec::default());
        let writer = FramedWrite::new(tx, ClientFrameCodec::default());
        Ok(Self {
            history: ChatHistory::default(),
            input: TextInput::default(),
            quit: false,
            reader,
            writer,
            user: user.into(),
        })
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Input(action) => self.do_input(action).await,
            Action::Send => self.do_send().await,
            Action::Quit => self.do_quit().await,
        }
    }

    async fn do_input(&mut self, action: TextInputAction) -> Result<Option<Action>> {
        self.input.action(action);
        Ok(None)
    }

    async fn do_quit(&mut self) -> Result<Option<Action>> {
        self.quit = true;
        Ok(None)
    }

    async fn do_send(&mut self) -> Result<Option<Action>> {
        let input_text = self.input.get_input();
        let message = SentMessage::new(&self.user, input_text.clone());
        let frame = ClientFrame::send(message);
        self.writer.send(frame).await?;
        self.history.push_self(input_text);
        Ok(Some(Action::Input(TextInputAction::Clear)))
    }
}

fn map_event_to_action(_app: &App, event: Event) -> Option<Action> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Some(Action::Quit),
            KeyCode::Enter => Some(Action::Send),
            KeyCode::Backspace => Some(Action::Input(TextInputAction::Backspace)),
            KeyCode::Delete => Some(Action::Input(TextInputAction::Delete)),
            KeyCode::Left => Some(Action::Input(TextInputAction::MoveLeft)),
            KeyCode::Right => Some(Action::Input(TextInputAction::MoveRight)),
            KeyCode::Char(c) => Some(Action::Input(TextInputAction::Char(c))),
            _ => None,
        },
        _ => None,
    }
}

pub async fn run(addr: String, user: String) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::connect(addr, user).await?;

    loop {
        let mut action = None;

        tokio::select! {
            // render received message to UI
            maybe_frame = app.reader.next() => {
                if let Some(Ok(frame)) = maybe_frame {
                    match frame {
                        ServerFrame::Receive(msg) => {
                            app.history.push_received(msg);
                        }
                    }
                }

            }

            // turn UI events into actions
            maybe_event = tui.next() => {
                if let Some(event) = maybe_event {
                    action = map_event_to_action(&app, event);
                }
            }
        }

        // application update
        while let Some(next_action) = action {
            action = app.update(next_action).await?;
        }

        // application render
        tui.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)]);
            let split = layout.split(f.size());

            let (x, y) = app.input.cursor_position(split[1]);
            f.set_cursor(x, y);

            f.render_widget(&app.history, split[0]);
            f.render_widget(&app.input, split[1]);
        })?;

        // application exit
        if app.quit {
            break;
        }
    }

    Ok(())
}
