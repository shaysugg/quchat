use std::collections::HashMap;

use qu_chat_models::{Message, Room, RoomState, UserProfile};

use crate::chat_room_client;

pub struct App {
    pub state: State<'static>,
    pub error: Option<String>,
    pub loading: bool,
    pub should_close: bool,
}

impl App {
    pub fn initial() -> Self {
        App {
            state: State::SignedOut(SignedOutState::new()),
            error: None,
            loading: false,
            should_close: false,
        }
    }
}

pub enum State<'r> {
    Authenticated(AuthenticatedState<'r>),
    SignedOut(SignedOutState<'r>),
}

pub struct AuthenticatedState<'r> {
    pub token: String,
    pub rooms: Vec<Room>,
    pub rooms_states: std::collections::HashMap<String, RoomState>,
    pub current_room_index: Option<usize>,
    pub selected_room_index: Option<usize>,
    pub current_room: Option<CurrentRoomState<'r>>,
    pub create_room: Option<CreateRoomState<'r>>,
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
            create_room: None,
            rooms_states: HashMap::new(),
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

pub struct CreateRoomState<'r> {
    pub name_field: Textfield<'r>,
}

impl CreateRoomState<'_> {
    pub fn new() -> Self {
        CreateRoomState {
            name_field: Textfield::new("name"),
        }
    }
}

pub struct SignedOutState<'r> {
    pub server_field: Textfield<'r>,
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

    ChatText(TextFieldAction),
    CreateRoomName(TextFieldAction),
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
    ScrollMessagesUp,
    ScrollMessagesDown,
    StartCreatingRoom,
    CreateNewRoom,
    CancelNewRoom,
    NewRoomIsCreated(chat_room_client::Result<Room>),
    ListenForRoomStateChanges,
    UpdateRoomStates,
    RoomStatesUpdated(chat_room_client::Result<HashMap<String, RoomState>>),
    MakeRoomAsSeen,
}

impl<'r> SignedOutState<'r> {
    pub fn new() -> Self {
        SignedOutState {
            server_field: Textfield::new_focused("server", true),
            username_field: Textfield::new("username"),
            password_field: Textfield::new("password"),
        }
    }
}

pub struct Textfield<'r> {
    pub label: &'r str,
    pub text: String,
    pub focused: bool,
    pub hint: &'r str,
}

impl<'r> Textfield<'r> {
    pub fn new(label: &'r str) -> Self {
        Textfield {
            label: label,
            text: String::new(),
            focused: false,
            hint: "",
        }
    }

    pub fn new_focused(label: &'r str, focuse: bool) -> Self {
        Textfield {
            label: label,
            text: String::new(),
            focused: focuse,
            hint: "",
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
                creator_id: "".to_string(),
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
                sender_name: name.to_string(),
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
            create_room: None,
            rooms_states: HashMap::new(),
        }
    }
}
