use std::collections::HashMap;
use socketioxide::extract::SocketRef;
use rand::Rng;

pub fn rotate_map(map: &HashMap<u8, String>, ) -> HashMap<u8, String> {
    let mut new_map = HashMap::new();
    let mut ordered_keys = map.keys().collect::<Vec<&u8>>();
    ordered_keys.sort();

    for key in ordered_keys {
        let next_key = if *key == (map.len() -1) as u8 { 0 } else { *key + 1 };
        new_map.insert(next_key, map.get(key).unwrap().clone());
    }

    new_map
}

pub fn broadcast_all(socket: SocketRef, data: impl serde::Serialize) {
    socket.emit("updatePlayers", &data).ok();
    socket.broadcast().emit("updatePlayers", &data).ok();
}


pub fn generate_room_id() -> String {
    let mut rng = rand::thread_rng();
    let mut room_id = String::new();
    for _ in 0..6 {
        room_id.push_str(&rng.gen_range(0..10).to_string());
    }
    room_id
}