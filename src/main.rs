use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

mod challenge;
mod info;
mod players;

// This constants are used in both info and players modules
// Thats why they are defined here
pub const SERVER_CURRENT_PLAYERS: u8 = 32;
pub const SERVER_MAX_PLAYERS: u8 = 128;

// NOTE: In Garry's Mod, the hard limit is 128 players
// If you change this value to a higher number, the game will simply clamp it to max 128

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:27015").expect("Could not bind to address");

    // Probably this is not the way SRCDS does it, but it works
    // How do they did this tho? I have no idea
    let mut random_numbers: HashMap<SocketAddr, i32> = HashMap::new();

    loop {
        // According to Valve, the maximum is 1400 bytes + IP/UDP headers
        let mut buf = [0; 1400];

        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data");

        // For some reason, the Source Engine Query protocol uses a 4-byte prefix before the actual payload (0xff 0xff 0xff 0xff)
        let header = buf.get(4).expect("Failed to get header"); // 4 + 1 = 5th byte

        const A2S_INFO_REQUEST_HEADER: u8 = 0x54;
        const A2S_PLAYERS_REQUEST_HEADER: u8 = 0x55;

        // Shadow the original buffer since the header is already known
        // Just skip it to the important parts
        let buf: &[u8] = &buf[5..amt];

        match header {
            &A2S_INFO_REQUEST_HEADER => info::handle_info(buf, &socket, src, &mut random_numbers),
            &A2S_PLAYERS_REQUEST_HEADER => players::handle_players(buf, &socket, src, &mut random_numbers),
            _ => {}
        }
    }
}
