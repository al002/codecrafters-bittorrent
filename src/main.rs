use serde_json::{self, Number, Value};
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
    let i = &encoded_value[1..encoded_value.len() - 1];

    let n: Number;
    if i.chars().next().unwrap() == '-' {
        n = Number::from(i.parse::<i64>().unwrap());
    } else {
        n = Number::from(i.parse::<u64>().unwrap());
    }

    return serde_json::Value::Number(n);
}

fn decode_list_value(encoded_value: &str) -> serde_json::Value {
    let mut content = &encoded_value[1..encoded_value.len() - 1];

    let mut v = vec![];

    while let Some(next_char) = content.chars().next() {
        if next_char == 'e' {
            break;
        }

        match next_char {
            n if n.is_digit(10) => {
                let decoded = decode_string_value(content);
                v.push(decoded.clone());
                if let Value::String(s) = decoded {
                    content = &content[s.len() + 2..];
                }
            }
            'i' => {
                let decoded = decode_integer_value(content);
                v.push(decoded.clone());

                if let Value::Number(n) = decoded {
                    if n.is_i64() {
                        if let Some(i) = n.as_i64() {
                            content = &content[i.to_string().len() + 2..];

                        }
                    } else if n.is_u64() {
                        if let Some(u) = n.as_i64() {
                            content = &content[u.to_string().len() + 2..];

                        }
                    }
                }
            }
            _ => {
                panic!("Unhandled encoded value: {}", encoded_value);
            }
        }
    }

    return serde_json::Value::Array(v);
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    let next_char = encoded_value.chars().next().unwrap();
    match next_char {
        n if n.is_digit(10) => decode_string_value(encoded_value),
        'i' => decode_integer_value(encoded_value),
        'l' => decode_list_value(encoded_value),
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
