extern crate reqwest;
extern crate serde_json;
extern crate serde;

use self::serde::Serialize;
use serde_json::Value;

pub struct ForeignApi{
    pub host: String
}

impl ForeignApi{

    pub fn receive_tx<T: Serialize>(self, slate: T) -> Value {
        let url = format!("http://{}/v1/wallet/foreign/receive_tx", &self.host);
        println!{"Signing slate"};
        let client: reqwest::Client = reqwest::Client::new();
        let mut res = client.post(&url).json(&slate).send().expect("Bad url!");
        let v: Value = serde_json::from_str(&res.text().expect("Bad response")).expect("Invalid json");
        v
    }
}

#[derive(Clone)]
pub struct OwnerApi{
    pub host: String,
    pub username: String,
    pub secret: String
}


impl OwnerApi{
    fn request<T: Serialize>(self, endpoint: &str, payload: T) -> Value {
        let url = format!("http://{}/v1/wallet/owner/{}", &self.host, &endpoint);
        let client = reqwest::Client::new();
        let mut resp = client.post(&url).basic_auth(&self.username, Some(&self.secret)).json(&payload).send().expect("Bad url!");
        let v: Value = serde_json::from_str(&resp.text().expect("Bad response")).expect("T_T");
        v
    }

    pub fn create_tx(self, amount: u64, fluff: bool) -> Value {
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
        self.request("issue_send_tx", &payload)
    }

    pub fn rollback(self, slate_id: &str) {
        println!("Rolling back transaction");
        let endpoint = format!("cancel_tx?tx_id={}", slate_id);
        self.request(&endpoint, "");
    }

    pub fn finalize(self, slate: Value) {
        self.request("finalize_tx", slate);
    }
}