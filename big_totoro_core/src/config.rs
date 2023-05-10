use crate::result::AppResult;

#[derive(Debug)]
pub struct Config {
    pub local_ip: String,
    pub local_ws_server_port: i64,
    pub token_secret: String,
    pub token_exp: i64,
    pub allow_platform: Vec<String>,
    pub etcd_uri: Vec<String>,
}

impl Config {
    pub fn new() -> AppResult<Self> {
        let env = std::env::var("ENV")?;
        dotenv::from_filename(format!("{}.env", env))?;

        Ok(Config {
            local_ip: local_ip_address::local_ip()?.to_string(),
            local_ws_server_port: dotenv::var("LOCAL_WS_SERVER_PORT")?.parse()?,
            token_secret: dotenv::var("TOKEN_SECRET")?.to_string(),
            token_exp: dotenv::var("TOKEN_EXP")?.parse()?,
            allow_platform: dotenv::var("ALLOW_PLATFORM")?
                .split(",")
                .map(str::to_string)
                .collect::<Vec<String>>(),
            etcd_uri: dotenv::var("ETCD_URI")?
                .split(",")
                .map(str::to_string)
                .collect::<Vec<String>>(),
        })
    }
}
