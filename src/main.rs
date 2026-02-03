use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Todo {
    id: i64,
    title: String,
    completed: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    completed: bool,
}

#[derive(Debug)]
enum AppError {
    NotFound,
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if matches!(err, sqlx::Error::RowNotFound) {
            Self::NotFound
        } else {
            Self::Sqlx(err)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
            Self::Sqlx(err) => {
                tracing::error!(error = %err, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response()
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not set. Copy .env.example to .env")?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(health))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            get(get_todo).patch(update_todo).delete(delete_todo),
        )
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    tracing::info!(%addr, "listening");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = sqlx::query_as::<_, Todo>(
        r#"
        SELECT id, title, completed, created_at
        FROM todos
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(todos))
}

async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title)
        VALUES ($1)
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(payload.title)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        SELECT id, title, completed, created_at
        FROM todos
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(todo))
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET completed = $1
        WHERE id = $2
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(payload.completed)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(todo))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM todos
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
