use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    Database(String),
    
    #[error("Error de autenticación: {0}")]
    Authentication(String),
    
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),
    
    #[error("Conflicto: {0}")]
    Conflict(String),
    
    #[error("Error de validación")]
    Validation {
        message: String,
        fields: HashMap<String, String>,
    },
    
    #[error("Solicitud inválida: {0}")]
    BadRequest(String),
    
    #[error("Error interno del servidor: {0}")]
    InternalServerError(String),
    
    #[error("Error de JWT: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Error de bcrypt: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    
    #[error("Error de SQLx: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Error de migración: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "code": "VALIDATION_ERROR",
    "message": "La entrada proporcionada no es válida",
    "fields": {
        "title": "Title must be between 3 and 120 characters"
    }
}))]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, String>>,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "error": {
        "code": "NOT_FOUND",
        "message": "Recurso no encontrado"
    }
}))]
pub struct ErrorPayload {
    pub error: ApiError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_payload) = match self {
            Self::Database(msg) => {
                eprintln!("❌ Error de base de datos: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPayload {
                        error: ApiError {
                            code: "DATABASE_ERROR".to_string(),
                            message: "Error de base de datos".to_string(),
                            fields: None,
                        },
                    },
                )
            }
            Self::Authentication(msg) => {
                eprintln!("🔐 Error de autenticación: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorPayload {
                        error: ApiError {
                            code: "AUTHENTICATION_ERROR".to_string(),
                            message: msg,
                            fields: None,
                        },
                    },
                )
            }
            Self::NotFound(msg) => {
                eprintln!("🔍 Recurso no encontrado: {}", msg);
                (
                    StatusCode::NOT_FOUND,
                    ErrorPayload {
                        error: ApiError {
                            code: "NOT_FOUND".to_string(),
                            message: msg,
                            fields: None,
                        },
                    },
                )
            }
            Self::Conflict(msg) => {
                eprintln!("⚡ Conflicto: {}", msg);
                (
                    StatusCode::CONFLICT,
                    ErrorPayload {
                        error: ApiError {
                            code: "CONFLICT".to_string(),
                            message: msg,
                            fields: None,
                        },
                    },
                )
            }
            Self::Validation { message, fields } => {
                eprintln!("✏️ Error de validación: {}", message);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorPayload {
                        error: ApiError {
                            code: "VALIDATION_ERROR".to_string(),
                            message,
                            fields: Some(fields),
                        },
                    },
                )
            }
            Self::BadRequest(msg) => {
                eprintln!("📝 Solicitud inválida: {}", msg);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorPayload {
                        error: ApiError {
                            code: "BAD_REQUEST".to_string(),
                            message: msg,
                            fields: None,
                        },
                    },
                )
            }
            Self::InternalServerError(msg) => {
                eprintln!("💥 Error interno: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPayload {
                        error: ApiError {
                            code: "INTERNAL_ERROR".to_string(),
                            message: "Ha ocurrido un error inesperado".to_string(),
                            fields: None,
                        },
                    },
                )
            }
            Self::Jwt(err) => {
                eprintln!("🎫 Error de JWT: {}", err);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorPayload {
                        error: ApiError {
                            code: "JWT_ERROR".to_string(),
                            message: "Token inválido o expirado".to_string(),
                            fields: None,
                        },
                    },
                )
            }
            Self::Bcrypt(err) => {
                eprintln!("🔐 Error de Bcrypt: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPayload {
                        error: ApiError {
                            code: "BCRYPT_ERROR".to_string(),
                            message: "Error de encriptación".to_string(),
                            fields: None,
                        },
                    },
                )
            }
            Self::Sqlx(err) => {
                eprintln!("🗄️ Error de SQLx: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPayload {
                        error: ApiError {
                            code: "DATABASE_ERROR".to_string(),
                            message: "Error de base de datos".to_string(),
                            fields: None,
                        },
                    },
                )
            }
            Self::Migration(err) => {
                eprintln!("🚀 Error de migración: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPayload {
                        error: ApiError {
                            code: "MIGRATION_ERROR".to_string(),
                            message: "Error en migración de base de datos".to_string(),
                            fields: None,
                        },
                    },
                )
            }
        };

        (status_code, Json(error_payload)).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let mut fields = HashMap::new();
        for (field, errors) in err.field_errors() {
            let messages: Vec<String> = errors
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Error de validación".to_string())
                })
                .collect();
            fields.insert(field.to_string(), messages.join(", "));
        }

        AppError::Validation {
            message: "La entrada proporcionada no es válida".to_string(),
            fields,
        }
    }
}
