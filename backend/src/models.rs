use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
#[allow(unused_imports)] 
use serde_json::json; 

// --- Modelos de Base de Datos / Respuesta ---

/// Representa a un usuario en la base de datos y en las respuestas de la API.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, sqlx::FromRow)]
#[schema(example = json!({
    "id": 1,
    "name": "Jesús Farfán Luna",
    "email": "lic.farfanluna@hotmail.com",
    "role": "user",
    "created_at": "2025-08-20T10:00:00Z"
}))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    /// Rol del usuario: 'user' o 'admin'
    pub role: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

/// Representa los datos del usuario devueltos en el login.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct UserLoginResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

/// Representa una tarea perteneciente a un usuario.
#[derive(Serialize, Deserialize, Debug, ToSchema, sqlx::FromRow)]
#[schema(example = json!({
    "id": 101,
    "user_id": 1,
    "title": "Implementar documentación de la API",
    "description": "Integrar Utoipa y Swagger UI para documentar todos los endpoints.",
    "status": "doing",
    "priority": "high",
    "due_date": "2025-08-22T23:59:59Z",
    "created_at": "2025-08-20T12:00:00Z",
    "updated_at": "2025-08-20T14:30:00Z",
    "tags": "rust,api,documentacion",
    "owner_name": "Jesús Farfán Luna",
    "owner_email": "lic.farfanluna@hotmail.com"
}))]
pub struct Task {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Option<String>,
    pub assigned_to: Option<String>,
    // Campos adicionales para administradores
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
}

/// Parámetros de consulta para filtrar y paginar tareas con búsqueda avanzada.
/// Para administradores incluye filtros adicionales por usuario.
#[derive(Deserialize, Debug, ToSchema, IntoParams)]
#[schema(example = json!({
    "page": 1,
    "per_page": 10,
    "search": "configurar sistema documentacion",
    "sort_by": "due_date",
    "sort_order": "asc", 
    "status": "todo,doing",
    "priority": "high,med",
    "tags": "rust,api,urgent",
    "due_date_start": "2025-08-01T00:00:00Z",
    "due_date_end": "2025-12-31T23:59:59Z",
    "user_id": 1,
    "owner_name": "Jesús",
    "owner_email": "admin@admin.com"
}))]
pub struct TaskQueryParams {
    /// Número de página para la paginación (mínimo 1).
    #[schema(minimum = 1, example = 1)]
    pub page: Option<i64>,
    
    /// Número de tareas por página (entre 1 y 100).
    #[schema(minimum = 1, maximum = 100, example = 10)]
    pub per_page: Option<i64>,
    
    /// Términos de búsqueda separados por espacios.
    #[schema(example = "configurar sistema")]
    pub search: Option<String>,
    
    /// Campo por el cual ordenar.
    #[schema(example = "due_date")]
    pub sort_by: Option<String>,
    
    /// Orden de clasificación: ASC o DESC.
    #[schema(example = "asc")]
    pub sort_order: Option<String>,
    
    /// Filtrar por estados separados por comas.
    #[schema(example = "todo,doing")]
    pub status: Option<String>,
    
    /// Filtrar por prioridades separadas por comas.
    #[schema(example = "high,med")]
    pub priority: Option<String>,
    
    /// Filtrar por tags separados por comas.
    #[schema(example = "rust,api,urgent")]
    pub tags: Option<String>,
    
    /// Fecha de inicio para filtrar por fecha de entrega.
    #[schema(example = "2025-08-01T00:00:00Z")]
    pub due_date_start: Option<String>,
    
    /// Fecha de fin para filtrar por fecha de entrega.
    #[schema(example = "2025-12-31T23:59:59Z")]
    pub due_date_end: Option<String>,
    
    // --- FILTROS EXCLUSIVOS PARA ADMINISTRADORES ---
    
    /// Filtrar por ID de usuario específico (solo administradores).
    #[schema(example = 1)]
    pub user_id: Option<i32>,
    
    /// Filtrar por nombre de propietario (solo administradores).
    #[schema(example = "Jesús")]
    pub owner_name: Option<String>,
    
    /// Filtrar por email de propietario (solo administradores).
    #[schema(example = "admin@admin.com")]
    pub owner_email: Option<String>,
    
    /// Filtrar por persona asignada a la tarea.
    #[schema(example = "Jesús Farfán")]
    pub assigned_to: Option<String>,
}

// --- Nuevos modelos para administración ---

/// Respuesta para listar usuarios (solo administradores)
#[derive(Serialize, Debug, ToSchema)]
#[schema(example = json!({
    "users": [],
    "pagination": {
        "page": 1,
        "per_page": 10,
        "total": 42,
        "total_pages": 5
    }
}))]
pub struct UsersResponse {
    pub users: Vec<UserSummary>,
    pub pagination: PaginationInfo,
}

/// Resumen de usuario para listados
#[derive(Serialize, Debug, ToSchema, sqlx::FromRow)]
#[schema(example = json!({
    "id": 1,
    "name": "Jesús Farfán Luna",
    "email": "lic.farfanluna@hotmail.com",
    "role": "user",
    "task_count": 12,
    "created_at": "2025-08-20T10:00:00Z"
}))]
pub struct UserSummary {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
    pub task_count: i64,
    pub created_at: String,
}

/// Estadísticas del sistema (solo administradores)
#[derive(Serialize, Debug, ToSchema)]
#[schema(example = json!({
    "total_users": 25,
    "total_tasks": 150,
    "tasks_by_status": {
        "todo": 45,
        "doing": 30,
        "done": 75
    },
    "tasks_by_priority": {
        "low": 50,
        "med": 60,
        "high": 40
    },
    "recent_activity": {
        "new_users_today": 2,
        "tasks_created_today": 8,
        "tasks_completed_today": 5
    }
}))]
pub struct SystemStats {
    pub total_users: i64,
    pub total_tasks: i64,
    pub tasks_by_status: TaskStatusStats,
    pub tasks_by_priority: TaskPriorityStats,
    pub recent_activity: RecentActivity,
}

#[derive(Serialize, Debug, ToSchema, sqlx::FromRow)]
pub struct TaskStatusStats {
    pub todo: i64,
    pub doing: i64,
    pub done: i64,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct TaskPriorityStats {
    pub low: i64,
    pub med: i64,
    pub high: i64,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RecentActivity {
    pub new_users_today: i64,
    pub tasks_created_today: i64,
    pub tasks_completed_today: i64,
}

/// Request body for updating a user's role
#[derive(Deserialize, Debug, ToSchema, Validate)]
pub struct UpdateUserRoleRequest {
    #[validate(custom(function = "validate_role"))]
    pub role: String,
}

// --- Resto de modelos sin cambios ---

#[derive(Serialize, Deserialize, Debug, ToSchema, Validate)]
#[schema(example = json!({
    "email": "lic.farfanluna@hotmail.com",
    "password": "demo123"
}))]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Validate)]
#[schema(example = json!({
    "name": "Jesús Farfán Luna",
    "email": "lic.farfanluna@admin.com",
    "password": "demo123"
}))]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, max = 100, message = "Password must be between 6 and 100 characters"))]
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema, Validate)]
#[schema(example = json!({
    "title": "Implementar documentación de la API",
    "description": "Integrar Utoipa y Swagger UI para documentar todos los endpoints.",
    "status": "todo",
    "priority": "high",
    "due_date": "2025-08-22T23:59:59Z",
    "tags": "rust,api,documentacion"
}))]
pub struct CreateTaskRequest {
    #[validate(length(min = 3, max = 120, message = "Title must be between 3 and 120 characters"))]
    pub title: String,
    pub description: Option<String>,
    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,
    #[validate(custom(function = "validate_priority"))]
    pub priority: Option<String>,
    #[validate(custom(function = "validate_due_date"))]
    pub due_date: Option<String>,
    #[validate(length(max = 500, message = "Tags cannot exceed 500 characters"))]
    pub tags: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema, Validate)]
#[schema(example = json!({
    "title": "Implementar documentación de la API - Actualizada",
    "description": "Integrar Utoipa y Swagger UI completamente.",
    "status": "doing",
    "priority": "high",
    "due_date": "2025-08-25T23:59:59Z",
    "tags": "rust,api,documentacion,urgente"
}))]
pub struct UpdateTaskRequest {
    #[validate(length(min = 3, max = 120, message = "Title must be between 3 and 120 characters"))]
    pub title: Option<String>,
    pub description: Option<String>,
    #[validate(custom(function = "validate_status"))]
    pub status: Option<String>,
    #[validate(custom(function = "validate_priority"))]
    pub priority: Option<String>,
    #[validate(custom(function = "validate_due_date"))]
    pub due_date: Option<String>,
    #[validate(length(max = 500, message = "Tags cannot exceed 500 characters"))]
    pub tags: Option<String>,
    pub assigned_to: Option<String>,
}

/// Respuesta paginada para las tareas
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[schema(example = json!({
    "tasks": [],
    "pagination": {
        "page": 1,
        "per_page": 10,
        "total": 42,
        "total_pages": 5
    }
}))]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct PaginationInfo {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Respuesta para el login exitoso
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[schema(example = json!({
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
        "id": 1,
        "name": "Jesús Farfán Luna",
        "email": "lic.farfanluna@hotmail.com",
        "role": "user",
        "created_at": "2025-08-20T10:00:00Z"
    }
}))]
pub struct LoginResponse {
    pub token: String,
    pub user: UserLoginResponse,
}

// --- Validadores ---
fn validate_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "todo" | "doing" | "done" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_status")),
    }
}

fn validate_priority(priority: &str) -> Result<(), validator::ValidationError> {
    match priority {
        "low" | "med" | "high" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_priority")),
    }
}

fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    match role {
        "user" | "admin" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_role")),
    }
}

fn validate_due_date(date_str: &str) -> Result<(), validator::ValidationError> {
    use chrono::{DateTime, Utc};
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        // Usar `date_naive()` para comparar solo la fecha, ignorando la hora.
        // Esto es consistente con la lógica en el handler de `routes.rs`.
        if date.date_naive() < Utc::now().date_naive() {
            return Err(validator::ValidationError::new("due_date_in_past"));
        }
    } else {
        // Si el formato no es válido, la validación falla.
        return Err(validator::ValidationError::new("invalid_date_format"));
    }
    Ok(())
}
