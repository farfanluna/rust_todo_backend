use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use crate::config::Config;
use crate::error::Result;

pub async fn init_db(config: &Config) -> Result<SqlitePool> {
    // Conecta a SQLite (crea el archivo si no existe)
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    println!("->> DB | Conexión establecida a: {}", config.database_url);

    Ok(pool)
}
