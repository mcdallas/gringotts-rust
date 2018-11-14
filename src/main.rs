#[macro_use]
extern crate serde_json;
mod grin;
mod backends;
use backends::MessageBroker;
use std::env::home_dir;

use std::fs;

#[macro_use]
extern crate clap;
use clap::App;

#[macro_use]
extern crate serde_derive;

fn read_secret() -> String {
    let mut path = home_dir().unwrap();
    path.push(".grin");
    path.push(".api_secret");
    let data = fs::read_to_string(path).expect("Unable to read file");
    data
}

fn main() {
    if !backends::Keybase::exists() { panic!("Keybase not found in PATH"); }
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("receive") {
        let host = matches.value_of("host").unwrap();
        let sender = matches.value_of("sender").unwrap();
        receive(host, sender);
    }
    if let Some(matches) = matches.subcommand_matches("send") {
        let host = matches.value_of("host").unwrap();
        let recipient = matches.value_of("recipient").unwrap();
        let grins :f64 = matches.value_of("amount").unwrap().parse().unwrap() ;
        let amount = (grins * 100000000.0) as u64 ;
        let ttl :u16 = matches.value_of("ttl").unwrap().parse().unwrap();
        let fluff = matches.is_present("fluff");
        let username = matches.value_of("username").unwrap();
        let secret = if matches.is_present("secret") {
            matches.value_of("secret").unwrap().to_owned()
        } else {
            read_secret()
        };

        send(amount, recipient, ttl, host, username, secret, fluff);
    }


}

fn receive(host: &str, sender: &str) {

    let api = grin::ForeignApi{host: host.to_owned()};
    let reply = backends::Keybase::listen(180, sender);
    match reply {
        Some(msg) => {
            let signed = api.receive_tx(msg).unwrap();
            println!("Returning slate");
            backends::Keybase::send(signed, sender, 60);
            println!("Done");
        },
        None => ()
    };
}

fn send(amount:u64, recipient: &str, ttl: u16, host: &str, username: &str, secret: String, fluff: bool) {
    let api = grin::OwnerApi{host: host.to_owned(), username: username.to_owned(), secret: secret.to_owned()};
    let slate = match api.clone().create_tx(amount, fluff) {
        Ok(val) => val,
        Err(e) => {
            println!("{}", e);
            return
        }
    };
    let cloned = slate.clone();
    let slate_id = cloned["id"].as_str();
    println!("Sending slate to {}", recipient);
    backends::Keybase::send(slate, recipient, ttl);
    match backends::Keybase::listen(ttl as u64, recipient) {
        Some(tx) => {
            match api.clone().finalize(tx) {
                Ok(_) => { println!("Transaction {} broadcasted", slate_id.unwrap()) },
                Err(e) => {
                     println!("{}", e);
                     api.rollback(slate_id.unwrap());
                }
            }
        },
        None => { api.rollback(slate_id.unwrap()); }
    }
}