use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub port: u16,
    pub host: String,
    pub allow_past_due_dates: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set".to_string())?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET must be set".to_string())?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .map_err(|_| "JWT_EXPIRATION_HOURS must be a valid number".to_string())?,
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| "PORT must be a valid number".to_string())?,
            host: env::var("HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            allow_past_due_dates: env::var("ALLOW_PAST_DUE_DATES")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| "ALLOW_PAST_DUE_DATES must be true or false".to_string())?,
        })
    }
}
