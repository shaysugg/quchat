use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Modifier,
};

use crate::state::*;

pub fn handle_events(app: &App) -> Option<Action> {
    let res = event::poll(std::time::Duration::from_millis(100));
    //TODO fix condition
    if let Ok(has) = res {
        if has {
            handle_events_blocking(&app)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn handle_events_blocking(app: &App) -> Option<Action> {
    let event = event::read();
    let state_action = match &event {
        Ok(Event::Key(key))
            if KeyCode::Char('c') == key.code && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Some(Action::Close)
        }
        //shouldn't check the app state
        //app state should be checked in state machines
        Ok(Event::Key(key)) if KeyCode::Enter == key.code && app.error.is_some() => {
            Some(Action::CloseError)
        }
        Ok(event) => app.state.handle_events(event),
        _ => None,
    };
    if let Some(_) = state_action {
        return state_action;
    }

    let app_action = match event {
        Ok(Event::Key(key)) if KeyCode::Char('q') == key.code => Some(Action::Close),
        _ => None,
    };
    app_action
}

impl<'r> State<'r> {
    fn handle_events(&self, event: &Event) -> Option<Action> {
        match self {
            State::SignedOut(s) => s.handle_events(event),
            State::Authenticated(s) => s.handle_events(event),
        }
    }
}

impl<'r> SignedOutState<'r> {
    fn handle_events(&self, event: &Event) -> Option<Action> {
        let action: Option<SignedOutAction> = match event {
            Event::Key(key) => match key.code {
                KeyCode::Tab => Some(SignedOutAction::NextFocus),
                KeyCode::Esc => Some(SignedOutAction::UnFocus),
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(SignedOutAction::Register)
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(SignedOutAction::Signin)
                }
                _ => try_handle_text_events(event).map(|a| SignedOutAction::Text(a)),
            },
            _ => None,
        };
        action.map(|a| Action::SigneOut(a))
    }
}

impl<'r> AuthenticatedState<'r> {
    fn handle_events(&self, event: &Event) -> Option<Action> {
        let action: Option<AuthenticatedAction> = match event {
            Event::Key(key) => match key.code {
                KeyCode::Up if self.current_room.is_none() => {
                    Some(AuthenticatedAction::SelectPrevRoom)
                }
                KeyCode::Down if self.current_room.is_none() => {
                    Some(AuthenticatedAction::SelectNextRoom)
                }
                KeyCode::Up if self.current_room.is_some() => {
                    Some(AuthenticatedAction::ScrollMessagesUp)
                }
                KeyCode::Down if self.current_room.is_some() => {
                    Some(AuthenticatedAction::ScrollMessagesDown)
                }
                KeyCode::Enter if self.create_room.is_some() => {
                    Some(AuthenticatedAction::CreateNewRoom)
                }
                KeyCode::Enter if self.current_room.is_none() => self
                    .selected_room_index
                    .map(|s| AuthenticatedAction::EnterRoom),

                KeyCode::Enter if self.current_room.is_some() => {
                    Some(AuthenticatedAction::SendMessage)
                }
                KeyCode::Esc if self.create_room.is_some() => {
                    Some(AuthenticatedAction::CancelNewRoom)
                }
                KeyCode::Esc if self.current_room.is_some() => Some(AuthenticatedAction::ExitRoom),

                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(AuthenticatedAction::Signout)
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(AuthenticatedAction::StartCreatingRoom)
                }

                _ if self.create_room.is_some() => {
                    try_handle_text_events(event).map(|e| AuthenticatedAction::CreateRoomName(e))
                }

                _ if self.current_room.is_some() => {
                    try_handle_text_events(event).map(|e| AuthenticatedAction::ChatText(e))
                }
                _ => None,
            },
            _ => None,
        };
        action.map(|a| Action::Authenticated(a))
    }
}

fn try_handle_text_events(event: &Event) -> Option<TextFieldAction> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char(ch) => Some(TextFieldAction::TextChanged(ch)),
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    Some(TextFieldAction::TextAllErased)
                } else {
                    Some(TextFieldAction::TextErased)
                }
            }
            _ => None,
        },
        _ => None,
    }
}
