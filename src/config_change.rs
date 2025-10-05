use log::{error, info, warn};
use notify::event::{EventKind, ModifyKind};
use notify::{Event, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::mpsc::{Sender, channel};

use crate::ControllerEvents;

pub async fn monitor_config(config: String, main_loop_tx: Sender<ControllerEvents>) {
    let (tx, rx) = channel::<Result<Event>>();
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(watcher) => watcher,
        Err(e) => {
            warn!(
                "Unable to setup config watcher, config changes will not be applied until restart.\nError: {}",
                e
            );
            return;
        }
    };

    match watcher.watch(Path::new(&config), RecursiveMode::NonRecursive) {
        Ok(_) => {}
        Err(e) => {
            warn!(
                "Unable to setup config watcher, config changes will not be applied until restart.\nError: {}",
                e
            );
            return;
        }
    }

    info!("Watching config for changes.");

    loop {
        for res in &rx {
            match res {
                Ok(event) => {
                    if event.paths.len() == 1 && event.paths[0].ends_with("config.json") {
                        match event.kind {
                            EventKind::Modify(modify_kind) => match modify_kind {
                                ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Other => {
                                    info!("Config changed, reloading proxy");

                                    let _ = main_loop_tx.send(ControllerEvents::ConfigUpdate);
                                }
                                _ => {}
                            },
                            EventKind::Remove(_) => {
                                warn!(
                                    "Config file removed, config will remain unchanged but the proxy will not be able to restart!"
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("Error with event: {:?}", e);
                }
            }
        }
    }
}
