#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
mod grin;
mod backends;
use backends::MessageBroker;


use std::process::Command;
fn main() {
    println!("{:?}", backends::Keybase::exists());
//    backends::Keybase::send("test", "mcdallas");
    let reply = backends::Keybase::poll(180,"mcdallas");
    match reply {
        Some(msg) => println!("GOT : {}", msg),
        None => ()
    }
//    println!("{:?}", reply);
    let api = grin::ForeignApi{host: "http://httpbin.org/post".to_owned()};
    api.receive_tx('a');
}
