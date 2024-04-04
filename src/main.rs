use std::{collections::HashMap, io, sync::Arc};

use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

type AppState = RwLock<HashMap<String, Movie>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Movie {
    id: String,
    name: String,
    year: u16,
    was_good: bool
}

impl Movie {
    pub fn new(id: &str, name: &str, year: u16, was_good: bool) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            year,
            was_good
        }
    }
}

async fn get_movie(Path(movie_id): Path<String>, State(state): State<Arc<AppState>>) -> Result<Json<Movie>, StatusCode> {
    // Add 404 error handling or some other error...
    let state = state.read().await;
    let movie = state.get(&String::from(movie_id));
    match movie {
        Some(m) => Ok(Json(m.clone())),  // get around the borrow checker... did this to save time
        None => Err(StatusCode::NOT_FOUND)
    }
}

async fn post_movie(State(state): State<Arc<AppState>>, Json(movie): Json<serde_json::Value>) -> Result<(), StatusCode> {
    // StatusCode::BAD_REQUEST
    let movie_json = serde_json::from_value::<Movie>(movie);
    match movie_json {
        Ok(movie) => {
            state.write().await.insert(movie.id.clone(), movie);
            Ok(())
        },
        Err(_) => Err(StatusCode::BAD_REQUEST)
    }
}

#[tokio::main]
async fn main() {
    // Create Axum server with the following endpoints:
    // 1. GET /movie/{id} - This should return back a movie given the id
    // 2. POST /movie - this should save movie in a DB (HashMap<String, Movie>). This movie will be sent
    // via a JSON payload. 
    
    // As a bonus: implement a caching layer so we don't need to make expensive "DB" lookups, etc.

    let shared_state = Arc::new(AppState::new(HashMap::<String, Movie>::new()));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/movie/:movie_id", get(get_movie))
        .route("/movie", post(post_movie))
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}