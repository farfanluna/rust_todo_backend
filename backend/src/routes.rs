use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use std::net::SocketAddr;
use validator::Validate;

use crate::auth::AuthenticatedUser;
use crate::security::{AdminUser, AuthenticatedUserWithRole, record_login_attempt};
use crate::error::{AppError, Result};
use crate::models::{
    CreateTaskRequest, LoginRequest, LoginResponse, PaginationInfo, RegisterRequest, 
    Task, TaskQueryParams, TasksResponse, UpdateTaskRequest, User, UserSummary, 
    UsersResponse, SystemStats, TaskStatusStats, TaskPriorityStats, RecentActivity, UserLoginResponse,
    UpdateUserRoleRequest
};
use crate::AppState;
use crate::security::get_real_ip;

// --- UNIFICADOR DE RUTAS (Expuesto a `main.rs`) ---
pub fn api_router() -> Router<AppState> {
    auth_routes()
        .merge(task_routes())
        .merge(admin_routes())
}

// --- SUB-ROUTERS ---
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/me", get(get_current_user))
}

fn task_routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/stats", get(get_task_stats))
        .route("/tasks/:id", get(get_task).put(update_task).delete(delete_task))
        .route("/users", get(get_users_for_assignment))
}

fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(get_all_users))
        .route("/admin/users/:id/tasks", get(get_user_tasks))
        .route("/admin/stats", get(get_system_stats))
        .route("/admin/users/:id/role", put(update_user_role))
        // Se elimina esta línea porque `GET /tasks` ya maneja el caso de admin
        // .route("/admin/tasks", get(get_all_tasks_admin))
}

// --- Handlers de Estado de la API ---
#[utoipa::path(get, path = "/", tag = "API Status")]
pub async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": "API de To-Do funcionando",
        "version": "0.2.0",
        "features": ["rate_limiting", "admin_roles", "advanced_security"]
    }))
}

/// (ADMIN) Actualiza el rol de un usuario.
#[utoipa::path(
    put,
    path = "/admin/users/{id}/role",
    tag = "Admin",
    security(("bearer_auth" = [])),
    request_body = UpdateUserRoleRequest,
    params(("id" = i32, Path, description = "ID del usuario a modificar"))
)]
pub async fn update_user_role(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> Result<Json<User>> {
    payload.validate()?;

    // Verificar que el usuario existe
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Usuario con ID {} no encontrado", user_id)))?;

    // Actualizar el rol
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(&payload.role)
        .bind(user_id)
        .execute(&state.db_pool)
        .await?;

    let updated_user = User {
        role: payload.role,
        ..user
    };
    
    println!("->> HANDLER | Rol de usuario actualizado: (ID: {}) a '{}'", user_id, updated_user.role);
    Ok(Json(updated_user))
}

/// Obtiene estadísticas de tareas por estado para el usuario actual.
#[utoipa::path(get, path = "/tasks/stats", tag = "Tasks", security(("bearer_auth" = [])))]
pub async fn get_task_stats(
    State(state): State<AppState>,
    user: AuthenticatedUserWithRole,
) -> Result<Json<TaskStatusStats>> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT 
            SUM(CASE WHEN status = 'todo' THEN 1 ELSE 0 END) as todo,
            SUM(CASE WHEN status = 'doing' THEN 1 ELSE 0 END) as doing,
            SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as done
         FROM tasks"
    );

    if !user.is_admin() {
        query_builder.push(" WHERE user_id = ").push_bind(user.user_id);
    }

    let stats: TaskStatusStats = query_builder.build_query_as()
        .fetch_one(&state.db_pool)
        .await?;

    Ok(Json(stats))
}


/// Obtiene todos los usuarios para asignación de tareas.
#[utoipa::path(get, path = "/users", tag = "Tasks", security(("bearer_auth" = [])))]
pub async fn get_users_for_assignment(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<Vec<UserSummary>>> {
    let users: Vec<UserSummary> = sqlx::query_as(
        "SELECT u.id, u.name, u.email, u.role, u.created_at,
         COUNT(t.id) as task_count
         FROM users u
         LEFT JOIN tasks t ON u.id = t.user_id
         GROUP BY u.id
         ORDER BY u.name ASC"
    )
        .fetch_all(&state.db_pool)
        .await?;
    Ok(Json(users))
}

// --- Handlers de Autenticación ---

/// Registra un nuevo usuario en el sistema.
#[utoipa::path(post, path = "/auth/register", tag = "Authentication", request_body = RegisterRequest)]
pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<User>)> {
    payload.validate()?;

    if sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.db_pool)
        .await?
        .is_some() 
    {
        return Err(AppError::Conflict("El email ya está registrado".to_string()));
    }

    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)?;

    let user_id = sqlx::query("INSERT INTO users (name, email, password_hash, role) VALUES (?, ?, ?, 'user')")
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&password_hash)
        .execute(&state.db_pool)
        .await?
        .last_insert_rowid();

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await?;
        
    println!("->> HANDLER | Usuario registrado: {} (ID: {}, Role: {})", user.email, user_id, user.role);
    Ok((StatusCode::CREATED, Json(user)))
}


/// Autentica a un usuario y devuelve un token JWT.
#[utoipa::path(post, path = "/auth/login", tag = "Authentication", request_body = LoginRequest)]
pub async fn login_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    payload.validate()?;

    // Ahora que get_real_ip es pública, esto funcionará.
    let ip = get_real_ip(&addr, &headers);
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok());

    let user_result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.db_pool)
        .await?;

    let user = match user_result {
        Some(u) => u,
        None => {
            record_login_attempt(&state, &ip, Some(&payload.email), false, user_agent).await?;
            return Err(AppError::Authentication("Credenciales inválidas".to_string()));
        }
    };

    if !bcrypt::verify(&payload.password, &user.password_hash)? {
        record_login_attempt(&state, &ip, Some(&payload.email), false, user_agent).await?;
        return Err(AppError::Authentication("Credenciales inválidas".to_string()));
    }

    record_login_attempt(&state, &ip, Some(&payload.email), true, user_agent).await?;

    let token = state.jwt_service.generate_token(user.id)?;
    
    let user_response = UserLoginResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
    };

    // Se usa `{:?}` para imprimir el enum 'role', que deriva `Debug`
    println!("->> HANDLER | Login exitoso para: {} (Role: {:?})", user_response.email, user_response.role);
    Ok(Json(LoginResponse { token, user: user_response }))
}

/// Obtiene los datos del usuario actualmente autenticado.
#[utoipa::path(get, path = "/me", tag = "Authentication", security(("bearer_auth" = [])))]
pub async fn get_current_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<User>> {
    let user_data: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user.user_id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(Json(user_data))
}

// --- Handlers de Tareas (Con Lógica de Roles) ---

/// Crea una nueva tarea.
#[utoipa::path(post, path = "/tasks", tag = "Tasks", security(("bearer_auth" = [])), request_body = CreateTaskRequest)]
pub async fn create_task(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>)> {
    payload.validate()?;

    if !state.config.allow_past_due_dates {
        if let Some(due_date_str) = &payload.due_date {
            if let Ok(due_date) = chrono::DateTime::parse_from_rfc3339(due_date_str) {
                if due_date.date_naive() < Utc::now().date_naive() {
                    return Err(AppError::BadRequest("La fecha de vencimiento no puede ser en el pasado".to_string()));
                }
            } else {
                return Err(AppError::BadRequest("Formato de fecha de vencimiento inválido".to_string()));
            }
        }
    }
    
    let task_id = sqlx::query(
        "INSERT INTO tasks (user_id, title, description, status, priority, due_date, tags, assigned_to) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(user.user_id)
        .bind(payload.title)
        .bind(payload.description)
        .bind(payload.status.unwrap_or_else(|| "todo".to_string()))
        .bind(payload.priority.unwrap_or_else(|| "med".to_string()))
        .bind(payload.due_date)
        .bind(payload.tags)
        .bind(payload.assigned_to)
        .execute(&state.db_pool)
        .await?
        .last_insert_rowid();

    let task: Task = sqlx::query_as(
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email
         FROM tasks t
         LEFT JOIN users u ON t.user_id = u.id
         WHERE t.id = ?"
    )
        .bind(task_id)
        .fetch_one(&state.db_pool)
        .await?;
    
    println!("->> HANDLER | Tarea creada: (ID: {}) por usuario (ID: {})", task.id, user.user_id);
    Ok((StatusCode::CREATED, Json(task)))
}


/// Obtiene la lista de tareas. Los usuarios normales solo ven sus tareas, los administradores ven todas.
#[utoipa::path(
    get,
    path = "/tasks",
    tag = "Tasks",
    security(("bearer_auth" = [])),
    params(TaskQueryParams)
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<TasksResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).max(1);
    let offset = (page - 1) * per_page;

    // --- SECCIÓN CORREGIDA Y SIMPLIFICADA ---

    // 1. Empezamos con la base de la consulta, que siempre es la misma.
    let base_select = "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email";
    let count_select = "SELECT COUNT(t.id)";
    let from_clause = "FROM tasks t LEFT JOIN users u ON t.user_id = u.id";

    // 2. Construimos los builders
    let mut query_builder = sqlx::QueryBuilder::new(format!("{} {}", base_select, from_clause));
    let mut count_builder = sqlx::QueryBuilder::new(format!("{} {}", count_select, from_clause));

    // 3. Añadimos la condición del WHERE. Siempre empezamos con 'WHERE 1=1'
    //    para poder añadir 'AND' de forma segura.
    query_builder.push(" WHERE 1=1");
    count_builder.push(" WHERE 1=1");

    // 4. Si el usuario NO es admin, añadimos la condición más importante.
    if !user.is_admin() {
        query_builder.push(" AND t.user_id = ").push_bind(user.user_id);
        count_builder.push(" AND t.user_id = ").push_bind(user.user_id);
    }

    // El resto del código no cambia.
    apply_task_filters(&mut query_builder, &mut count_builder, &params, user.is_admin());

    let total_record: (i64,) = count_builder.build_query_as()
        .fetch_one(&state.db_pool)
        .await?;
    let total = total_record.0;

    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("DESC");
    
    let sort_column = match sort_by {
        "due_date" => "t.due_date",
        "priority" => "t.priority",
        "status" => "t.status",
        "title" => "t.title",
        "owner_name" if user.is_admin() => "u.name",
        _ => "t.created_at",
    };
    let sort_direction = if sort_order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };
    
    query_builder.push(format_args!(" ORDER BY {} {}", sort_column, sort_direction));
    query_builder.push(" LIMIT ").push_bind(per_page).push(" OFFSET ").push_bind(offset);

    let tasks: Vec<Task> = query_builder.build_query_as()
        .fetch_all(&state.db_pool)
        .await?;
    
    let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as i64 };

    println!("->> HANDLER | Tareas obtenidas: {} (Usuario: {}, Admin: {})", 
             tasks.len(), user.user_id, user.is_admin());

    Ok(Json(TasksResponse {
        tasks,
        pagination: PaginationInfo { page, per_page, total, total_pages },
    }))
}



/// Aplica todos los filtros de búsqueda de tareas a los QueryBuilders.
fn apply_task_filters<'a>(
    query_builder: &mut sqlx::QueryBuilder<'a, sqlx::Sqlite>,
    count_builder: &mut sqlx::QueryBuilder<'a, sqlx::Sqlite>,
    params: &'a TaskQueryParams,
    is_admin: bool,
) {
    // 1. Filtro de BÚSQUEDA (search)
    if let Some(search_term) = &params.search {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term.trim().to_lowercase());
            query_builder.push(" AND (LOWER(t.title) LIKE ").push_bind(search_pattern.clone()).push(" OR LOWER(t.description) LIKE ").push_bind(search_pattern.clone()).push(")");
            count_builder.push(" AND (LOWER(t.title) LIKE ").push_bind(search_pattern.clone()).push(" OR LOWER(t.description) LIKE ").push_bind(search_pattern.clone()).push(")");
        }
    }

    // 2. Filtro por ESTADOS MÚLTIPLES (status)
    if let Some(statuses) = &params.status {
        let status_vec: Vec<String> = statuses.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if !status_vec.is_empty() {
            query_builder.push(" AND t.status IN (");
            let mut separated = query_builder.separated(", ");
            for status in &status_vec {
                separated.push_bind(status.clone());
            }
            separated.push_unseparated(")");

            count_builder.push(" AND t.status IN (");
            let mut separated = count_builder.separated(", ");
            for status in &status_vec {
                separated.push_bind(status.clone());
            }
            separated.push_unseparated(")");
        }
    }

    // 3. Filtro por PRIORIDADES MÚLTIPLES (priority)
    if let Some(priorities) = &params.priority {
        let priority_vec: Vec<String> = priorities.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect();
        if !priority_vec.is_empty() {
            query_builder.push(" AND t.priority IN (");
            let mut separated = query_builder.separated(", ");
            for priority in &priority_vec {
                separated.push_bind(priority.clone());
            }
            separated.push_unseparated(")");
            
            count_builder.push(" AND t.priority IN (");
            let mut separated = count_builder.separated(", ");
            for priority in &priority_vec {
                separated.push_bind(priority.clone());
            }
            separated.push_unseparated(")");
        }
    }
    
    // 4. Filtro por TAGS
    if let Some(tags) = &params.tags {
        let tag_vec: Vec<String> = tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if !tag_vec.is_empty() {
            query_builder.push(" AND (");
            for (i, tag) in tag_vec.iter().enumerate() {
                if i > 0 {
                    query_builder.push(" OR ");
                }
                query_builder.push("LOWER(t.tags) LIKE ").push_bind(format!("%{}%", tag.to_lowercase()));
            }
            query_builder.push(")");

            count_builder.push(" AND (");
            for (i, tag) in tag_vec.iter().enumerate() {
                if i > 0 {
                    count_builder.push(" OR ");
                }
                count_builder.push("LOWER(t.tags) LIKE ").push_bind(format!("%{}%", tag.to_lowercase()));
            }
            count_builder.push(")");
        }
    }
    
    // 5. Filtro por RANGO DE FECHAS DE ENTREGA
    if let Some(start_date) = &params.due_date_start {
        if !start_date.is_empty() {
            query_builder.push(" AND t.due_date >= ").push_bind(start_date.clone());
            count_builder.push(" AND t.due_date >= ").push_bind(start_date.clone());
        }
    }
    if let Some(end_date) = &params.due_date_end {
        if !end_date.is_empty() {
            query_builder.push(" AND t.due_date <= ").push_bind(end_date.clone());
            count_builder.push(" AND t.due_date <= ").push_bind(end_date.clone());
        }
    }
    
    // --- FILTROS EXCLUSIVOS DE ADMINISTRADOR ---
    if is_admin {
        if let Some(user_id) = params.user_id {
            query_builder.push(" AND t.user_id = ").push_bind(user_id);
            count_builder.push(" AND t.user_id = ").push_bind(user_id);
        }
        if let Some(name) = &params.owner_name {
            if !name.is_empty() {
                let pattern = format!("%{}%", name.trim().to_lowercase());
                query_builder.push(" AND LOWER(u.name) LIKE ").push_bind(pattern.clone());
                count_builder.push(" AND LOWER(u.name) LIKE ").push_bind(pattern.clone());
            }
        }
        if let Some(email) = &params.owner_email {
            if !email.is_empty() {
                let pattern = format!("%{}%", email.trim().to_lowercase());
                query_builder.push(" AND LOWER(u.email) LIKE ").push_bind(pattern.clone());
                count_builder.push(" AND LOWER(u.email) LIKE ").push_bind(pattern.clone());
            }
        }
    }
    if let Some(assigned_to) = &params.assigned_to {
        if assigned_to == "unassigned" {
            query_builder.push(" AND (t.assigned_to IS NULL OR t.assigned_to = '')");
            count_builder.push(" AND (t.assigned_to IS NULL OR t.assigned_to = '')");
        } else if !assigned_to.is_empty() {
            let pattern = format!("%{}%", assigned_to.trim().to_lowercase());
            query_builder.push(" AND LOWER(t.assigned_to) LIKE ").push_bind(pattern.clone());
            count_builder.push(" AND LOWER(t.assigned_to) LIKE ").push_bind(pattern.clone());
        }
    }
}

/// Obtiene una tarea específica por su ID.
#[utoipa::path(
    get,
    path = "/tasks/{id}",
    tag = "Tasks",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "ID de la tarea"))
)]
pub async fn get_task(
    State(state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Path(id): Path<i64>,
) -> Result<Json<Task>> {
    let query = if user.is_admin() {
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email 
         FROM tasks t 
         LEFT JOIN users u ON t.user_id = u.id 
         WHERE t.id = ?"
    } else {
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email 
         FROM tasks t 
         LEFT JOIN users u ON t.user_id = u.id 
         WHERE t.id = ? AND t.user_id = ?"
    };

    let task = if user.is_admin() {
        sqlx::query_as::<_, Task>(query)
            .bind(id)
            .fetch_optional(&state.db_pool)
            .await?
    } else {
        sqlx::query_as::<_, Task>(query)
            .bind(id)
            .bind(user.user_id)
            .fetch_optional(&state.db_pool)
            .await?
    };

    task.ok_or_else(|| AppError::NotFound(format!("Tarea con ID {} no encontrada", id)))
        .map(Json)
}

/// Actualiza una tarea existente.
#[utoipa::path(put, path = "/tasks/{id}", tag = "Tasks", security(("bearer_auth" = [])), request_body = UpdateTaskRequest)]
pub async fn update_task(
    State(state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>> {
    payload.validate()?;

    if !state.config.allow_past_due_dates {
        if let Some(due_date_str) = &payload.due_date {
            if let Ok(due_date) = chrono::DateTime::parse_from_rfc3339(due_date_str) {
                if due_date.date_naive() < Utc::now().date_naive() {
                    return Err(AppError::BadRequest("La fecha de vencimiento no puede ser en el pasado".to_string()));
                }
            } else {
                return Err(AppError::BadRequest("Formato de fecha de vencimiento inválido".to_string()));
            }
        }
    }

    let mut tx = state.db_pool.begin().await?;

    // Verificar permisos
    let query = if user.is_admin() {
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email FROM tasks t LEFT JOIN users u ON t.user_id = u.id WHERE t.id = ?"
    } else {
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email FROM tasks t LEFT JOIN users u ON t.user_id = u.id WHERE t.id = ? AND t.user_id = ?"
    };

    let task: Task = if user.is_admin() {
        sqlx::query_as(query)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
    } else {
        sqlx::query_as(query)
            .bind(id)
            .bind(user.user_id)
            .fetch_optional(&mut *tx)
            .await?
    }
    .ok_or_else(|| AppError::NotFound(format!("Tarea con ID {} no encontrada", id)))?;

    let title = payload.title.unwrap_or(task.title);
    let description = payload.description;
    let status = payload.status.unwrap_or(task.status);
    let priority = payload.priority.unwrap_or(task.priority);
    let due_date = payload.due_date;
    let tags = payload.tags;
    let assigned_to = payload.assigned_to;

    sqlx::query(
        "UPDATE tasks SET title = ?, description = ?, status = ?, priority = ?, 
         due_date = ?, tags = ?, assigned_to = ?, updated_at = ? WHERE id = ?"
    )
        .bind(title).bind(description).bind(status).bind(priority)
        .bind(due_date).bind(tags).bind(assigned_to).bind(Utc::now().to_rfc3339()).bind(id)
        .execute(&mut *tx)
        .await?;

    let updated_task: Task = sqlx::query_as(
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email
         FROM tasks t
         LEFT JOIN users u ON t.user_id = u.id
         WHERE t.id = ?"
    )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    
    println!("->> HANDLER | Tarea actualizada: (ID: {}) por usuario (ID: {}, Admin: {})", 
             id, user.user_id, user.is_admin());
    Ok(Json(updated_task))
}

/// Elimina una tarea por su ID.
#[utoipa::path(delete, path = "/tasks/{id}", tag = "Tasks", security(("bearer_auth" = [])), params(("id" = i64, Path, description = "ID de la tarea a eliminar")))]
pub async fn delete_task(
    State(state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    let query = if user.is_admin() {
        "DELETE FROM tasks WHERE id = ?"
    } else {
        "DELETE FROM tasks WHERE id = ? AND user_id = ?"
    };

    let result = if user.is_admin() {
        sqlx::query(query)
            .bind(id)
            .execute(&state.db_pool)
            .await?
    } else {
        sqlx::query(query)
            .bind(id)
            .bind(user.user_id)
            .execute(&state.db_pool)
            .await?
    };
        
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Tarea con ID {} no encontrada", id)));
    }
    
    println!("->> HANDLER | Tarea eliminada: (ID: {}) por usuario (ID: {}, Admin: {})", 
             id, user.user_id, user.is_admin());
    Ok(StatusCode::NO_CONTENT)
}

// --- Handlers Exclusivos para Administradores ---

/// Lista todos los usuarios del sistema (solo administradores).
#[utoipa::path(get, path = "/admin/users", tag = "Admin", security(("bearer_auth" = [])))]
pub async fn get_all_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(params): Query<TaskQueryParams>, // Reutilizamos para paginación
) -> Result<Json<UsersResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).max(1);
    let offset = (page - 1) * per_page;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await?;

    let users: Vec<UserSummary> = sqlx::query_as(
        "SELECT u.id, u.name, u.email, u.role, u.created_at,
         COUNT(t.id) as task_count
         FROM users u
         LEFT JOIN tasks t ON u.id = t.user_id
         GROUP BY u.id
         ORDER BY u.created_at DESC
         LIMIT ? OFFSET ?"
    )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await?;

    let total_pages = if total.0 == 0 { 0 } else { (total.0 as f64 / per_page as f64).ceil() as i64 };

    Ok(Json(UsersResponse {
        users,
        pagination: PaginationInfo {
            page,
            per_page,
            total: total.0,
            total_pages,
        },
    }))
}

/// Obtiene las tareas de un usuario específico (solo administradores).
#[utoipa::path(
    get, 
    path = "/admin/users/{id}/tasks", 
    tag = "Admin", 
    security(("bearer_auth" = [])),
    params(("id" = i32, Path, description = "ID del usuario"))
)]
pub async fn get_user_tasks(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(user_id): Path<i32>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<TasksResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).max(1);
    let offset = (page - 1) * per_page;

    // Verificar que el usuario existe
    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)")
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await?;

    if !user_exists {
        return Err(AppError::NotFound(format!("Usuario con ID {} no encontrado", user_id)));
    }

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await?;

    let tasks: Vec<Task> = sqlx::query_as(
        "SELECT t.id, t.user_id, t.title, t.description, t.status, t.priority, t.due_date, t.created_at, t.updated_at, t.tags, t.assigned_to, u.name as owner_name, u.email as owner_email
         FROM tasks t
         LEFT JOIN users u ON t.user_id = u.id
         WHERE t.user_id = ?
         ORDER BY t.created_at DESC
         LIMIT ? OFFSET ?"
    )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await?;

    let total_pages = if total.0 == 0 { 0 } else { (total.0 as f64 / per_page as f64).ceil() as i64 };

    Ok(Json(TasksResponse {
        tasks,
        pagination: PaginationInfo {
            page,
            per_page,
            total: total.0,
            total_pages,
        },
    }))
}




/// (ADMIN) Obtiene estadísticas agregadas del sistema de forma eficiente.
#[utoipa::path(get, path = "/admin/stats", tag = "Admin", security(("bearer_auth" = [])))]
pub async fn get_system_stats(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<SystemStats>> {
    
    // --- PASO 1: Obtener las estadísticas que no dependen de la tabla 'tasks' ---
    // De esta forma, si no hay tareas, al menos obtenemos el conteo de usuarios.
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool).await?;

    let new_users_today: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE DATE(created_at) = DATE('now')")
        .fetch_one(&state.db_pool).await?;

    // --- PASO 2: Definir el struct para las estadísticas de tareas y añadir la receta ---
    
    // CORRECCIÓN: Se añade `#[derive(sqlx::FromRow)]` para que sqlx sepa cómo
    // mapear la fila de la base de datos a este struct.
    #[derive(sqlx::FromRow)]
    struct TaskStatsRow {
        total_tasks: i64,
        todo_count: i64,
        doing_count: i64,
        done_count: i64,
        low_priority: i64,
        med_priority: i64,
        high_priority: i64,
        tasks_created_today: i64,
        tasks_completed_today: i64,
    }
    
    // --- PASO 3: Ejecutar la consulta para obtener las estadísticas de tareas ---
    let task_stats: TaskStatsRow = sqlx::query_as(
        r#"
        SELECT
            COALESCE(COUNT(*), 0) as total_tasks,
            COALESCE(SUM(CASE WHEN status = 'todo' THEN 1 ELSE 0 END), 0) as todo_count,
            COALESCE(SUM(CASE WHEN status = 'doing' THEN 1 ELSE 0 END), 0) as doing_count,
            COALESCE(SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END), 0) as done_count,
            COALESCE(SUM(CASE WHEN priority = 'low' THEN 1 ELSE 0 END), 0) as low_priority,
            COALESCE(SUM(CASE WHEN priority = 'med' THEN 1 ELSE 0 END), 0) as med_priority,
            COALESCE(SUM(CASE WHEN priority = 'high' THEN 1 ELSE 0 END), 0) as high_priority,
            COALESCE((SELECT COUNT(*) FROM tasks WHERE DATE(created_at) = DATE('now')), 0) as tasks_created_today,
            COALESCE((SELECT COUNT(*) FROM tasks WHERE status = 'done' AND DATE(updated_at) = DATE('now')), 0) as tasks_completed_today
        FROM tasks
        "#
    )
    .fetch_optional(&state.db_pool) // Usamos fetch_optional para que no falle si no hay tareas
    .await?
    .unwrap_or(TaskStatsRow { // Si no devuelve nada (tabla vacía), usamos valores por defecto.
        total_tasks: 0, todo_count: 0, doing_count: 0, done_count: 0,
        low_priority: 0, med_priority: 0, high_priority: 0,
        tasks_created_today: 0, tasks_completed_today: 0
    });


    Ok(Json(SystemStats {
        total_users: total_users.0,
        total_tasks: task_stats.total_tasks,
        tasks_by_status: TaskStatusStats {
            todo: task_stats.todo_count,
            doing: task_stats.doing_count,
            done: task_stats.done_count,
        },
        tasks_by_priority: TaskPriorityStats {
            low: task_stats.low_priority,
            med: task_stats.med_priority,
            high: task_stats.high_priority,
        },
        recent_activity: RecentActivity {
            new_users_today: new_users_today.0,
            tasks_created_today: task_stats.tasks_created_today,
            tasks_completed_today: task_stats.tasks_completed_today,
        },
    }))
}
