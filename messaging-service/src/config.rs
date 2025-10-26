use std::env;

#[derive(Clone)]
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
                .expect("PORT must be a number"),
            mongo_uri: env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            redis_uri: env::var("REDIS_URL").expect("REDIS_URL must be set"),
        }
    }
}
