use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Error;
use strum::FromRepr;

#[derive(PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Clone, Debug, FromRepr)]
#[serde(rename_all = "lowercase")]
pub enum Games {
    Minecraft = 25565,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct GameMap {
    mapping: HashMap<Games, ServerMap>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ServerMap {
    mapping: HashMap<String, (String, usize)>,
}

impl GameMap {
    pub fn test() {
        let mut sMap = HashMap::<String, (String, usize)>::new();
        let mut gMap = HashMap::<Games, ServerMap>::new();
        sMap.insert("localhost".to_string(), ("127.0.0.1".to_string(), 25565));
        let servermap = ServerMap { mapping: sMap };
        gMap.insert(Games::Minecraft, servermap);
        let gamemap = GameMap { mapping: gMap };
        println!("{:?}", serde_json::to_string(&gamemap).unwrap());
    }
    pub fn from_config() -> Result<GameMap, Error> {
        let config_location =
            env::var("CONFIG_LOCATION").unwrap_or("/etc/gprox/config.json".to_string());

        let config_string = fs::read_to_string(config_location)?;

        let config = serde_json::from_str(&config_string).unwrap();

        Ok(GameMap { mapping: config })
    }

    pub fn get_games(&self) -> Vec<&Games> {
        Vec::from_iter(self.mapping.keys())
    }

    pub fn get_mapping(&self, game: &Games) -> Result<ServerMap, MappingError> {
        match self.mapping.get(game) {
            Some(map) => Ok(map.clone()),
            None => Err(MappingError::GameError(format!(
                "Mapping does not exist for {:?}.",
                game
            ))),
        }
    }
}

impl ServerMap {
    pub fn lookup(&self, target_host: &String) -> Result<&(String, usize), MappingError> {
        match self.mapping.get(target_host) {
            Some(map) => Ok(map),
            None => Err(MappingError::TargetError(
                "Target does not exist in mapping".to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub enum MappingError {
    TargetError(String),
    GameError(String),
}
