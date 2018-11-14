extern crate serde;
use self::serde::Serialize;
use serde_json::{from_str, to_string, Value};
use std::process::{Command, Stdio};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

fn is_slate(blob: &Value) -> bool {
    if !blob.is_object() {
        return false;
    }
    blob.get("num_participants").is_some() && blob.get("id").is_some() && blob.get("tx").is_some()
}

pub trait MessageBroker {
    fn exists() -> bool;

    fn send<T: Serialize>(message: T, recipient: &str, ttl: u16) -> bool;

    fn listen(nseconds: u64, sender: &str) -> Option<Value>;
}

pub struct Keybase;

impl Keybase {
    fn get_unread(sender: &str) -> Vec<String> {
        let payload = to_string(&json!({
            "method": "read",
            "params": {
                "options": {
                    "channel": {
                            "name": sender, "topic_type": "dev"
                        }
                    },
                    "unread_only": true, "peek": false
                }
            }
        )).unwrap();
        let mut proc = Command::new("keybase");
        proc.args(&["chat", "api", "-m", &payload]);
        let output = proc.output().expect("No output").stdout;
        let response: Value = from_str(from_utf8(&output).expect("Bad output")).unwrap();

        let mut unread: Vec<String> = Vec::new();
        for msg in response["result"]["messages"].as_array().unwrap().iter() {
            if (msg["msg"]["content"]["type"] == "text") && (msg["msg"]["unread"] == true) {
                let message = msg["msg"]["content"]["text"]["body"].as_str().unwrap();
                unread.push(message.to_owned());
            }
        }
        unread
    }
}

impl MessageBroker for Keybase {
    fn exists() -> bool {
        let mut proc = if cfg!(target_os = "windows") {
            Command::new("where")
        } else {
            Command::new("which")
        };
        proc.arg("keybase").stdout(Stdio::null()).status().is_ok()
    }

    fn send<T: Serialize>(message: T, recipient: &str, ttl: u16) -> bool {
        let seconds = format!("{}s", ttl);
        let mut proc = Command::new("keybase");
        let msg = to_string(&message).expect("Serialization error");
        let args = [
            "chat",
            "send",
            "--exploding-lifetime",
            &seconds,
            "--topic-type",
            "dev",
            recipient,
            &msg,
        ];
        proc.args(&args).stdout(Stdio::null());
        proc.status().is_ok()
    }

    fn listen(nseconds: u64, sender: &str) -> Option<Value> {
        let start = Instant::now();
        println!("Waiting for message from {}...", sender);
        while start.elapsed().as_secs() < nseconds {
            let unread = Keybase::get_unread(sender);
            for msg in unread.iter() {
                match from_str(msg) {
                    Ok(slate) => if is_slate(&slate) {
                        println!("Received message from {}", sender);
                        return Some(slate);
                    },
                    Err(_) => (),
                }
            }
            sleep(Duration::from_millis(1000));
        }
        println!(
            "Did not receive reply from {} in {} seconds",
            sender, nseconds
        );
        None
    }
}
