use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, SocketRef, State}, socket::Socket, SocketIo
};
mod utils;
mod state;
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
use state::{GameState, Player, Content, Submission, LikeDrawInput, PlayerInput};
struct Game(pub Mutex<GameState>);


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting server");

    // Or I could make the Game => Mutex<Option<GameState>> 
    let (layer, io) = SocketIo::builder()
        .with_state(Game(Mutex::new(GameState::default())))
        .build_layer();

    io.ns("/", |s: SocketRef | {
        info!("New connection: {}", s.id);
        s.on("addPlayer", |socket: SocketRef, Data::<PlayerInput>(player), State(Game(game))| {
            info!("Received addPlayer event: {:?}", player);
            let mut game = game.lock().unwrap();    
            if let Ok(players) = game.add_player(player, socket.id.as_str().to_string()) {
                socket.emit("updatePlayers", players.clone()).ok();
                socket.broadcast().emit("updatePlayers", players).ok();
                socket.emit("updateLeaderBoard", game.get_leaderboard()).ok();
                // io_clone.emit("updatePlayers", players).ok(); // Emit to all clients
                // io_clone.emit("updateLeaderBoard", game.get_leaderboard()).ok(); // Emit to all clients
            }
        });

        s.on("startGame", |s: SocketRef, State(Game(game)), io: State<SocketIo>| {
            info!("Received startGame event");
            let mut game = game.lock().unwrap();
            if let Ok(_) = game.start_game() {
                io.emit("gameStarted", game.get_leaderboard()).ok();
            }
        });

        s.on("submitPrompt", |s: SocketRef, Data::<Submission>(submission), State(Game(game)), io: State<SocketIo>| {
            info!("Received submitPrompt event: {:?}", submission);
            let mut game = game.lock().unwrap();
            if let Ok(true) = game.submit_img_or_prompt(submission) {
                io.emit("promptFinished", game.get_leaderboard()).ok();
            }
        });

        s.on("getRoundInfo", |s: SocketRef, Data::<usize>(player_idx), State(Game(game)), io: State<SocketIo>| {
            info!("Received getRoundInfo event: {:?}", player_idx);
            let game = game.lock().unwrap();
            if let Ok(content) = game.send_round_info(player_idx as u8) {
                s.emit("roundInfo", content).ok();
            }
        });

        s.on("submitRoundInfo", |s: SocketRef, Data::<Submission>(submission), State(Game(game)), io: State<SocketIo>| {
            info!("Received submitRoundInfo event: {:?}", submission);
            let mut game = game.lock().unwrap();
            if let Ok(true) = game.submit_img_or_prompt(submission) {
                if game.game_finished() {
                    io.emit("gameFinished", ()).ok();
                } else {
                io.emit("roundFinished", ()).ok();
                }
            }
        });

        s.on("getAllImgsOrPrompts", |s: SocketRef, Data::<usize>(round), State(Game(game)), io: State<SocketIo>| {
            info!("Received getAllImgsOrPrompts event: {:?}", round);
            let game = game.lock().unwrap();
            if let Ok(all_imgs_prompts) = game.get_all_imgs_or_prompts(round as u8) {
                s.emit("allImgsOrPrompts", all_imgs_prompts).ok();
            }
        });

        s.on("likeDrawing", |s: SocketRef, Data::<LikeDrawInput>(input), State(Game(game)), io: State<SocketIo>| {
            info!("Received likeDrawing event: {:?}", input);
            let mut game = game.lock().unwrap();
            match game.like_img(input) {
                Ok((best_img, true)) => {
                    io.emit("bestImg", best_img).ok();
                },
                Ok((best_img, false)) => {
                    io.emit("bestImg", best_img).ok();
                },
                Err(e) => {
                    info!("Error: {:?}", e);
                }   
            }
        });

        s.on("backRoom", |s: SocketRef, State(Game(game)), io: State<SocketIo>| {
            info!("Received backRoom event");
            let mut game = game.lock().unwrap();
            if let Ok(_) = game.reset_game() {
                io.emit("backRoom", ()).ok();
            }
        });

    });

    let app = axum::Router::new()
        .with_state(io)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // Enable CORS policy
                .layer(layer),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}