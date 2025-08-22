use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

// Importamos nuestro gestor de errores personalizado
use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user_id)
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
}


// Esta es la única definición del struct, y es clonable
// para poder ser parte del AppState.
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

impl JwtService {
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            expiration_hours,
        }
    }

    /// Genera un nuevo token JWT para un ID de usuario.
    pub fn generate_token(&self, user_id: i32) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        
        // El '?' al final convierte automáticamente el error de `encode` en nuestro AppError::Jwt.
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;

        Ok(token)
    }

    /// Valida un token y devuelve sus datos si es correcto.
    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>> {
        // El '?' convierte el error si la decodificación falla.
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data)
    }

    /// Extrae el ID de usuario de un token válido.
    pub fn extract_user_id(&self, token: &str) -> Result<i32> {
        let token_data = self.validate_token(token)?;
        token_data
            .claims
            .sub
            .parse::<i32>()
            .map_err(|_| AppError::Authentication("ID de usuario inválido en el token".to_string()))
    }
}