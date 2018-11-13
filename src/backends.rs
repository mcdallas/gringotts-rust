extern crate serde;
use self::serde::{Serialize, Deserialize, Serializer};
use serde_json::{Value, to_string, from_str};
use std::process::Command;
use std::str::from_utf8;
use std::time::Instant;
use std::time::Duration;
use std::thread::sleep;

pub trait MessageBroker {
    fn exists() -> bool;

    fn send<T: Serialize>(message: T, recipient: &str) -> bool;

    fn receive(sender: &str) -> Vec<String>;

    fn poll(nseconds: u64, sender: &str) ->  Option<Value>;

}

pub struct Keybase;

impl MessageBroker for Keybase {

    fn exists() -> bool {
        let mut proc = if cfg!(target_os = "windows") {
            Command::new("where")
        }
        else {
            Command::new("which")
        };
        proc.arg("keybase").status().is_ok()
    }

    fn send<T: Serialize>(message: T, recipient: &str) -> bool {
        let mut proc = Command::new("keybase");
        let msg = to_string(&message).expect("Serialization error");
        proc.args(&["chat", "send", "--exploding-lifetime", "60s", "--topic-type", "dev", recipient, &msg ]);
        let _ = proc.output().unwrap().stdout;
        proc.status().is_ok()
    }

    fn receive(sender: &str) -> Vec<String> {
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
        let response :Value = from_str(from_utf8(&output).expect("Bad output")).unwrap();

        let mut unread :Vec<String> = Vec::new();
        for msg in response["result"]["messages"].as_array().unwrap().iter() {

            if (msg["msg"]["content"]["type"] == "text") && (msg["msg"]["unread"] == true) {
                let message = msg["msg"]["content"]["text"]["body"].as_str().unwrap();
                unread.push(message.to_owned());
            }
        }
        unread
    }

    fn poll(nseconds: u64, sender: &str) -> Option<Value> {
        let start = Instant::now();

        while start.elapsed().as_secs() < nseconds {
            let unread = Keybase::receive(sender);
            for msg in unread.iter() {
                match from_str(msg) {
                    Ok(slate) => return Some(slate),
                    Err(_) => ()
                }
            }
            sleep(Duration::from_millis(1000));
        }
        None
    }
}
