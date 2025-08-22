use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use std::net::SocketAddr;
use crate::{error::{AppError, Result}, AppState};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: i32,
    pub window_duration_minutes: i32,
    pub block_duration_minutes: i32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_duration_minutes: 15,
            block_duration_minutes: 60,
        }
    }
}


#[derive(sqlx::FromRow, Debug)]
struct RateLimit {
    blocked_until: Option<String>,
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response> { // <-- Corrected
    let ip = get_real_ip(&addr, &headers);
    let endpoint = request.uri().path().to_string();

    // Configuración específica según el endpoint
    let config = match endpoint.as_str() {
        path if path.starts_with("/auth/login") => RateLimitConfig {
            requests_per_window: 10,
            window_duration_minutes: 15,
            block_duration_minutes: 30,
        },
        path if path.starts_with("/auth/register") => RateLimitConfig {
            requests_per_window: 5,
            window_duration_minutes: 60,
            block_duration_minutes: 60,
        },
        _ => RateLimitConfig::default(),
    };

    // Verificar si la IP está bloqueada
    if let Some(rate_limit) = get_rate_limit(&state, &ip, &endpoint).await? {
        if let Some(blocked_until) = rate_limit.blocked_until {
            let blocked_time = chrono::DateTime::parse_from_rfc3339(&blocked_until)
                .map_err(|_| AppError::InternalServerError("Error parsing blocked time".to_string()))?;

            if Utc::now() < blocked_time {
                println!("->> SECURITY | IP {} bloqueada hasta {}", ip, blocked_until);
                return Err(AppError::Authentication(
                    format!("IP bloqueada por exceso de requests. Intenta después de {}", blocked_until)
                ));
            }
        }
    }

    // Procesar la request
    let response = next.run(request).await;

    // Actualizar contador de rate limiting
    update_rate_limit(&state, &ip, &endpoint, &config).await?;

    Ok(response)
}

async fn get_rate_limit(
    state: &AppState,
    ip: &str,
    endpoint: &str,
) -> Result<Option<RateLimit>> { // <-- Corrected
    let rate_limit = sqlx::query_as::<_, RateLimit>(
        "SELECT blocked_until
         FROM rate_limits
         WHERE ip_address = ? AND endpoint = ?"
    )
    .bind(ip)
    .bind(endpoint)
    .fetch_optional(&state.db_pool)
    .await?;

    Ok(rate_limit)
}

async fn update_rate_limit(
    state: &AppState,
    ip: &str,
    endpoint: &str,
    config: &RateLimitConfig,
) -> Result<()> { // <-- Corrected
    let now = Utc::now();
    let window_start = now - Duration::minutes(config.window_duration_minutes as i64);

    // Intentar actualizar un registro existente
    let result = sqlx::query(
        "UPDATE rate_limits
         SET request_count = request_count + 1, updated_at = ?
         WHERE ip_address = ? AND endpoint = ?
         AND datetime(window_start) > datetime(?)"
    )
    .bind(now.to_rfc3339())
    .bind(ip)
    .bind(endpoint)
    .bind(window_start.to_rfc3339())
    .execute(&state.db_pool)
    .await?;

    if result.rows_affected() == 0 {
        // Crear nuevo registro o resetear ventana
        sqlx::query(
            "INSERT OR REPLACE INTO rate_limits
             (ip_address, endpoint, request_count, window_start, updated_at)
             VALUES (?, ?, 1, ?, ?)"
        )
        .bind(ip)
        .bind(endpoint)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&state.db_pool)
        .await?;
    } else {
        // Verificar si se excedió el límite
        let current_count: (i32,) = sqlx::query_as(
            "SELECT request_count FROM rate_limits
             WHERE ip_address = ? AND endpoint = ?"
        )
        .bind(ip)
        .bind(endpoint)
        .fetch_one(&state.db_pool)
        .await?;

        if current_count.0 > config.requests_per_window {
            let blocked_until = now + Duration::minutes(config.block_duration_minutes as i64);

            sqlx::query(
                "UPDATE rate_limits
                 SET blocked_until = ?, updated_at = ?
                 WHERE ip_address = ? AND endpoint = ?"
            )
            .bind(blocked_until.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(ip)
            .bind(endpoint)
            .execute(&state.db_pool)
            .await?;

            println!("->> SECURITY | IP {} bloqueada por exceder límite de rate", ip);
        }
    }

    Ok(())
}

pub async fn record_login_attempt(
    state: &AppState,
    ip: &str,
    email: Option<&str>,
    success: bool,
    user_agent: Option<&str>,
) -> Result<()> { // <-- Corrected
    sqlx::query(
        "INSERT INTO login_attempts (ip_address, email, success, user_agent)
         VALUES (?, ?, ?, ?)"
    )
    .bind(ip)
    .bind(email)
    .bind(success)
    .bind(user_agent)
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

pub fn get_real_ip(addr: &SocketAddr, headers: &HeaderMap) -> String {
    // Prioridad para detectar la IP real
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    addr.ip().to_string()
}