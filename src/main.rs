use anyhow::{anyhow, Error, Result};
use serde_json::{self, Map};
use std::{env, fs, path::Path, str};

// Available if you need it!
// use serde_bencode;

struct DecodeResult<'a>(serde_json::Value, &'a [u8]);

fn decode_string_value(encoded_value: &[u8]) -> Result<DecodeResult, Error> {
    // Example: "5:hello" -> "hello"

    match encoded_value.iter().position(|&c| c == b':') {
        Some(colon_pos) => {
            let size_str = str::from_utf8(&encoded_value[..colon_pos]).unwrap();
            let size = size_str
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid size: {}", size_str))?;
            let end = colon_pos + 1 + size;
            let word = String::from_utf8_lossy(&encoded_value[colon_pos + 1..end]);

            return Ok(DecodeResult(
                serde_json::Value::String(word.to_string()),
                &encoded_value[end..],
            ));
        }
        None => Err(anyhow!("Invalid bencode syntax: {:?}", encoded_value)),
    }
}

fn decode_integer_value(encoded_value: &[u8]) -> Result<DecodeResult, Error> {
    match encoded_value.iter().position(|&c| c == b'e') {
        Some(end_pos) => {
            let n = String::from_utf8_lossy(&encoded_value[1..end_pos]);
            let n = n.parse::<i64>()?;

            return Ok(DecodeResult(
                serde_json::Value::Number(n.into()),
                &encoded_value[end_pos + 1..],
            ));
        }
        None => Err(anyhow!("Invalid bencode syntax: {:?}", encoded_value)),
    }
}

fn decode_list_value(encoded_value: &[u8]) -> Result<DecodeResult, Error> {
    let mut rest = &encoded_value[1..];

    let mut v = vec![];

    while let Some(next_char) = rest.iter().next() {
        if *next_char == b'e' {
            return Ok(DecodeResult(serde_json::Value::Array(v), &rest[1..]));
        }

        let DecodeResult(decoded_value, new_rest) = decode_bencoded_value(rest)?;
        v.push(decoded_value);
        rest = new_rest;
    }

    Err(anyhow!(
        "Invalid bencode syntax: list not terminated with 'e'"
    ))
}

fn decode_dictionary_value(encoded_value: &[u8]) -> Result<DecodeResult, Error> {
    let mut rest = &encoded_value[1..];

    let mut m: Map<String, serde_json::Value> = Map::new();

    if let Some(e) = rest.iter().next_back() {
        if *e != b'e' {
            return Err(anyhow!(
                "Invalid bencode syntax: dictionary not terminated with 'e'"
            ));
        }
    }

    while let Some(next_char) = rest.iter().next() {
        if *next_char == b'e' {
            break;
        }

        let DecodeResult(key, new_rest) = decode_bencoded_value(rest)?;

        let DecodeResult(value, new_rest) = decode_bencoded_value(new_rest)?;

        m.insert(key.as_str().unwrap().into(), value);
        rest = new_rest;
    }

    Ok(DecodeResult(serde_json::Value::Object(m), &rest[1..]))
}

fn decode_bencoded_value(encoded_value: &[u8]) -> Result<DecodeResult, Error> {
    // If encoded_value starts with a digit, it's a number
    let next_char = encoded_value.iter().next().unwrap();
    match next_char {
        n if n.is_ascii_digit() => decode_string_value(encoded_value),
        b'i' => decode_integer_value(encoded_value),
        b'l' => decode_list_value(encoded_value),
        b'd' => decode_dictionary_value(encoded_value),
        _ => Err(anyhow!("Invalid bencode syntax: {:?}", encoded_value)),
    }
}

fn parse_info(file_path: &Path) -> Result<serde_json::Value, Error> {
    match fs::read(file_path) {
        Ok(data) => {
            let DecodeResult(value, _) = decode_bencoded_value(&data).unwrap();

            return Ok(value);
        }
        Err(_) => Err(anyhow!("file not exists: {}", file_path.to_str().unwrap())),
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
        let decoded_value = decode_bencoded_value(encoded_value.as_bytes());
        println!("{}", decoded_value.expect("String").0.to_string());
    } else if command == "info" {
        let p = &args[2];
        if let Ok(json) = parse_info(Path::new(p)) {
            println!(
                "Tracker URL: {}",
                json.get("announce").unwrap().as_str().unwrap()
            );

            println!(
                "Length: {}",
                json.get("info").unwrap().get("length").unwrap()
            );
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
