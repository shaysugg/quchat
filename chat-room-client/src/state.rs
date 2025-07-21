use std::thread::JoinHandle;

use crate::{
    chat_room_client::{self, Message, Room, UserProfile},
    render,
};

pub struct App {
    pub state: State<'static>,
    pub error: Option<String>,
    pub loading: bool,
    pub should_close: bool,
}

pub enum State<'r> {
    Authenticated(AuthenticatedState<'r>),
    SignedOut(SignedOutState<'r>),
}

pub struct AuthenticatedState<'r> {
    pub token: String,
    pub rooms: Vec<Room>,
    pub current_room_index: Option<usize>,
    pub selected_room_index: Option<usize>,
    pub current_room: Option<CurrentRoomState<'r>>,
    pub profile: Option<UserProfile>,
    //settings
}

impl<'r> AuthenticatedState<'r> {
    pub fn new(token: String) -> Self {
        AuthenticatedState {
            token: token,
            rooms: Vec::new(),
            current_room: None,
            selected_room_index: None,
            current_room_index: None,
            profile: None,
        }
    }
}

pub struct CurrentRoomState<'r> {
    pub messages: Vec<Message>,
    pub message_field: Textfield<'r>,
    pub name: String,
    pub id: String,
    pub selected_message: usize,
    pub join_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl<'r> CurrentRoomState<'r> {
    pub fn abort_join_handles(&self) {
        for join_handle in &self.join_handles {
            join_handle.abort();
        }
    }
}

impl<'r> Drop for CurrentRoomState<'r> {
    fn drop(&mut self) {
        self.abort_join_handles();
    }
}

pub struct SignedOutState<'r> {
    pub username_field: Textfield<'r>,
    pub password_field: Textfield<'r>,
}

#[derive(Debug)]
pub enum Action {
    SigneOut(SignedOutAction),
    Authenticated(AuthenticatedAction),
    Loading(bool),
    Close,
    CloseError,
}
#[derive(Debug)]
pub enum SignedOutAction {
    Signin,
    Register,
    Text(TextFieldAction),
    NextFocus,
    UnFocus,
    SignedIn(anyhow::Result<String>),
    Registered(anyhow::Result<String>),
}

#[derive(Debug)]
pub enum AuthenticatedAction {
    SelectNextRoom,
    SelectPrevRoom,
    EnterRoom,
    ExitRoom,
    SendMessage,
    SelectNextMessage,
    SelectPrevMessage,
    ChatText(TextFieldAction),
    LoadUserProfile,
    UserprofileLoaded(chat_room_client::Result<UserProfile>),
    LoadRooms,
    RoomsLoaded(chat_room_client::Result<Vec<Room>>),
    PrevMessagesLoaded(chat_room_client::Result<Vec<Message>>),
    SyncMessages,
    LoadPrevMessages,
    MessageChanged(Message),
    Signout,
    SignoutCompleted(chat_room_client::Result<()>),
}

impl<'r> SignedOutState<'r> {
    pub fn new() -> Self {
        SignedOutState {
            username_field: Textfield::new("username"),
            password_field: Textfield::new("password"),
        }
    }
}

pub struct Textfield<'r> {
    pub label: &'r str,
    pub text: String,
    pub focused: bool,
}

impl<'r> Textfield<'r> {
    pub fn new(label: &'r str) -> Self {
        Textfield {
            label: label,
            text: String::new(),
            focused: false,
        }
    }

    pub fn handle_action(&mut self, action: &TextFieldAction) {
        match action {
            TextFieldAction::TextChanged(ch) => self.text.push(*ch),
            TextFieldAction::TextErased => {
                if self.text.is_empty() {
                    return;
                }
                let last_i = self.text.len() - 1;
                let _ = self.text.remove(last_i);
            }
            TextFieldAction::TextAllErased => self.text.clear(),
        };
    }
}
#[derive(Debug, Clone)]
pub enum TextFieldAction {
    TextChanged(char),
    TextErased,
    TextAllErased,
}

//mock
impl<'r> AuthenticatedState<'r> {
    pub fn mock() -> Self {
        let names = ["hello"; 50];
        let rooms = names
            .iter()
            .map(|name| Room {
                id: name.to_string(),
                name: name.to_string(),
                create_date: 0,
            })
            .collect::<Vec<Room>>();

        let message = ["msg1"; 100];
        let msgs = message
            .iter()
            .map(|name| Message {
                id: name.to_string(),
                content: name.to_string(),
                sender_id: name.to_string(),
                room_id: name.to_string(),
                create_date: 0,
            })
            .collect::<Vec<Message>>();

        let room_name = rooms[0].name.clone();
        let room_id = rooms[0].id.clone();
        AuthenticatedState {
            token: String::new(),
            rooms: rooms,
            current_room_index: Some(0),
            current_room: Some(CurrentRoomState {
                messages: msgs,
                message_field: Textfield::new("message"),
                name: room_name,
                id: room_id,
                selected_message: 0,
                join_handles: Vec::new(),
            }),
            selected_room_index: Some(0),
            profile: None,
        }
    }
}
