use crate::asciiart;
use crate::state::{
    Action, App, AuthenticatedState, SignedOutAction, SignedOutState, State, TextFieldAction,
    Textfield,
};
use ratatui::layout::{Constraint, Flex, Layout, Margin, Offset};
use ratatui::prelude::{Buffer, Rect};

use ratatui::style::{Color, Modifier, Style, Stylize};

use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Padding,
    Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget,
};
use ratatui::DefaultTerminal;

pub fn draw(app: &App, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    terminal.draw(|frame| frame.render_widget(app, frame.area()))?;
    Ok(())
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered();

        let instructions = self.state.instructions();
        let spans = instructions
            .iter()
            .flat_map(|i| i.spans())
            .collect::<Vec<Span<'static>>>();

        let inner = block.inner(area);

        block
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title_bottom(spans)
            .title_alignment(ratatui::layout::Alignment::Right)
            .render(area, buf);

        self.state.render(inner, buf);

        if let Some(ref err) = self.error {
            let content = Text::raw(err.to_string());
            let block = Block::bordered().title("Error");
            let area = center(
                inner,
                Constraint::Percentage(50),
                Constraint::Length(4), // top and bottom border + content
            );
            Clear.render(area, buf);
            Paragraph::new(content).block(block).render(area, buf);
        }

        if self.loading {
            let block = Block::default();
            let area = center(
                inner,
                Constraint::Percentage(50),
                Constraint::Length(4), // top and bottom border + content
            );
            Clear.render(area, buf);
            Paragraph::new("loading").block(block).render(area, buf);
        }
    }
}

impl<'r> Widget for &State<'r> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            State::SignedOut(s) => s.render(area, buf),
            State::Authenticated(s) => s.render(area, buf),
        };
    }
}

impl<'r> Widget for &SignedOutState<'r> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);

        let [welcome_area, username_are, password_area] = layout.areas(area);

        self.username_field.render(username_are, buf);
        self.password_field.render(password_area, buf);

        let content = Text::raw(asciiart::WELCOME);
        Paragraph::new(content).centered().render(welcome_area, buf);
    }
}

struct Instructions<'r> {
    name: &'r str,
    key: &'r str,
}

impl<'r> Instructions<'r> {
    pub fn spans(&self) -> [Span<'r>; 4] {
        [
            Span::raw(" "),
            Span::raw(self.formatted_key()).bold(),
            Span::raw(self.name),
            Span::raw(" "),
        ]
    }

    fn formatted_key(&self) -> String {
        format!("<{}>", self.key)
    }
}

trait Instructable<'r> {
    fn instructions(&self) -> Vec<Instructions<'static>>;
}

impl<'r> Instructable<'r> for State<'r> {
    fn instructions(&self) -> Vec<Instructions<'static>> {
        match self {
            State::SignedOut(s) => vec![
                Instructions {
                    name: "Register",
                    key: "^r",
                },
                Instructions {
                    name: "Signin",
                    key: "^s",
                },
                Instructions {
                    name: "Change text field",
                    key: "TAB",
                },
            ],
            State::Authenticated(s) => {
                if s.current_room.is_some() {
                    vec![
                        Instructions {
                            name: "Back",
                            key: "ESC",
                        },
                        Instructions {
                            name: "Send Message",
                            key: "Enter",
                        },
                        Instructions {
                            name: "Up/Down",
                            key: "↑↓",
                        },
                        Instructions {
                            name: "SignOut",
                            key: "^s",
                        },
                    ]
                } else {
                    vec![
                        Instructions {
                            name: "Up/Down",
                            key: "↑↓",
                        },
                        Instructions {
                            name: "Enter",
                            key: "select room",
                        },
                        Instructions {
                            name: "SignOut",
                            key: "^s",
                        },
                    ]
                }
            }
            _ => Vec::new(),
        }
    }
}

impl<'r> Widget for &Textfield<'r> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = if self.focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new()
        };
        let block = Block::bordered()
            .title(self.label)
            .border_style(style)
            .border_type(BorderType::Rounded);

        Paragraph::new(Text::from(self.text.as_str()))
            .block(block)
            .render(area, buf);
    }
}

impl<'r> Widget for &AuthenticatedState<'r> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [menu, main] = layout.areas(area);

        self.render_menu(menu, buf);
        self.render_main(main, buf);
    }
}

impl<'r> AuthenticatedState<'r> {
    fn render_menu(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]);
        let [list_area, profile_area] = layout.areas(area);

        if let Some(ref profile) = self.profile {
            let block = Block::new().borders(Borders::TOP);

            let paragraph = Paragraph::new(Line::raw(&profile.name).centered());
            paragraph.block(block).render(profile_area, buf);
        }

        let block = Block::new()
            .title(Line::raw("Rooms").centered())
            .border_set(border::THICK);

        //room_list
        let items: Vec<ListItem> = self
            .rooms
            .iter()
            .enumerate()
            .map(|(i, room)| ListItem::from(room.name.clone()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_style(Style::new().bold())
            .highlight_spacing(HighlightSpacing::Always);

        let mut list_state = ListState::default();
        list_state.select(self.selected_room_index);

        // list.render(menu, buf);
        StatefulWidget::render(&list, list_area, buf, &mut list_state);
    }

    fn render_main(&self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .border_set(border::THICK)
            .borders(Borders::LEFT);

        let main_inner = main_block.inner(area);

        main_block.render(area, buf);

        //current room

        if let Some(ref room) = self.current_room {
            let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
            let [messages_are, textfield_area] = layout.areas(main_inner);

            let items: Vec<ListItem> = room
                .messages
                .iter()
                .enumerate()
                .map(|(_, message)| {
                    let text = format!("{}: {}", message.sender_id, message.content);
                    if let Some(ref profile) = self.profile {
                        if message.sender_id == profile.id {
                            let line = Line::from(text.clone()).right_aligned();
                            return ListItem::new(line);
                        }
                    }

                    let line = Line::from(text.clone());
                    return ListItem::new(line);
                })
                .collect();

            let mut scrollbar_state =
                ScrollbarState::new(items.len()).position(room.selected_message);
            let list = List::new(items);

            Widget::render(list, messages_are, buf);

            let scroll = Scrollbar::default();

            StatefulWidget::render(scroll, area, buf, &mut scrollbar_state);

            room.message_field.render(textfield_area, buf);
        } else {
            let text = Text::from("Select a room to start chat")
                .alignment(ratatui::layout::Alignment::Center);
            text.render(main_inner, buf);
        }
    }
}

// impl<'r> Textfield<'r> {

// }

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
