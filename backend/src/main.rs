// Se importa 'IntoMakeServiceWithConnectInfo' para habilitar el extractor ConnectInfo.
// Este import es necesario para la línea `axum::serve` al final del archivo.


use axum::{http::Method, middleware, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

// Imports de utoipa
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// --- MÓDULOS DE LA APLICACIÓN ---
// Declaración de todos tus módulos.
mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod security;

#[cfg(test)]
mod tests;

// --- IMPORTS DE COMPONENTES ---
use crate::auth::JwtService;
use crate::config::Config;
use crate::error::ErrorPayload;
use crate::security::rate_limit_middleware;

// Se importan TODOS los modelos que se usarán en la documentación de la API.
use crate::models::{
    CreateTaskRequest, LoginRequest, LoginResponse, PaginationInfo, RegisterRequest,
    SystemStats, Task, TaskPriorityStats, TaskQueryParams, TaskStatusStats, TasksResponse,
    UpdateTaskRequest, User, UserSummary, UsersResponse, RecentActivity
};


// El AppState contiene los recursos compartidos de la aplicación.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::SqlitePool,
    pub jwt_service: JwtService,
    pub config: Config,
}


// --- DOCUMENTACIÓN OpenAPI ---
#[derive(OpenApi)]
#[openapi(
    paths(
        // Rutas existentes
        routes::root_handler,
        routes::register_user,
        routes::login_user,
        routes::get_current_user,
        routes::get_tasks,
        routes::create_task,
        routes::get_task,
        routes::update_task,
        routes::delete_task,
        // --- NUEVAS RUTAS DE ADMIN ---
        routes::get_all_users,
        routes::get_user_tasks,
        routes::get_system_stats,
    ),
    components(
        schemas(
            // Modelos existentes
            User, 
            Task, 
            LoginRequest, 
            RegisterRequest, 
            CreateTaskRequest, 
            UpdateTaskRequest,
            TaskQueryParams,
            TasksResponse,
            LoginResponse,
            ErrorPayload,
            PaginationInfo,
            // --- NUEVOS MODELOS DE ADMIN ---
            UsersResponse,
            UserSummary,
            SystemStats,
            TaskStatusStats,
            TaskPriorityStats,
            RecentActivity
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "API Status", description = "Operaciones para chequear el estado de la API"),
        (name = "Authentication", description = "Endpoints para registro, login y gestión de usuarios"),
        (name = "Tasks", description = "Gestión completa de tareas"),
        (name = "Admin", description = "Operaciones exclusivas para administradores")
    ),
    info(
        title = "API de To-Do en Rust (v2)",
        version = "0.2.0",
        description = "Una API RESTful completa con roles de usuario y seguridad avanzada."
    )
)]
struct ApiDoc;

// Este struct añade el esquema de seguridad Bearer a la documentación OpenAPI.
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}


// --- PUNTO DE ENTRADA DE LA APLICACIÓN ---
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Cargar configuración desde .env
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("Error al cargar la configuración desde .env");

    // 2. Inicializar base de datos
    let db_pool = db::init_db(&config).await?;

    // 3. Inicializar el servicio JWT
    let jwt_service = JwtService::new(&config.jwt_secret, config.jwt_expiration_hours);

    // 4. Crear el estado compartido de la aplicación
    let app_state = AppState {
        db_pool,
        jwt_service,
        config: config.clone(),
    };

    // --- 5. CONSTRUIR EL ROUTER CON LAS CAPAS DE SEGURIDAD (MIDDLEWARE) ---
    let app = Router::new()
        .route("/", axum::routing::get(routes::root_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", routes::api_router())
        .layer(
            // El orden de las capas (layers) es importante. Se aplican de abajo hacia arriba.
            // Primero el CORS para permitir peticiones desde orígenes diferentes.
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
        // Después, el rate limiting para proteger todos los endpoints contra ataques de fuerza bruta.
        .layer(
            middleware::from_fn_with_state(app_state.clone(), rate_limit_middleware)
        )
        .with_state(app_state);

    // 6. Iniciar el servidor
    let server_address_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = server_address_str.parse()?;

    println!("🚀 SERVIDOR INICIADO (v2)");
    println!("📡 Escuchando en: http://{}", addr);
    println!("📚 UI de Swagger disponible en: http://{}/swagger-ui", addr);

    let listener = TcpListener::bind(addr).await?;

    // --- SOLUCIÓN APLICADA ---
    // Se envuelve el 'app' con el servicio que provee `ConnectInfo<SocketAddr>`.
    // Esto hace que el extractor `ConnectInfo` esté disponible en los handlers y middlewares.
    

            axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

    Ok(())
}
