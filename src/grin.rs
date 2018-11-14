extern crate reqwest;
extern crate serde_json;
extern crate serde;

use self::serde::Serialize;
use serde_json::Value;

pub struct ForeignApi{
    pub host: String
}

//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct Slate {
//	/// The number of participants intended to take part in this transaction
//	pub num_participants: usize,
//	/// Unique transaction ID, selected by sender
//	pub id: Uuid,
//	/// The core transaction data:
//	/// inputs, outputs, kernels, kernel offset
//	pub tx: Transaction,
//	/// base amount (excluding fee)
//	pub amount: u64,
//	/// fee amount
//	pub fee: u64,
//	/// Block height for the transaction
//	pub height: u64,
//	/// Lock height
//	pub lock_height: u64,
//	/// Participant data, each participant in the transaction will
//	/// insert their public data here. For now, 0 is sender and 1
//	/// is receiver, though this will change for multi-party
//	pub participant_data: Vec<String>,
//}


//}
impl ForeignApi{
//    pub fn receive_tx<T: Serialize>(self, slate: T) {
//
//        let payload = json!({
//      "name": "John Doe",
//      "age": 43,
//      "phones": [
//        "+44 1234567",
//        "+44 2345678"
//      ]
//    });
//        let client: reqwest::Client = reqwest::Client::new();
//        let mut res = client.post(&self.host)
//        .json(&payload)
//        .send().unwrap();
////        let json = res.json();
////        let json: Ip = reqwest::get("http://httpbin.org/ip").unwrap().json().unwrap();
//
////        let mut buf = String::new();
////        res.read_to_string(&mut buf)
////        .expect("Failed to read response");
//        let v: Value = serde_json::from_str(&res.text().unwrap()).unwrap();
//        println!("{:?}", v["data"]);
//    }

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

    pub fn create_tx(self, amount: u32) -> Value {
        let payload = json!({
            "amount": amount,
            "minimum_confirmations": 5,
            "method": "file",
            "dest": "",
            "max_outputs": 2,
            "num_change_outputs": 1,
            "selection_strategy_is_use_all": true,
            "fluff": false
        });
        self.request("create_tx", &payload)
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