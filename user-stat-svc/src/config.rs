use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub db_url: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

// init config
impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("user_stat_svc.yml"),
            File::open("/etc/user_stat_svc.yml"),
            env::var("USER_STAT_SVC_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(cfg_path)) => serde_yaml::from_reader(File::open(cfg_path)?),
            _ => bail!(""),
        };
        Ok(ret?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_work() {
        let conf = AppConfig::load().expect("load config fail");
        println!("config-port: {}", conf.server.port);
        assert_eq!(conf.server.port, 5001);

        println!("auth-info: {}", conf.auth.pk);
    }
}
