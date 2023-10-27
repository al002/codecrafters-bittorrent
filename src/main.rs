use anyhow::{anyhow, Error, Ok, Result};
use serde_json::{self, Map};
use std::env;

// Available if you need it!
// use serde_bencode;

struct DecodeResult<'a>(serde_json::Value, &'a str);

fn decode_string_value(encoded_value: &str) -> Result<DecodeResult, Error> {
    // Example: "5:hello" -> "hello"

    match encoded_value.split_once(":") {
        Some((size, rest)) => {
            let size = size
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid size: {}", size))?;

            let (word, rest) = rest.split_at(size);
            return Ok(DecodeResult(word.into(), rest));
        }
        None => Err(anyhow!("Invalid bencode syntax: {}", encoded_value)),
    }
}

fn decode_integer_value(encoded_value: &str) -> Result<DecodeResult, Error> {
    match encoded_value.split_at(1).1.split_once('e') {
        Some((n, rest)) => {
            let n = n.parse::<i64>()?;
            return Ok(DecodeResult(serde_json::Value::Number(n.into()), rest));
        }
        None => Err(anyhow!("Invalid bencode syntax: {}", encoded_value)),
    }
}

fn decode_list_value(encoded_value: &str) -> Result<DecodeResult, Error> {
    let mut rest = &encoded_value[1..];

    let mut v = vec![];

    while let Some(next_char) = rest.chars().next() {
        if next_char == 'e' {
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

fn decode_dictionary_value(encoded_value: &str) -> Result<DecodeResult, Error> {
    let mut rest = &encoded_value[1..];

    let mut m = Map::new();
    let mut v: Vec<serde_json::Value> = vec![];

    if let Some(e) = rest.chars().next_back() {
        if e != 'e' {
            return Err(anyhow!(
                "Invalid bencode syntax: dictionary not terminated with 'e'"
            ));
        }
    }

    while let Some(next_char) = rest.chars().next() {
        if next_char == 'e' {
            break;
        }

        let DecodeResult(decoded_value, new_rest) = decode_bencoded_value(rest)?;
        v.push(decoded_value);
        rest = new_rest;
    }

    let even_elements: Vec<serde_json::Value> = v
        .clone()
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i % 2 == 0)
        .map(|(_, e)| e)
        .collect();

    let odd_elements: Vec<serde_json::Value> = v
        .clone()
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i % 2 != 0)
        .map(|(_, e)| e)
        .collect();

    let pairs = even_elements.into_iter().zip(odd_elements);

    for p in pairs {
        m.insert(p.0.to_string(), p.1);
    }

    Ok(DecodeResult(serde_json::Value::Object(m), &rest[1..]))
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<DecodeResult> {
    // If encoded_value starts with a digit, it's a number
    let next_char = encoded_value.chars().next().unwrap();
    match next_char {
        n if n.is_digit(10) => decode_string_value(encoded_value),
        'i' => decode_integer_value(encoded_value),
        'l' => decode_list_value(encoded_value),
        'd' => decode_dictionary_value(encoded_value),
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
