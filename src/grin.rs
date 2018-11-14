extern crate reqwest;
extern crate serde_json;
extern crate serde;

use self::serde::Serialize;
use serde_json::Value;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum Error {
    BadUrl(String),
    BadResponse(String),
    DeserializationError(String)
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::BadUrl(ref b) => write!(f, "Bad URL: {}", b),
			Error::BadResponse(ref url) => write!(f, "Bad Response from {}", url),
			Error::DeserializationError(ref url) => write!(f, "Deserialization error : {}", url),
		}
	}
}

pub struct ForeignApi{
    pub host: String
}

impl ForeignApi{

    pub fn receive_tx<T: Serialize>(self, slate: T) -> Result<Value, Error> {
        let url = format!("http://{}/v1/wallet/foreign/receive_tx", &self.host);
        println!{"Signing slate"};
        let client: reqwest::Client = reqwest::Client::new();
        let mut res = match client.post(&url).json(&slate).send() {
            Ok(r) => r,
            Err(_) => { return Err(Error::BadUrl(url)) }
        };
        let txt = match res.text() {
            Ok(text) => text,
            Err(_) => { return Err(Error::BadResponse(url))}
        };
        let v = match serde_json::from_str(&txt) {
            Ok(json) => json,
            Err(_) => { return Err(Error::DeserializationError(txt)) }
        };
        Ok(v)
    }
}

#[derive(Clone)]
pub struct OwnerApi{
    pub host: String,
    pub username: String,
    pub secret: String
}


impl OwnerApi{
    fn request<T: Serialize>(self, endpoint: &str, payload: T) -> Result<Value, Error> {
        let url = format!("http://{}/v1/wallet/owner/{}", &self.host, &endpoint);
        let client = reqwest::Client::new();

        let mut resp = match client.post(&url).basic_auth(&self.username, Some(&self.secret)).json(&payload).send() {
            Ok(response) => response,
            Err(_) => { return Err(Error::BadUrl(url)) }
        };
        let txt = match resp.text() {
            Ok(text) => text,
            Err(_) => { return Err(Error::BadResponse(url))}
        };
        let v = match serde_json::from_str(&txt) {
            Ok(json) => json,
            Err(_) => { return Err(Error::DeserializationError(txt)) }
        };
        Ok(v)
    }

    pub fn create_tx(self, amount: u64, fluff: bool) -> Result<Value, Error> {
        let payload = json!({
            "amount": amount,
            "minimum_confirmations": 5,
            "method": "file",
            "dest": "",
            "max_outputs": 2,
            "num_change_outputs": 1,
            "selection_strategy_is_use_all": true,
            "fluff": fluff
        });
        println!("Creating new transaction");
        self.request("issue_send_tx", &payload)
    }

    pub fn rollback(self, slate_id: &str) -> Result<Value, Error>{
        println!("Rolling back transaction");
        let endpoint = format!("cancel_tx?tx_id={}", slate_id);
        self.request(&endpoint, "")
    }

    pub fn finalize(self, slate: Value) -> Result<Value, Error> {
        println!("Finalizing transaction");
        self.request("finalize_tx", slate)
    }
}