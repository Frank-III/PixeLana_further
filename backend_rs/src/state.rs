use std::sync::Mutex;
// use std::sync::Arc;
use std::sync::RwLock;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::{collections::HashMap, hash::Hash};

use crate::utils::rotate_map;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, )]
pub struct Player {
    pub id: String,
    #[serde(rename = "pubKey")]
    pub pub_key: String,
    pub name: String,
    #[serde(rename = "isHost")]
    pub is_host: bool,
    pub avatar: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInput {
    #[serde(rename = "pubKey")]
    pub pub_key: String,
    pub name: String,
    pub avatar: String,
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    #[serde(rename = "playerIdx")]
  pub player_idx: u8,
  pub content: String
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Content {
  r#type: String,
  data: String,
  idx: u8
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AllImgOrPrompt {
  content: Vec<Content>,
  round: u8
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeDrawInput {
  pub player_idx: u8,
  pub like_idx: u8
}


#[derive(thiserror::Error, Debug, serde::Serialize)]
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
    #[error("Invalid round")]
    InvalidRound
}


#[derive(Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub round: u8,
    pub game_started: bool,
    pub round_imgs_or_prompts: Vec<HashMap<u8, String>>,
    pub prev_round_imgs_or_prompts: Option<HashMap<u8, String>>,
    pub leader_board: HashMap<String, i32>
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new()
    }
} 

impl GameState {
    pub fn new() -> GameState {
        GameState {
            players: Vec::new(),
            round: 0,
            game_started: false,
            round_imgs_or_prompts: Vec::new(),
            prev_round_imgs_or_prompts: None,
            leader_board: HashMap::new()
        }
    }

    pub fn get_leaderboard(&self) -> Vec<(String, i32)> {
      let mut leaderboard = self.leader_board.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<(String, i32)>>();
      leaderboard.sort_by(|a,b| b.1.cmp(&a.1));
      leaderboard
    }

    pub fn reset_game(&mut self) -> Result<()> {
        self.round = 0;
        self.game_started = false;
        self.round_imgs_or_prompts = Vec::new();  
        self.prev_round_imgs_or_prompts = None;
        Ok(())
    }

    pub fn add_player(&mut self, player: PlayerInput, socket_id: String) -> Result<Vec<Player>> {
        if self.game_started {
            bail!(GameError::GameStarted)
        }
        if self.players.len() >= 8 {
            bail!(GameError::GameFull)
        };
        self.players.push(Player {
            id: socket_id,
            pub_key: player.pub_key,
            name: player.name,
            is_host: self.players.len() == 0,
            avatar: player.avatar
        });
        return Ok(self.players.clone())
        // return Ok(players.len() as u8);
    }

    // TODO: add if is host check
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
        if self.round_imgs_or_prompts.len() as u8 == self.round {
            self.round_imgs_or_prompts.push(HashMap::new());
        }
        let round_imgs_or_prompts = self.round_imgs_or_prompts.get_mut(self.round as usize).unwrap();
        round_imgs_or_prompts.insert(submission.player_idx, submission.content);

          if round_imgs_or_prompts.len() as u8 == self.players.len() as u8 {
            self.round += 1;
            self.prev_round_imgs_or_prompts = Some(rotate_map(round_imgs_or_prompts));
            return Ok(true); 
          }

        Ok(false)
    }

    pub fn send_round_info(&self, player_id:u8) -> Result<Content> {
      let prev_state = self.prev_round_imgs_or_prompts.as_ref().ok_or(GameError::GameNotStarted)?;
      let round_info = prev_state.get(&player_id).ok_or(GameError::PlayerDoesNotExist)?;

      Ok({Content {
        r#type: if self.round % 2 == 0 { "story".to_string() } else { "image".to_string() },
        data: round_info.clone(),
        idx: 0
      }})
    }

    pub fn get_all_imgs_or_prompts(&self, round: u8) -> Result<AllImgOrPrompt> {
      let round_img_or_prompt = (self.round_imgs_or_prompts).iter().enumerate().map(|(idx, round_map)| {
        Content {
          r#type: if idx % 2 == 0 { "story".to_string() } else { "image".to_string() },
          data: round_map.get(&round).unwrap().clone(),
          idx: idx as u8
        }
      }).collect::<Vec<Content>>();
      Ok(AllImgOrPrompt {
        content: round_img_or_prompt,
        round: self.round + 1
    })
    }

    pub fn like_img(&mut self, input: LikeDrawInput) ->Result<(String, bool)> {
      let player_len = self.players.len() as u8;
      let img_vec_idx = if input.player_idx > input.like_idx { (input.player_idx - input.like_idx) % player_len as u8 } else { (input.like_idx - input.player_idx) % player_len as u8};
      let best_img = self.round_imgs_or_prompts.get(img_vec_idx as usize).unwrap().get(&input.like_idx).unwrap();
      let like_player = self.players.get(input.like_idx as usize).ok_or(GameError::PlayerDoesNotExist)?;
      let player_score = self.leader_board.get_mut(&like_player.pub_key).unwrap();
      *player_score += 1;
      Ok((best_img.clone(), input.player_idx == self.players.len() as u8 - 1))
    }

    pub fn game_finished(&self) -> bool {
      self.round == self.players.len() as u8
    }
}