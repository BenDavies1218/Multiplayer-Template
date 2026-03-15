//! Client transport and connection setup

use core::net::{Ipv4Addr, SocketAddr};

use bevy::prelude::*;

use crate::client_config::GameClientConfig;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use game_core::performance_config::GamePerformanceConfig;
use game_core::world_config::GameWorldConfig;
use game_networking::config::Config;
use game_networking::config::SharedSettings;
use lightyear::interpolation::timeline::InterpolationConfig;
use lightyear::netcode::client_plugin::NetcodeConfig;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use lightyear::{netcode::NetcodeClient, websocket::client::WebSocketTarget};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientTransports {
    #[cfg(not(target_family = "wasm"))]
    Udp,
    WebTransport,
    WebSocket,
}

/// Event that examples can trigger to spawn a client.
#[derive(Component, Clone, Debug)]
#[component(on_add = ExampleClient::on_add)]
pub struct ExampleClient {
    pub client_id: u64,
    /// The client port to listen on
    pub client_port: u16,
    /// The socket address of the server
    pub server_addr: SocketAddr,
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ClientTransports,
    pub shared: SharedSettings,
}

impl ExampleClient {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let client_config = world
                .get_resource::<GameClientConfig>()
                .cloned()
                .unwrap_or_default();
            let performance_config = world
                .get_resource::<GamePerformanceConfig>()
                .cloned()
                .unwrap_or_default();
            let world_config = world
                .get_resource::<GameWorldConfig>()
                .cloned()
                .unwrap_or_default();
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ExampleClient>().unwrap();
            let config = Config::from_configs(
                &performance_config,
                &world_config,
                &settings.server_addr.ip().to_string(),
                settings.server_addr.port(),
            );
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);
            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                PredictionManager::default(),
                // Add interpolation delay to smooth out visual stuttering of remote entities
                // Configurable via INTERPOLATION_BUFFER_MS environment variable
                InterpolationConfig::default()
                    .with_send_interval_ratio(client_config.rendering.interpolation_send_ratio)
                    .with_min_delay(config.interpolation_buffer()),
                Name::from("Client"),
            ));

            let add_netcode = |entity_mut: &mut EntityWorldMut, config: &Config| -> Result {
                // use dummy zeroed key explicitly here.
                let auth = Authentication::Manual {
                    server_addr: settings.server_addr,
                    client_id: settings.client_id,
                    private_key: settings.shared.private_key,
                    protocol_id: settings.shared.protocol_id,
                };
                let netcode_config = NetcodeConfig {
                    // Make sure that the server times out clients when their connection is closed
                    // Configurable via CLIENT_TIMEOUT_SECS environment variable
                    client_timeout_secs: config.client_timeout_secs,
                    token_expire_secs: client_config.transport.token_expiration,
                    ..default()
                };
                entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);
                Ok(())
            };

            match settings.transport {
                #[cfg(not(target_family = "wasm"))]
                ClientTransports::Udp => {
                    add_netcode(&mut entity_mut, &config)?;
                    entity_mut.insert(UdpIo::default());
                }
                ClientTransports::WebTransport => {
                    add_netcode(&mut entity_mut, &config)?;
                    let certificate_digest = {
                        #[cfg(target_family = "wasm")]
                        {
                            // Lightyear expects hex digest without colons
                            include_str!("../../../certificates/digest.txt")
                                .trim()
                                .replace(':', "")
                        }
                        #[cfg(not(target_family = "wasm"))]
                        {
                            "".to_string()
                        }
                    };
                    entity_mut.insert(WebTransportClientIo { certificate_digest });
                }
                ClientTransports::WebSocket => {
                    add_netcode(&mut entity_mut, &config)?;
                    let client_config = {
                        #[cfg(target_family = "wasm")]
                        {
                            ClientConfig::default()
                        }
                        #[cfg(not(target_family = "wasm"))]
                        {
                            ClientConfig::builder().with_no_cert_validation()
                        }
                    };
                    entity_mut.insert(WebSocketClientIo {
                        config: client_config,
                        target: WebSocketTarget::Addr(Default::default()),
                    });
                }
            };
            Ok(())
        });
    }
}

pub fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger(Connect {
        entity: client.into_inner(),
    });
}
