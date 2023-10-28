use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use serde_json::{self, Map};
use std::{env, fs, path::Path, str};

// Available if you need it!
// use serde_bencode;

#[derive(Debug, Serialize, Deserialize)]
struct Torrent {
    announce: String,
    info: Info,
}

impl Torrent {
    pub fn hash_info(&self) -> anyhow::Result<String> {
        use sha1::{Digest, Sha1};

        let info = self.encoded_info()?;
        let mut hasher = Sha1::new();

        hasher.update(&info);

        let result = hasher.finalize();

        Ok(hex::encode(&result[..]))
    }

    fn encoded_info(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_bencode::to_bytes(&self.info)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    name: String,

    #[serde(rename = "piece length")]
    piece_length: usize,

    #[serde(
        deserialize_with = "deserialize_pieces",
        serialize_with = "serialize_pieces"
    )]
    pieces: Vec<u8>,

    #[serde(flatten)]
    keys: Keys,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Keys {
    SingleFile { length: usize },
    MultiFile { files: File },
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

struct DecodeResult<'a>(serde_json::Value, &'a [u8]);

fn deserialize_pieces<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf: ByteBuf = Deserialize::deserialize(deserializer)?;

    Ok(buf.into_vec())
}

fn serialize_pieces<S>(pieces: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(pieces)
}

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

fn parse_torrent(file_path: &Path) -> Result<Torrent, Error> {
    match fs::read(file_path) {
        Ok(data) => {
            let torrent = serde_bencode::from_bytes::<Torrent>(&data).unwrap();

            println!("info hash: {}", torrent.hash_info().unwrap());
            return Ok(torrent);
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
        if let Ok(torrent) = parse_torrent(Path::new(p)) {
            let length = match torrent.info.keys {
                Keys::SingleFile { length } => Some(length),
                _ => None,
            };

            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", length.unwrap());
            println!("Info Hash: {}", torrent.hash_info().unwrap());
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}

// If encode by ourself, we need to implement things like serde_bencode, and decode should not use serde_json, because content may not be valid utf-8, json string value must be a String type(therefore valid utf-8), we need keep bytes


// fn encode_integer_value(decoded_value: i64) -> Vec<u8> {
//     let mut output: Vec<u8> = vec![];
//     output.push(b'i');
//     output.append(&mut decoded_value.to_string().as_bytes().to_vec());
//     output.push(b'e');
//
//     output
// }
//
// fn encode_string_value(decoded_value: &Vec<u8>) -> Vec<u8> {
//     let mut output: Vec<u8> = vec![];
//     output.append(&mut decoded_value.len().to_string().as_bytes().to_vec());
//     output.push(b':');
//     output.append(&mut decoded_value.clone());
//     output
// }
//
// fn encode_list_value(list: serde_json::Value) -> Vec<u8> {
//     let mut output: Vec<u8> = vec![];
//     output.push(b'l');
//
//     for element in list.as_array().unwrap() {
//         output.append(&mut encode_json_value(element.clone()));
//     }
//
//     output.push(b'e');
//     output
// }
//
// fn encode_dictionary_value(decoded_value: serde_json::Value) -> Vec<u8> {
//     let m = decoded_value.as_object().unwrap();
//     let mut output: Vec<u8> = vec![];
//     output.push(b'd');
//
//     for (k, v) in m.into_iter() {
//         output.append(&mut encode_string_value(&k.as_bytes().to_vec()));
//         output.append(&mut encode_json_value(v.clone()));
//     }
//     output.push(b'e');
//     output
// }
//
// fn encode_json_value(decoded_value: serde_json::Value) -> Vec<u8> {
//     match decoded_value {
//         v if v.is_u64() => encode_integer_value(v.as_i64().unwrap()),
//         v if v.is_i64() => encode_integer_value(v.as_i64().unwrap()),
//         v if v.is_string() => encode_string_value(&v.as_str().unwrap().as_bytes().to_owned()),
//         v if v.is_array() => encode_list_value(v),
//         v if v.is_object() => encode_dictionary_value(v),
//         _ => vec!(),
//     }
// }
