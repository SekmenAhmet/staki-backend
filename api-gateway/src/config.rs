use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub auth_service_url: String,
    pub social_service_url: String,
    pub messaging_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a number"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            social_service_url: env::var("SOCIAL_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            messaging_service_url: env::var("MESSAGING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8003".to_string()),
        }
    }
}
