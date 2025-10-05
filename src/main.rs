mod config_change;
mod connection;
mod minecraft;
mod proxy;
mod proxy_config;

use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::proxy_config::{GameMap, Games};

#[tokio::main]
async fn main() {
    // GameMap::test();
    // return;
    colog::init();

    Controller::init().await;
}

pub struct Controller {
    rx: Receiver<ControllerEvents>,
    config: GameMap,
    join_handles: HashMap<Games, JoinHandle<()>>,
}

impl Controller {
    pub async fn init() {
        info!("Starting proxy server.");
        info!("Loading config.");

        let config = GameMap::from_config().unwrap_or_else(|e| {
            error!("Failed to load config!\n{:?}", e);
            panic!();
        });

        let (tx, rx) = mpsc::channel::<ControllerEvents>();

        // let tx = Arc::new(Mutex::new(tx));
        let config_location = config.config_location.clone();

        tokio::spawn(async move { config_change::monitor_config(config_location, tx).await });

        let mut controller = Controller {
            rx: rx,
            config: config,
            join_handles: HashMap::new(),
        };

        controller.start_all_listeners().await;

        controller.main_loop().await;
    }

    // On config change already established connections should not be interrupted
    async fn change_config(mut self) -> Controller {
        let config = GameMap::from_config().unwrap_or_else(|e| {
            error!("Failed to load config!\n{:?}", e);
            panic!();
        });

        self.config = config;
        self.restart_listeners().await
    }

    async fn restart_listeners(mut self) -> Controller {
        self = self.stop_all_listeners().await;
        self.start_all_listeners().await;
        self
    }

    async fn start_all_listeners(&mut self) {
        for game in self.config.get_games() {
            self.start_listener(game).await;
        }
    }

    async fn start_listener(&mut self, game: Games) {
        match TcpListener::bind(("0.0.0.0", game.clone() as u16)).await {
            Ok(listener) => {
                match self.config.get_mapping(&game) {
                    Ok(mapping) => {
                        info!("Bound to {:?}", game.clone() as usize);
                        let game_clone = game.clone();
                        let join_handle = tokio::spawn(async move {
                            proxy::listener_loop(listener, mapping, game).await;
                        });
                        self.join_handles.insert(game_clone, join_handle);
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

    async fn stop_all_listeners(mut self) -> Controller {
        for (_, handle) in self.join_handles {
            Controller::stop_listener(handle).await;
        }
        self.join_handles = HashMap::new();
        self
    }

    async fn stop_listener(handle: JoinHandle<()>) {
        if !handle.is_finished() {
            handle.abort();
            let _ = handle.await;
        }
    }

    async fn main_loop(mut self) {
        loop {
            match self.rx.recv() {
                Ok(ControllerEvents::ConfigUpdate) => {
                    info!("Config change detected, updating listeners...");
                    self = self.change_config().await;
                }
                Ok(ControllerEvents::Exit) => {
                    info!("Proxy shutting down, exit condition triggered");
                    self.stop_all_listeners().await;
                    return;
                }
                Err(e) => {
                    error!("recv error: {:?}", e);
                    return;
                }
            }
        }
    }
}

pub enum ControllerEvents {
    ConfigUpdate,
    Exit,
}
