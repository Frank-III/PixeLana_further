use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use std::{collections::HashMap, hash::Hash};
use anyhow::{bail, Result};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use axum::routing::get;
use serde_json::Value;
use tracing_subscriber::FmtSubscriber;
use std::sync::Mutex;
use std::sync::Arc;
use tracing::info;
mod utils;
mod state;





#[derive(Default)]
struct Todos(pub Mutex<Vec<Todo>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    completed: bool,
    editing: bool,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting server");

    let (layer, io) = SocketIo::builder()
        .with_state(Todos::default())
        .build_layer();

    io.ns("/", |s: SocketRef, State(Todos(todos))| {
        info!("New connection: {}", s.id);

        let todos = todos.lock().unwrap().clone();

        // Because variadic args are not supported, array arguments are flattened.
        // Therefore to send a json array (required for the todomvc app) we need to wrap it in another array.
        s.emit("todos", [todos]).unwrap();

        s.on(
            "update-store",
            |s: SocketRef, Data::<Vec<Todo>>(new_todos), State(Todos(todos))| {
                info!("Received update-store event: {:?}", new_todos);

                let mut todos = todos.lock().unwrap();
                todos.clear();
                todos.extend_from_slice(&new_todos);

                s.broadcast().emit("update-store", [new_todos]).unwrap();
            },
        );
    });

    let app = axum::Router::new()
        .with_state(io)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // Enable CORS policy
                .layer(layer),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}