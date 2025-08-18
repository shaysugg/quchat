use anyhow::bail;
use chat_room_client::Client;
use futures::channel::mpsc::channel;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::error,
};

use crate::{
    chat_room_client::{self, Message, RoomState},
    state::{
        Action, App, AuthenticatedAction, AuthenticatedState, CreateRoomState, CurrentRoomState,
        SignedOutAction, SignedOutState, State, Textfield,
    },
    token,
};

pub fn handle_action(
    action: Action,
    app: &mut App,
    client: Arc<Client>,
    sideeffect: &UnboundedSender<Action>,
) {
    if app.loading {
        return;
    }
    if app.error.is_some() {
        if matches!(action, Action::CloseError) {
            app.error = None;
        }
        return;
    }
    match action {
        Action::Close => {
            app.should_close = true;
        }
        Action::Loading(is_loading) => {
            app.loading = is_loading;
        }
        Action::CloseError => {
            app.error = None;
        }
        Action::SigneOut(signed_out_action) => match signed_out_action {
            SignedOutAction::Signin => {
                if let State::SignedOut(ref state) = app.state {
                    let username = state.username_field.text.clone();
                    let password = state.password_field.text.clone();
                    let tx = sideeffect.clone();

                    tokio::spawn(async move {
                        let res = chat_room_client::sigin(client.deref(), &username, &password)
                            .await
                            .map_err(|e| anyhow::Error::from(e))
                            .and_then(|res| match token::persist_token(&res.token) {
                                Ok(_) => Ok(res.token),
                                Err(err) => bail!(err.to_string()),
                            });
                        tx.send(Action::SigneOut(SignedOutAction::SignedIn(res)))
                            .unwrap();
                    });
                }
            }
            SignedOutAction::Register => {
                if let State::SignedOut(ref state) = app.state {
                    let username = state.username_field.text.clone();
                    let password = state.password_field.text.clone();
                    let tx = sideeffect.clone();

                    tokio::spawn(async move {
                        let res = chat_room_client::register(client.deref(), &username, &password)
                            .await
                            .map_err(|e| anyhow::Error::from(e))
                            .and_then(|res| match token::persist_token(&res.token) {
                                Ok(_) => Ok(res.token),
                                Err(err) => bail!(err.to_string()),
                            });
                        tx.send(Action::SigneOut(SignedOutAction::Registered(res)))
                            .unwrap();
                    });
                }
            }
            SignedOutAction::Text(text_action) => {
                if let State::SignedOut(ref mut state) = app.state {
                    if state.username_field.focused {
                        state.username_field.handle_action(&text_action);
                    } else if state.password_field.focused {
                        state.password_field.handle_action(&text_action);
                    }
                }
            }
            SignedOutAction::NextFocus => {
                if let State::SignedOut(ref mut state) = app.state {
                    if state.username_field.focused {
                        state.username_field.focused = false;
                        state.password_field.focused = true;
                    } else {
                        state.username_field.focused = true;
                        state.password_field.focused = false;
                    }
                }
            }
            SignedOutAction::UnFocus => {
                if let State::SignedOut(ref mut state) = app.state {
                    state.username_field.focused = false;
                    state.password_field.focused = false;
                }
            }
            SignedOutAction::SignedIn(res) => {
                if let State::SignedOut(_) = app.state {
                    app.loading = false;
                    match res {
                        Ok(token) => {
                            new_authenticate(app, token, client, sideeffect);
                        }

                        Err(err) => app.error = Some(err.to_string()),
                    }
                }
            }
            SignedOutAction::Registered(res) => {
                if let State::SignedOut(_) = app.state {
                    app.loading = false;
                    match res {
                        Ok(token) => {
                            new_authenticate(app, token, client, sideeffect);
                        }

                        Err(err) => app.error = Some(err.to_string()),
                    }
                }
            }
        },
        Action::Authenticated(action) => {
            if let State::Authenticated(ref mut state) = app.state {
                match action {
                    AuthenticatedAction::SelectNextRoom => match state.selected_room_index {
                        Some(index) => {
                            if index < state.rooms.len() {
                                state.selected_room_index = Some(index + 1);
                            }
                        }
                        None => state.selected_room_index = Some(0),
                    },
                    AuthenticatedAction::SelectPrevRoom => match state.selected_room_index {
                        Some(index) => {
                            if index > 0 {
                                state.selected_room_index = Some(index - 1);
                            }
                        }
                        None => state.selected_room_index = Some(0),
                    },
                    AuthenticatedAction::ScrollMessagesDown => {
                        if let Some(ref mut room) = state.current_room {
                            if room.selected_message < room.messages.len() {
                                room.selected_message = room.selected_message + 1;
                            }
                        }
                    }
                    AuthenticatedAction::ScrollMessagesUp => {
                        if let Some(ref mut room) = state.current_room {
                            if room.selected_message > 0 {
                                room.selected_message = room.selected_message - 1;
                            }
                        }
                    }
                    AuthenticatedAction::EnterRoom => {
                        if let Some(index) = state.selected_room_index {
                            if let Some(ref prev_room) = state.current_room {
                                prev_room.abort_join_handles();
                            }
                            let mut message_field = Textfield::new("message");
                            message_field.focused = true;
                            message_field.hint = "<Enter> Send";
                            state.current_room_index = Some(index);
                            state.current_room = Some(CurrentRoomState {
                                messages: Vec::new(),
                                join_handles: Vec::new(),
                                message_field,
                                name: state.rooms[index].name.to_string(),
                                id: state.rooms[index].id.to_string(),
                                selected_message: 0,
                            });
                            sideeffect
                                .send(Action::Authenticated(AuthenticatedAction::LoadPrevMessages))
                                .unwrap();
                            sideeffect
                                .send(Action::Authenticated(AuthenticatedAction::SyncMessages))
                                .unwrap();
                            sideeffect
                                .send(Action::Authenticated(AuthenticatedAction::MakeRoomAsSeen))
                                .unwrap();
                        }
                    }
                    AuthenticatedAction::ExitRoom => {
                        state.current_room_index = None;
                        state.current_room = None;
                    }
                    AuthenticatedAction::ChatText(text_field_action) => {
                        if let Some(ref mut room) = state.current_room {
                            room.message_field.handle_action(&text_field_action);
                        }
                    }
                    AuthenticatedAction::SendMessage => {
                        if let Some(ref mut room) = state.current_room {
                            let message = room.message_field.text.clone();
                            let token = state.token.clone();
                            let room_id = room.id.clone();
                            if message.is_empty() {
                                return;
                            }
                            room.message_field.text.clear();
                            tokio::spawn(async move {
                                let _ = chat_room_client::send_message(
                                    client.deref(),
                                    &token,
                                    &message,
                                    &room_id,
                                )
                                .await;
                            });
                        }
                    }
                    AuthenticatedAction::LoadRooms => {
                        let token = state.token.clone();
                        let tx = sideeffect.clone();
                        tokio::spawn(async move {
                            let res = chat_room_client::rooms(client.deref(), &token)
                                .await
                                .map(|r| r.rooms);
                            tx.send(Action::Authenticated(AuthenticatedAction::RoomsLoaded(res)))
                        });
                    }
                    AuthenticatedAction::RoomsLoaded(res) => match res {
                        Ok(rooms) => {
                            state.rooms = rooms;
                            sideeffect
                                .send(Action::Authenticated(AuthenticatedAction::UpdateRoomStates))
                                .unwrap();

                            sideeffect
                                .send(Action::Authenticated(
                                    AuthenticatedAction::ListenForRoomStateChanges,
                                ))
                                .unwrap();
                        }
                        Err(error) => app.error = Some(error.to_string()),
                    },

                    AuthenticatedAction::SyncMessages => {
                        if let Some(ref mut room) = state.current_room {
                            let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(10);
                            let token = state.token.clone();
                            let sideeffect: UnboundedSender<Action> = sideeffect.clone();
                            let room_id = room.id.clone();
                            let join_handle1 = tokio::spawn(async move {
                                chat_room_client::messages(
                                    client.deref(),
                                    &token,
                                    &room_id,
                                    tx.clone(),
                                )
                                .await;
                            });
                            let join_handle2 = tokio::spawn(async move {
                                loop {
                                    if let Some(msg) = rx.recv().await {
                                        sideeffect
                                            .send(Action::Authenticated(
                                                AuthenticatedAction::MessageChanged(msg),
                                            ))
                                            .unwrap();
                                    }
                                }
                            });

                            room.join_handles = vec![join_handle1, join_handle2];
                        }
                    }
                    AuthenticatedAction::LoadPrevMessages => {
                        if let Some(ref mut room) = state.current_room {
                            let token = state.token.clone();
                            let sideeffect = sideeffect.clone();
                            let room_id = room.id.clone();
                            tokio::spawn(async move {
                                let result = chat_room_client::last_messages(
                                    client.deref(),
                                    &room_id,
                                    &token,
                                )
                                .await;
                                sideeffect.send(Action::Authenticated(
                                    AuthenticatedAction::PrevMessagesLoaded(result),
                                ))
                            });
                        }
                    }
                    AuthenticatedAction::PrevMessagesLoaded(result) => {
                        if let Some(ref mut room) = state.current_room {
                            match result {
                                Ok(mut prev_messages) => room.messages.append(&mut prev_messages),
                                Err(err) => app.error = Some(err.to_string()),
                            }
                        }
                    }
                    AuthenticatedAction::MessageChanged(msg) => {
                        if let Some(ref mut room) = state.current_room {
                            room.messages.push(msg);
                        }
                    }
                    AuthenticatedAction::LoadUserProfile => {
                        let token = state.token.clone();
                        let sideeffect = sideeffect.clone();
                        tokio::spawn(async move {
                            let res = chat_room_client::whoami(client.deref(), &token).await;
                            sideeffect
                                .send(Action::Authenticated(
                                    AuthenticatedAction::UserprofileLoaded(res),
                                ))
                                .unwrap();
                        });
                    }
                    AuthenticatedAction::UserprofileLoaded(res) => match res {
                        Ok(profile) => state.profile = Some(profile),
                        Err(err) => app.error = Some(err.to_string()),
                    },
                    AuthenticatedAction::Signout => {
                        let token = state.token.clone();
                        let sideeffect = sideeffect.clone();
                        tokio::spawn(async move {
                            let res = chat_room_client::signout(&client, &token).await;
                            sideeffect
                                .send(Action::Authenticated(
                                    AuthenticatedAction::SignoutCompleted(res),
                                ))
                                .unwrap();
                        });
                    }
                    AuthenticatedAction::SignoutCompleted(result) => match result {
                        Ok(_) => new_signout(app),
                        Err(error) => app.error = Some(error.to_string()),
                    },
                    AuthenticatedAction::StartCreatingRoom => {
                        state.create_room = Some(CreateRoomState::new())
                    }
                    AuthenticatedAction::CreateNewRoom => {
                        if let Some(ref mut create_room) = state.create_room {
                            let token = state.token.clone();
                            let sideeffect = sideeffect.clone();
                            let params = chat_room_client::CreateRoomParam {
                                name: create_room.name_field.text.clone(),
                            };
                            tokio::spawn(async move {
                                let res =
                                    chat_room_client::create_room(&client, &params, &token).await;
                                sideeffect
                                    .send(Action::Authenticated(
                                        AuthenticatedAction::NewRoomIsCreated(res),
                                    ))
                                    .unwrap();
                            });
                        }
                    }
                    AuthenticatedAction::NewRoomIsCreated(res) => match res {
                        Ok(room) => {
                            state.rooms.insert(0, room);
                            state.create_room = None;
                        }
                        Err(err) => app.error = Some(err.to_string()),
                    },
                    AuthenticatedAction::CreateRoomName(action) => {
                        if let Some(ref mut create_room) = state.create_room {
                            create_room.name_field.handle_action(&action);
                        }
                    }
                    AuthenticatedAction::CancelNewRoom => {
                        if state.create_room.is_some() {
                            state.create_room = None
                        }
                    }
                    AuthenticatedAction::ListenForRoomStateChanges => {
                        let token = state.token.clone();
                        let ids = state
                            .rooms
                            .iter()
                            .map(|r| r.id.clone())
                            .collect::<Vec<String>>();
                        let sideeffect = sideeffect.clone();
                        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);
                        tokio::spawn(async move {
                            chat_room_client::room_states_events(
                                &client,
                                &token,
                                ids.iter().map(|i| i.as_str()).collect::<Vec<&str>>(),
                                tx.clone(),
                            )
                            .await;
                        });
                        tokio::spawn(async move {
                            while let Some(_) = rx.recv().await {
                                sideeffect
                                    .send(Action::Authenticated(
                                        AuthenticatedAction::UpdateRoomStates,
                                    ))
                                    .unwrap();
                            }
                        });
                    }

                    AuthenticatedAction::UpdateRoomStates => {
                        let token = state.token.clone();
                        let ids = state
                            .rooms
                            .iter()
                            .map(|r| r.id.clone())
                            .collect::<Vec<String>>();
                        let sideeffect = sideeffect.clone();
                        tokio::spawn(async move {
                            // let ids = ids.into_iter().map(|i| i.as_str().clone()).collect::<Vec<&str>>();
                            match chat_room_client::room_states(
                                &client,
                                &token,
                                ids.iter().map(|i| i.as_str()).collect(),
                            )
                            .await
                            {
                                Ok(room_states) => {
                                    let a = room_states
                                        .into_iter()
                                        .map(|i| (i.room_id.clone(), i))
                                        .collect::<HashMap<String, RoomState>>();

                                    sideeffect
                                        .send(Action::Authenticated(
                                            AuthenticatedAction::RoomStatesUpdated(Ok(a)),
                                        ))
                                        .unwrap();
                                }
                                Err(error) => {
                                    sideeffect
                                        .send(Action::Authenticated(
                                            AuthenticatedAction::RoomStatesUpdated(Err(error)),
                                        ))
                                        .unwrap();
                                }
                            };
                        });
                    }

                    AuthenticatedAction::RoomStatesUpdated(res) => match res {
                        Ok(states) => {
                            println!("{}", states.len());
                            state.rooms_states = states
                        }
                        Err(error) => app.error = Some(error.to_string()),
                    },

                    AuthenticatedAction::MakeRoomAsSeen => {
                        if let Some(ref current) = state.current_room {
                            let token = state.token.clone();
                            let room_id = current.id.clone();
                            if let Some(rooms_states) = state.rooms_states.get_mut(&current.id) {
                                rooms_states.has_unread = false;
                            }

                            tokio::spawn(async move {
                                let _ =
                                    chat_room_client::update_room_seen(&client, &token, &room_id)
                                        .await;
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn new_authenticate(
    app: &mut App,
    token: String,
    client: Arc<chat_room_client::Client>,
    sideeffect: &tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let state = State::Authenticated(AuthenticatedState::new(token));

    app.state = state;
    app.error = None;
    app.loading = false;

    handle_action(
        Action::Authenticated(AuthenticatedAction::LoadRooms),
        app,
        Arc::clone(&client),
        sideeffect,
    );

    handle_action(
        Action::Authenticated(AuthenticatedAction::LoadUserProfile),
        app,
        Arc::clone(&client),
        sideeffect,
    );
}

pub fn new_signout(app: &mut App) {
    if token::read_token().is_some() {
        let _ = token::delete_token();
    }
    let state = State::SignedOut(SignedOutState::new());

    app.state = state;
    app.error = None;
    app.loading = false;
}
