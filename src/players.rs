use std::{
    collections::HashMap,
    ffi::CString,
    net::{SocketAddr, UdpSocket},
};

use crate::{
    challenge::{generate_challenge, is_challenge_valid},
    SERVER_CURRENT_PLAYERS,
};

const A2S_PLAYER_RESPONSE_HEADER: u8 = 0x44;

pub fn handle_players(
    buf: &[u8],
    socket: &UdpSocket,
    src: SocketAddr,
    random_numbers: &mut HashMap<SocketAddr, i32>,
) {
    let challenge = match <[u8; 4]>::try_from(buf) {
        Ok(nums) => i32::from_le_bytes(nums),
        Err(_) => 0,
    };

    if challenge == 0 {
        let response = generate_challenge(src, random_numbers);

        socket.send_to(&response, src)
            .expect("Failed to send challenge response");

        return;
    }

    if !is_challenge_valid(&src, challenge, random_numbers) {
        return;
    }

    let mut response = Vec::from([
        0xff, 0xff, 0xff, 0xff,
        A2S_PLAYER_RESPONSE_HEADER,
        SERVER_CURRENT_PLAYERS,
    ]);

    for i in 0..SERVER_CURRENT_PLAYERS {
        let player_name = CString::new(format!("Player{}", i+1)).expect("Failed to create CString");
        let player_score: i32 = 0; // Player's score (usually "frags" or "kills")
        let player_duration: f32 = 32.0; // Time (in seconds) player has been connected to the server

        response.push(i);
        response.extend_from_slice(player_name.to_bytes_with_nul());
        response.extend_from_slice(&player_score.to_le_bytes());
        response.extend_from_slice(&player_duration.to_le_bytes());
    }

    socket.send_to(&response, src)
        .expect("Failed to send player response");
}
