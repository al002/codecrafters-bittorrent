use serde_json;
use std::env;
use anyhow::{Result, Ok, anyhow};

// Available if you need it!
// use serde_bencode;

fn decode_string_value(encoded_value: &str) -> Result<(serde_json::Value, &str),  anyhow::Error> {
    // Example: "5:hello" -> "hello"

    match encoded_value.split_once(":") {
        Some((size, rest)) => {
            let size = size.parse::<usize>().map_err(|_| anyhow!("Invalid size: {}", size))?;

            let (word, rest) = rest.split_at(size);
            return Ok((word.into(), rest));
        },
        None => Err(anyhow!("Invalid bencode syntax: {}", encoded_value)),
    }
}

fn decode_integer_value(encoded_value: &str) -> Result<(serde_json::Value, &str),  anyhow::Error> {
    match encoded_value.split_at(1).1.split_once('e') {
        Some((n, rest)) => {
            let n = n.parse::<i64>()?;
            return Ok((serde_json::Value::Number(n.into()), rest));
        },
        None => Err(anyhow!("Invalid bencode syntax: {}", encoded_value)),
    }
}

fn decode_list_value(encoded_value: &str) -> Result<(serde_json::Value, &str),  anyhow::Error> {
    let mut rest = &encoded_value[1..];

    let mut v = vec![];

    while let Some(next_char) = rest.chars().next() {
        if next_char == 'e' {
            return Ok((serde_json::Value::Array(v), &rest[1..]));
        }

        let (decoded_value, new_rest) = decode_bencoded_value(rest)?;
        v.push(decoded_value);
        rest = new_rest;
    }

    Err(anyhow!(
        "Invalid bencode syntax: list not terminated with 'e'"
    ))
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<(serde_json::Value, &str)> {
    // If encoded_value starts with a digit, it's a number
    let next_char = encoded_value.chars().next().unwrap();
    match next_char {
        n if n.is_digit(10) => decode_string_value(encoded_value),
        'i' => decode_integer_value(encoded_value),
        'l' => decode_list_value(encoded_value),
        _ => Err(anyhow!("Invalid bencode syntax: {}", encoded_value)),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.expect("String").0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
