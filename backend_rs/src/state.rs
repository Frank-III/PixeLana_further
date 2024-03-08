use std::ops::Sub;
use std::sync::Mutex;
use std::sync::Arc;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::{collections::HashMap, hash::Hash};

use crate::utils::rotate_map;

#[derive(Debug, serde::Deserialize)]
pub struct Player {
    pub id: String,
    pub pub_key: String,
    pub name: String,
    pub is_host: bool,
    pub avatar: String,
}

pub enum ClientEvent {

}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Submission {
  pub player_id: u8,
  pub content: String
}


#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("Game is full")]
    GameFull,
    #[error("Player already exists")]
    PlayerExists,
    #[error("Not enough players")]
    NotEnoughPlayers,
    #[error("Player does not exist")]
    PlayerDoesNotExist,
    #[error("Game has not started")]
    GameNotStarted,
    #[error("Game has already started")]
    GameStarted,
    #[error("Game is over")]
    GameOver,
    #[error("Invalid player id")]
    InvalidPlayerId,
    #[error("Invalid round")]
    InvalidRound
}

#[derive(Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub round: u8,
    pub game_started: bool,
    pub round_imgs_or_prompts: Mutex<Vec<HashMap<u8, String>>>,
    pub prev_round_imgs_or_prompts: Option<Mutex<HashMap<u8, String>>>,
    pub leader_board: Mutex<HashMap<u8, i32>>
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: Vec::new(),
            round: 0,
            game_started: false,
            round_imgs_or_prompts: Mutex::new(Vec::new()),
            prev_round_imgs_or_prompts: None,
            leader_board: Mutex::new(HashMap::new())
        }
    }

    pub fn reset_game(&mut self) -> Result<()> {
        self.round = 0;
        self.game_started = false;
        self.round_imgs_or_prompts = Mutex::new(Vec::new());  
        self.prev_round_imgs_or_prompts = None;
        Ok(())
    }

    pub fn add_player(&mut self, player: Player) -> Result<u8> {
        if self.players.len() >= 8 {
            bail!(GameError::GameFull)
        };
        self.players.push(player);
        return Ok(self.players.len() as u8);
    }

    pub fn start_game(&mut self) -> Result<()> {
        if self.game_started {
            bail!(GameError::GameStarted)
        }
        if self.players.len() < 3 {
            bail!(GameError::NotEnoughPlayers)
        }
        self.game_started = true;
        Ok(())
    }

    pub fn submit_img_or_prompt(&mut self, submission: Submission) -> Result<bool> {
        let mut round_imgs_or_prompts = self.round_imgs_or_prompts.lock().unwrap();
        if round_imgs_or_prompts.len() as u8 == self.round {
            round_imgs_or_prompts.push(HashMap::new());
        }
        let round_imgs_or_prompts = round_imgs_or_prompts.get_mut(self.round as usize).unwrap();
        round_imgs_or_prompts.insert(submission.player_id, submission.content);

        if round_imgs_or_prompts.len() as u8 == self.players.len() as u8 {
          self.prev_round_imgs_or_prompts = Some(Mutex::new(rotate_map(round_imgs_or_prompts)))
          return Ok(true); 
        }

        Ok(false)
    }

    pub fn send_round_info() -> Result<()> {
        Ok(())
    }

    pub fn get_all_imgs_or_prompts(&self) -> Result<HashMap<u8, String>> {
        let round_imgs_or_prompts = self.round_imgs_or_prompts.lock().unwrap();
        let round_imgs_or_prompts = round_imgs_or_prompts.get(self.round as usize).unwrap();
        Ok(round_imgs_or_prompts.clone())
    }

}