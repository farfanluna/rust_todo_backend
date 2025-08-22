use axum::{  
    async_trait,  
    extract::FromRequestParts,  
    http::request::Parts,  
};  
use crate::{  
    auth::AuthenticatedUser,  
    error::{AppError, Result},  
    AppState,  
};  
  
#[derive(Debug, Clone, PartialEq)]  
pub enum UserRole {  
    User,  
    Admin,  
}  
  
impl UserRole {  
    pub fn from_string(role: &str) -> Self {  
        match role.to_lowercase().as_str() {  
            "admin" => UserRole::Admin,  
            _ => UserRole::User,  
        }  
    }  
      
    pub fn to_string(&self) -> &'static str {  
        match self {  
            UserRole::User => "user",  
            UserRole::Admin => "admin",  
        }  
    }  
}  
  
/// Representa un usuario autenticado con información de rol  
#[derive(Debug)]  
pub struct AuthenticatedUserWithRole {  
    pub user_id: i32,  
    pub role: UserRole,  
    pub email: String,  
    pub name: String,  
}  
  
impl AuthenticatedUserWithRole {  
    pub fn is_admin(&self) -> bool {  
        self.role == UserRole::Admin  
    }  
}  
  
#[async_trait]  
impl FromRequestParts<AppState> for AuthenticatedUserWithRole {  
    type Rejection = AppError;  
  
    async fn from_request_parts(  
        parts: &mut Parts,  
        state: &AppState,  
    ) -> Result<Self> {  
        println!("->> MIDDLEWARE | Extrayendo usuario autenticado con rol...");  
          
        // Primero obtener el usuario autenticado básico  
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;  
          
        // Luego obtener información completa del usuario incluyendo el rol  
        let user_data: UserWithRole = sqlx::query_as(  
            "SELECT id, name, email, role FROM users WHERE id = ?"  
        )  
        .bind(auth_user.user_id)  
        .fetch_optional(&state.db_pool)  
        .await?  
        .ok_or_else(|| {  
            AppError::Authentication("Usuario no encontrado en la base de datos".to_string())  
        })?;  
  
        let role = UserRole::from_string(&user_data.role);  
          
        println!("->> MIDDLEWARE | Usuario autenticado (ID: {}, Role: {})",   
                 auth_user.user_id, role.to_string());  
  
        Ok(AuthenticatedUserWithRole {  
            user_id: auth_user.user_id,  
            role,  
            email: user_data.email,  
            name: user_data.name,  
        })  
    }  
}  
  
#[allow(dead_code)] 
#[derive(Debug)]
pub struct AdminUser {
    pub email: String,
    pub name: String,
}
  
#[async_trait]  
impl FromRequestParts<AppState> for AdminUser {  
    type Rejection = AppError;  
  
    async fn from_request_parts(  
        parts: &mut Parts,  
        state: &AppState,  
    ) -> Result<Self> {  
        println!("->> MIDDLEWARE | Verificando acceso de administrador...");  
          
        let auth_user = AuthenticatedUserWithRole::from_request_parts(parts, state).await?;  
          
        if !auth_user.is_admin() {  
            println!("->> MIDDLEWARE | Acceso denegado: usuario no es administrador");  
            return Err(AppError::Authentication(  
                "Se requieren privilegios de administrador para acceder a este recurso".to_string()  
            ));  
        }  
          
        println!("->> MIDDLEWARE | Acceso de administrador concedido (ID: {})", auth_user.user_id);  
          
        Ok(AdminUser {  
            email: auth_user.email,  
            name: auth_user.name,  
        })  
    }  
}  
  
#[derive(sqlx::FromRow, Debug)]  
struct UserWithRole {  
    name: String,  
    email: String,  
    role: String,  
}  