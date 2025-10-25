use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub mongo_uri: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8001".to_string())
                .parse()
                .expect("PORT must be a number"),
            mongo_uri: env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}
