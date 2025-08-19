use state::{Action, App, SignedOutState, State};
use std::sync::Arc;

pub mod asciiart;
pub mod chat_room_client;
pub mod events;
pub mod render;
pub mod state;
pub mod token;

pub mod state_machine;

#[tokio::main]
async fn main() {
    let client_builder = reqwest::ClientBuilder::new();
    let reqwest_client = client_builder
        .build()
        .expect("Unable to create http client");

    let (unauthorized_tx, mut unauthorized_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let client = chat_room_client::Client {
        inner: reqwest_client,
        unauthtorized_sender: unauthorized_tx,
    };
    let client = Arc::new(client);

    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<Action>();

    let mut terminal = ratatui::init();
    let mut app: App = start_app(Arc::clone(&client), &action_tx);

    while !app.should_close {
        render::draw(&app, &mut terminal).unwrap();

        if let Some(action) = events::handle_events(&app) {
            action_tx.send(action).unwrap();
        }

        while let Ok(action) = action_rx.try_recv() {
            state_machine::handle_action(action, &mut app, Arc::clone(&client), &action_tx);
        }

        while let Ok(_) = unauthorized_rx.try_recv() {
            state_machine::new_signout(&mut app);
        }
    }
}

fn start_app(
    client: Arc<chat_room_client::Client>,
    sideeffect: &tokio::sync::mpsc::UnboundedSender<Action>,
) -> App {
    let mut app = App {
        state: State::SignedOut(SignedOutState::new()),
        error: None,
        loading: false,
        should_close: false,
    };

    match token::read_token() {
        Some(token) => {
            state_machine::new_authenticate(&mut app, token, client, sideeffect);
            app
        }
        None => app,
    }
}
