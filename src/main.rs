#[macro_use]
extern crate serde_json;
mod grin;
mod backends;
use backends::MessageBroker;
use std::env::home_dir;

use std::fs;

fn main() {
    if !backends::Keybase::exists() { panic!("Keybase not found in PATH"); }
    receive("192.168.0.5:13415", "mcdallas");
    let data = fs::read_to_string(home_dir().unwrap()).expect("Unable to read file");

}

fn receive(host: &str, sender: &str) {

    let api = grin::ForeignApi{host: host.to_owned()};
    let reply = backends::Keybase::listen(180, sender);
    match reply {
        Some(msg) => {
            let signed = api.receive_tx(msg);
            println!("Returning slate");
            backends::Keybase::send(signed, sender, 60);
            println!("Done");
        },
        None => { println!("Did not receive msg from {}", sender) }
    };
}

fn send(amount:u32, recipient: &str, ttl: u16, host: &str, username: &str, secret: &str, fluff: bool) {
    let api = grin::OwnerApi{host: host.to_owned(), username: username.to_owned(), secret: secret.to_owned()};
    let slate = api.clone().create_tx(amount);
    let cloned = slate.clone();
    let slate_id = cloned["id"].as_str();
    backends::Keybase::send(slate, recipient, ttl);
    match backends::Keybase::listen(ttl as u64, recipient) {
        Some(tx) => {
            println!("Received reply from {}", recipient);
            api.finalize(tx);
            println!("Transaction {} broadcasted", slate_id.unwrap());
        },
        None => {

            api.rollback(slate_id.unwrap())
        }
    }
}