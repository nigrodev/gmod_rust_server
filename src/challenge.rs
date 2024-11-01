use std::{collections::HashMap, net::SocketAddr};

const CHALLENGE_RESPONSE_HEADER: u8 = 0x41;

pub fn generate_challenge(
    src: SocketAddr,
    random_numbers: &mut HashMap<SocketAddr, i32>,
) -> Vec<u8> {
    let random_number = rand::random::<i32>();

    random_numbers.insert(src, random_number);

    [0xff, 0xff, 0xff, 0xff, CHALLENGE_RESPONSE_HEADER]
        .iter()
        .copied()
        .chain(random_number.to_le_bytes().iter().copied())
        .collect::<Vec<u8>>()
}

pub fn is_challenge_valid(
    src: &SocketAddr,
    challenge_number: i32,
    random_numbers: &mut HashMap<SocketAddr, i32>,
) -> bool {
    let result = match random_numbers.get(src) {
        Some(number) => challenge_number == *number,
        None => false,
    };

    random_numbers.remove(&src); // remove the challenge even if it's invalid

    result
}
