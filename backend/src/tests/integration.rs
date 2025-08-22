#![allow(unused_imports)]
use super::*;
use crate::{
    routes::api_router,
    auth::{AuthenticatedUser, JwtService},
    config::Config,
    db::init_db,
    models::{
        CreateTaskRequest, LoginRequest, LoginResponse, RegisterRequest, Task, TasksResponse,
        UpdateTaskRequest, User, UserLoginResponse,
    },
    AppState,
};
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::ServiceBuilderExt;
use axum::extract::connect_info::MockConnectInfo;

async fn setup_test_app() -> (Router, AppState) {
    let config = Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test_secret".to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        jwt_expiration_hours: 24,
        allow_past_due_dates: false,
    };
    let db_pool = init_db(&config).await.unwrap();
    let jwt_service = JwtService::new("test_secret", config.jwt_expiration_hours);
    let state = AppState {
        db_pool,
        jwt_service,
        config,
    };
    let app = api_router()
        .with_state(state.clone())
        .layer(MockConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3000))));
    (app, state)
}

async fn register_and_login_user(
    app: &Router,
    name: &str,
    email: &str,
    password: &str,
) -> (UserLoginResponse, String) {
    let register_payload = RegisterRequest {
        name: name.to_string(),
        email: email.to_string(),
        password: password.to_string(),
    };

    let req = Request::builder()
        .method(Method::POST)
        .uri("/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let login_payload = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let req = Request::builder()
        .method(Method::POST)
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let login_response: LoginResponse = serde_json::from_slice(&body).unwrap();

    (login_response.user, login_response.token)
}

#[tokio::test]
async fn test_task_filtering() {
    let (app, _state) = setup_test_app().await;
    let (_user, token) = register_and_login_user(&app, "Test User", "test@example.com", "password").await;

    // Create some tasks
    let tasks_to_create = vec![
        ("Task 1", "Description 1", "todo", "high", "tag1,tag2"),
        ("Task 2", "Description 2", "doing", "med", "tag2,tag3"),
        ("Task 3", "Description 3", "done", "low", "tag3,tag4"),
    ];

    for (title, description, status, priority, tags) in tasks_to_create {
        let create_task_payload = json!({
            "title": title,
            "description": description,
            "status": status,
            "priority": priority,
            "tags": tags,
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/tasks")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&create_task_payload).unwrap()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    // Test filtering by search term
    let req = Request::builder()
        .method(Method::GET)
        .uri("/tasks?search=Task%201")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let tasks_response: TasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks_response.tasks.len(), 1);
    assert_eq!(tasks_response.tasks[0].title, "Task 1");

    // Test filtering by tags
    let req = Request::builder()
        .method(Method::GET)
        .uri("/tasks?tags=tag1")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let tasks_response: TasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks_response.tasks.len(), 1);
    assert_eq!(tasks_response.tasks[0].title, "Task 1");

    // Test filtering by assigned_to (unassigned)
    let req = Request::builder()
        .method(Method::GET)
        .uri("/tasks?assigned_to=unassigned")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let tasks_response: TasksResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tasks_response.tasks.len(), 3);
}

#[tokio::test]
async fn test_task_crud_with_due_date() {
    let (app, _state) = setup_test_app().await;
    let (_user, token) = register_and_login_user(&app, "Test User", "test@example.com", "password").await;

    // 1. Create a task with a future due date to avoid validation errors
    let due_date = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    let create_task_payload = json!({
        "title": "Task with due date",
        "description": "This is a test task",
        "due_date": due_date,
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri("/tasks")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&create_task_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let created_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(created_task.title, "Task with due date");

    // 2. Update the task
    let update_task_payload = json!({
        "title": "Updated task with due date",
    });

    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/tasks/{}", created_task.id))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&update_task_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let updated_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_task.title, "Updated task with due date");

    // 3. Delete the task
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/tasks/{}", created_task.id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
