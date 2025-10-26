use std::env;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub mongo_uri: String,
    pub redis_uri: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8082".to_string())
                .parse()
                .expect("Port must be a number"),
            mongo_uri: env::var("MONGO_URI").expect("MONGO_URI must be set"),
            redis_uri: env::var("REDIS_URI").expect("REDIS_URI must be set"),
        }
    }
}
