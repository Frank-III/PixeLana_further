use socketioxide::{
    extract::{Data, SocketRef, State}, SocketIo
};
mod utils;
mod state;
use std::{collections::HashMap, sync::RwLock};
use anyhow::{bail, Result};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer};
use tracing_subscriber::FmtSubscriber;
use std::sync::Mutex;
use tracing::{info, warn};
use state::{GameState, Submission, LikeDrawInput, };

use crate::state::AddPlayerInput;

// struct Game(pub Mutex<GameState>);
#[derive(Default)]
struct Games(pub RwLock<HashMap<String, Mutex<GameState>>>);

#[derive(Debug, serde::Deserialize)]
struct SubmitRoomInput {
    #[serde(rename = "roomId")]
    pub room_id: String,
    pub submission: Submission
}

#[derive(Debug, serde::Deserialize)]
struct JoinRoomInput {
    #[serde(rename = "roomId")]
    pub room_id: String,
    pub player: AddPlayerInput
}


#[derive(Debug, serde::Deserialize)]
struct RoomLikeImgInput {
    #[serde(rename = "roomId")]
    pub room_id: String,
    pub like: LikeDrawInput 
}

#[derive(Debug, serde::Deserialize)]
struct GetRoundInfoInput{
    #[serde(rename = "roomId")]
    pub room_id: String,
    #[serde(rename = "playerIdx")]
    pub player_idx: usize
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting server");

    // Or I could make the Game => Mutex<Option<GameState>> 
    let (layer, io) = SocketIo::builder()
        .with_state(Games::default())
        .build_layer();

    io.ns("/", |s: SocketRef | {
        info!("New connection: {}", s.id);
        
        // add listen to event: newRoom
        s.on("newRoom", |socket: SocketRef, Data::<AddPlayerInput>(player), State(Games(games))| {
            info!("Received newRoom event");
            let mut games = games.write().unwrap();
            let room_id = utils::generate_room_id();
            let mut new_game = GameState::new();
            socket.join(room_id.clone()).ok();
            info!("New Room ID: {}", room_id);  
            if let Ok(players) = new_game.add_player(player, socket.id.as_str().to_string()) {
                games.insert(room_id.clone(), Mutex::new(new_game));
                // TODO: send out roomId as well
                socket.emit("roomCreated", (&players, room_id)).ok();
            }
        });

        s.on("joinRoom", |socket: SocketRef, Data::<JoinRoomInput>(JoinRoomInput {room_id, player}), State(Games(games))| {
            info!("Received startGame event");
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                if let Ok(players) = game.add_player(player, socket.id.as_str().to_string()) {
                    socket.join(room_id.clone()).ok();
                    socket.to(room_id.clone()).emit("updatePlayers", vec![&players]).ok();
                    socket.emit("updatePlayers", vec![&players]).ok();
                    return 
                }
            }
            warn!("Room Id not Found")
        });

        s.on("startGame", |socket: SocketRef, Data::<String>(room_id), State(Games(games))| {
            info!("Received startGame event");
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                if let Ok(_) = game.start_game() {
                    socket.to(room_id).emit("promptStart", ()).ok();
                    socket.emit("promptStart", ()).ok();
                    return
                }
            }
            warn!("Could not start game {:?}", room_id);
        });

        s.on("submitPrompt", |socket: SocketRef, Data::<SubmitRoomInput>(SubmitRoomInput {room_id, submission}), State(Games(games))| {
            info!("Received Game {:?} submitPrompt event: {:?}", room_id, submission);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                if let Ok(true) = game.submit_img_or_prompt(submission) {
                    socket.to(room_id).emit("promptFinished", ()).ok();
                    socket.emit("promptFinished", ()).ok();
                    return
                }
            }
            warn!("Room Id not Found")
        });

        s.on("getRoundInfo", |s: SocketRef, Data::<GetRoundInfoInput>(GetRoundInfoInput {room_id, player_idx}), State(Games(games))| {
            info!("Received Game {:?} getRoundInfo event: {:?}", room_id, player_idx);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let game = game.lock().unwrap();
                if let Ok(content) = game.send_round_info(player_idx as u8) {
                    s.emit("roundInfo", content).ok();
                }
            }
        });

        s.on("submitRoundInfo", |socket: SocketRef, Data::<SubmitRoomInput>(SubmitRoomInput {room_id, submission}), State(Games(games))| {
            info!("Received Game {:?} submitRoundInfo event: {:?}", room_id, submission);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                if let Ok(true) = game.submit_img_or_prompt(submission) {
                    info!("Game {:?} round finished", room_id);
                    if game.game_finished() {
                        socket.to(room_id).emit("gameFinished", ()).ok();
                        socket.emit("gameFinished", ()).ok();
                    } else {
                        socket.to(room_id).emit("roundFinished", ()).ok();
                        socket.emit("roundFinished", ()).ok();
                    }
                }
                return
            }
            warn!("Room Id not Found")
        });

        s.on("getAllImgsOrPrompts", |socket: SocketRef, Data::<(String, usize)>((room_id, round)), State(Games(games))| {
            info!("Received getAllImgsOrPrompts event: {:?}", round);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let game = game.lock().unwrap();
                if let Ok(all_imgs_prompts) = game.get_all_imgs_or_prompts(round as u8) {
                    socket.emit("allImgsOrPrompts", all_imgs_prompts).ok();
                }
            }
        });

        s.on("likeDrawing", |socket: SocketRef, Data::<RoomLikeImgInput>(RoomLikeImgInput {room_id, like}), State(Games(games))| {
            info!("Received Game {:?} likeDrawing event: {:?}", room_id, like);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                let round = like.player_idx;
                match game.like_img(like) {
                    Ok((best_img, like_end)) => {
                        socket.emit("bestImage", &best_img).ok();
                        socket.to(room_id.clone()).emit("bestImage", best_img).ok();
                        if !like_end {
                            socket.emit("roundImgLiked", round + 1).ok();
                            socket.to(room_id.clone()).emit("roundImgLiked", round + 1).ok();
                            let leader_board = game.get_leaderboard();
                            socket.emit("updateLeaderBoard", &leader_board).ok();
                            socket.to(room_id).emit("updateLeaderBoard", &leader_board).ok();
                        }
                    },
                    Err(e) => {
                        warn!("Error: {:?}", e);
                    }   
                }
                return
            }
            warn!("Room Id not Found")
        });

        s.on("backRoom", |socket: SocketRef, Data::<String>(room_id), State(Games(games))| {
            info!("Received backRoom event: {:?}", room_id);
            let games = games.read().unwrap();
            if let Some(game) = games.get(&room_id) {
                let mut game = game.lock().unwrap();
                if let Ok(_) = game.reset_game() {
                    socket.emit("goBackLobby", ()).ok();
                    socket.to(room_id).emit("goBackLobby", ()).ok();
                    return
                }
            }
            warn!("Room Id not Found")
        });

    });

    // logic to handle disconnects

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