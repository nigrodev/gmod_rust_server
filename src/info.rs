use memchr::memchr;
use std::{
    collections::HashMap,
    ffi::CStr,
    net::{SocketAddr, UdpSocket},
};

use crate::{
    challenge::{generate_challenge, is_challenge_valid},
    SERVER_CURRENT_PLAYERS, SERVER_MAX_PLAYERS,
};

#[allow(dead_code)]
enum ServerType {
    Dedicated,
    NonDedicated,
    SourceTV,
}

impl Into<u8> for ServerType {
    fn into(self) -> u8 {
        match self {
            ServerType::Dedicated => b'd',    // d
            ServerType::NonDedicated => b'l', // l
            ServerType::SourceTV => b'p',     // p
        }
    }
}

#[allow(dead_code)]
enum Environment {
    Linux,
    Windows,
    Mac,
}

impl Into<u8> for Environment {
    fn into(self) -> u8 {
        match self {
            Environment::Linux => b'l',
            Environment::Windows => b'w',
            Environment::Mac => b'm',
        }
    }
}

pub fn handle_info(
    buf: &[u8],
    socket: &UdpSocket,
    src: SocketAddr,
    random_numbers: &mut HashMap<SocketAddr, i32>,
) {
    let null_pos = memchr(0, buf).expect("No null byte found in the buffer");
    let subslice = &buf[..=null_pos];
    let payload: &CStr = CStr::from_bytes_with_nul(subslice).expect("Invalid C String");

    let challenge_buf = buf.get(null_pos + 1..null_pos + 5);

    let challenge = match challenge_buf {
        Some(challenge_buf) => match <[u8; 4]>::try_from(challenge_buf) {
            Ok(nums) => Some(i32::from_le_bytes(nums)),
            Err(_) => None,
        },
        None => None,
    };

    const A2S_INFO_REQUEST_PAYLOAD: &'static CStr = c"Source Engine Query";

    if payload != A2S_INFO_REQUEST_PAYLOAD {
        return;
    };

    let Some(challenge) = challenge else {
        let response = generate_challenge(src, random_numbers);

        socket.send_to(&response, src)
            .expect("Failed to send challenge response");

        return;
    };

    if !is_challenge_valid(&src, challenge, random_numbers) {
        return;
    }

    let response = create_info_response();

    socket
        .send_to(&response, src)
        .expect("Failed to send info response");
}

fn create_info_response() -> Vec<u8> {
    // Fixed values?
    const A2S_INFO_RESPONSE_HEADER: u8 = 0x49;
    const PROTOCOL_VERSION: u8 = 0x11;

    // Dynamic Values
    const SERVER_NAME: &'static CStr = c"Gmod Server";
    const SERVER_LOADED_MAP: &'static CStr = c"gm_construct";
    const SERVER_GAME_DIR: &'static CStr = c"garrysmod";
    const SERVER_GAMEMODE: &'static CStr = c"Sandbox";
    const GAME_STEAM_APP_ID: i16 = 4000;
    const SERVER_NUM_BOTS: u8 = 0;

    // Never seen this being used in the wild, but it exists
    const SERVER_TYPE: ServerType = ServerType::Dedicated;
    const SERVER_ENVIRONMENT: Environment = Environment::Windows;

    const SERVER_REQUIRES_PASSWORD: bool = false; // false for public, true for private
    const SERVER_USES_VAC: bool = true; // false for unsecured, true for secured

    const SERVER_LAST_UPDATE_VERSION: &'static CStr = c"2024.10.29"; // Hardcoded: 29 October 2024
    const EXTRA_DATA_FLAGS: u8 = 0x80 | 0x10 | 0x20 | 0x01; // Server Port Number, Server SteamID, Server Tags and Game ID
    const SERVER_PORT: i16 = 27015; // Default port

    // ?? I have no idea what both of these are
    // NOTE: Here these values are just random numbers with a bunch of zeroes
    // Probably they are important for something, but it seems to work with them like this
    const SERVER_STEAM_ID: u64 = 1000000000000000000;
    const SERVER_GAME_ID: u64 = 10000000000000000000;

    // gm is the gamemode
    // ver is probably the version of the gamemode or server
    // gmc ???? it defaults to "other" on a clean sandbox server
    const SERVER_TAGS: &'static CStr = c" gm:sandbox gmc:other ver:241029";

    // The order of the fields is important
    // NOTE: As written in the Valve Developer Wiki, all strings are null-terminated and all integers are little-endian
    let mut response = Vec::from([
        0xff, 0xff, 0xff, 0xff,
        A2S_INFO_RESPONSE_HEADER,
        PROTOCOL_VERSION,
    ]);
    response.extend_from_slice(SERVER_NAME.to_bytes_with_nul());
    response.extend_from_slice(SERVER_LOADED_MAP.to_bytes_with_nul());
    response.extend_from_slice(SERVER_GAME_DIR.to_bytes_with_nul());
    response.extend_from_slice(SERVER_GAMEMODE.to_bytes_with_nul());
    response.extend_from_slice(&GAME_STEAM_APP_ID.to_le_bytes());
    response.push(SERVER_CURRENT_PLAYERS);
    response.push(SERVER_MAX_PLAYERS);
    response.push(SERVER_NUM_BOTS);
    response.push(SERVER_TYPE.into());
    response.push(SERVER_ENVIRONMENT.into());
    response.push(SERVER_REQUIRES_PASSWORD as u8);
    response.push(SERVER_USES_VAC as u8);
    response.extend_from_slice(SERVER_LAST_UPDATE_VERSION.to_bytes_with_nul());
    response.push(EXTRA_DATA_FLAGS);
    response.extend_from_slice(&SERVER_PORT.to_le_bytes());
    response.extend_from_slice(&SERVER_STEAM_ID.to_le_bytes());
    response.extend_from_slice(SERVER_TAGS.to_bytes_with_nul());
    response.extend_from_slice(&SERVER_GAME_ID.to_le_bytes());

    response
}
