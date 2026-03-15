//! `ServerPlugin` — assembles all server-side systems into the Bevy app.
//!
//! Step 5: Camera-relative movement with jump, sprint, crouch.

use bevy::prelude::*;
use game_networking::movement::update_crouch_collider;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::ClientOf;

use crate::movement::handle_character_actions;
use crate::spawning::{handle_connected, handle_new_client};

#[derive(Clone)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_character_actions, update_crouch_collider).chain(),
        );

        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
        app.add_systems(Update, shutdown_when_empty);
    }
}

/// Shut the server down once a client has connected and then all disconnect.
fn shutdown_when_empty(
    clients: Query<Entity, (With<ClientOf>, With<Connected>)>,
    mut has_had_clients: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    let count = clients.iter().count();
    if count > 0 {
        *has_had_clients = true;
    } else if *has_had_clients {
        info!("[server] no clients remaining, shutting down");
        exit.write(AppExit::Success);
    }
}
