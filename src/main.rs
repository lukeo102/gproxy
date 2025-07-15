mod connection;
mod minecraft;
mod proxy;
mod proxy_config;

use log::{error, info, warn};
use std::error::Error;
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::connection::ConnectionDetails;
use crate::minecraft::minecraft::Minecraft;
use crate::proxy_config::{GameMap, Games, MappingError, ServerMap};

#[tokio::main]
async fn main() {
    // GameMap::test();
    // return;
    colog::init();

    info!("Starting proxy server.");
    info!("Getting config.");
    let config = GameMap::from_config().unwrap_or_else(|e| {
        error!("Failed to load config!\n{:?}", e);
        panic!();
    });

    let games = config.get_games();
    let mut bind_join_handles = Vec::new();

    info!("Binding to ports!");
    for game in games {
        let game = game.clone();

        match TcpListener::bind(("0.0.0.0", game.clone() as u16)).await {
            Ok(listener) => {
                match config.get_mapping(&game) {
                    Ok(mapping) => {
                        info!("Bound to {:?}", game.clone() as usize);
                        let join_handle = tokio::spawn(async move {
                            proxy::listener_loop(listener, mapping, game).await;
                        });
                        bind_join_handles.push(join_handle);
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                    }
                };
            }

            Err(e) => {
                warn!("Failed to bind to port.\n{:?}", e);
            }
        }
    }

    for join_handle in bind_join_handles {
        let _ = join_handle.await;
    }
}
