use axum::{
    http::{header::IntoIter, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use sqlx::{
    migrate::{self, MigrateDatabase},
    query, Pool, Sqlite, SqlitePool,
};
use std::fs::OpenOptions;

const DB_URL: &str = "sqlite://todos.db";

#[derive(Clone)]
struct AppState {
    pub db: Pool<Sqlite>,
}

pub type ApiResponse = Result<Response, ApiError>;
pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("someting went wrong {:?}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

mod endpoints;
#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    // initialize tracing
    tracing_subscriber::fmt::init();

    let _ = OpenOptions::new().create(true).open("todos.json");

    // build our application with a route
    // `GET /` goes to `root`
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await.expect("DHDH");
    }
    let db = SqlitePool::connect(&db_url).await.expect("F");

    /* let result = sqlx::query!("SELECT * FROM todos"); */

    // `POST /users` goes to `create_user`
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migration");

    let state = AppState { db };

    // run our app with hyper
    let app = Router::<AppState>::new()
        .merge(endpoints::todos::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
