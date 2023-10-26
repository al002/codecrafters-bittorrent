use serde_json::{self, Number};
use std::env;

// Available if you need it!
// use serde_bencode;

fn decode_string_value(encoded_value: &str) -> serde_json::Value {
    // Example: "5:hello" -> "hello"
    let colon_index = encoded_value.find(':').unwrap();
    let number_string = &encoded_value[..colon_index];
    let number = number_string.parse::<i64>().unwrap();
    let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    return serde_json::Value::String(string.to_string());
}

fn decode_integer_value(encoded_value: &str) -> serde_json::Value {
    let i = &encoded_value[1..encoded_value.len() -  1];

    let n: Number;
    if i.chars().next().unwrap() == '-' {
       n = Number::from(i.parse::<i64>().unwrap());
    } else {
       n = Number::from(i.parse::<u64>().unwrap());
    }

    return serde_json::Value::Number(n);
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    let next_char = encoded_value.chars().next().unwrap();
    match next_char {
        n if n.is_digit(10) => decode_string_value(encoded_value),
        'i' => decode_integer_value(encoded_value),
        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
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
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
