use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::{error::AppError, AppState};

// El extractor que valida el JWT y devuelve el ID del usuario.
// Se puede usar en cualquier handler que requiera autenticación.
#[derive(Debug)]
pub struct AuthenticatedUser {
    pub user_id: i32,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extraer el token del header "Authorization"
        let headers = parts.headers.clone();
        let auth_header = headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Authentication("Missing Authorization header".to_string()))?;

        // 2. Verificar que el header tiene el formato "Bearer <token>"
        let bearer_token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Authentication("Invalid token format".to_string()))?;

        // 3. Decodificar y validar el token usando el servicio JWT
        let token_data = state.jwt_service.validate_token(bearer_token)?;

        // 4. Devolver el usuario autenticado
        Ok(AuthenticatedUser { user_id: token_data.claims.sub.parse().unwrap() })
    }
}
