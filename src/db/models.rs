extern crate prs_utility_rust;
extern crate qs_rs;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use crate::prs_utility_rust::utility;
use crate::qs_rs::qs;

use super::chrono;
use super::eos;
use super::schema::blocks;
use super::schema::contents;
use super::schema::last_status;
use super::schema::notifies;
use super::schema::posts;
use super::schema::transactions;
use super::schema::users;

#[derive(Serialize, Deserialize)]
pub struct UserList(pub Vec<User>);

#[derive(Serialize, Deserialize)]
pub struct BlockList(pub Vec<Block>);

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub user_address: String,
    pub status: String,
    pub tx_id: String,
    pub updated_at: chrono::NaiveDateTime,
    pub topic: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub user_address: &'a str,
    pub status: &'a str,
    pub tx_id: &'a str,
    pub updated_at: chrono::NaiveDateTime,
    pub topic: &'a str,
}

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub publish_tx_id: String,
    pub user_address: String,
    pub file_hash: String,
    pub topic: String,
    pub url: String,
    pub updated_tx_id: String,
    pub updated_at: chrono::NaiveDateTime,
    pub fetched: bool,
    pub verify: bool,
    pub encryption: String,
    pub hash_alg: String,
    pub deleted: bool,
}

#[derive(Queryable, PartialEq, QueryableByName, Debug, Serialize)]
#[table_name = "posts"]
pub struct PostPartial {
    pub publish_tx_id: String,
    pub file_hash: String,
    pub topic: String,
    pub deleted: bool,
}

#[derive(Queryable, PartialEq, QueryableByName, Debug, Serialize)]
#[table_name = "posts"]
pub struct PostJson {
    pub publish_tx_id: String,
    pub file_hash: String,
    pub topic: String,
    pub updated_tx_id: String,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted: bool,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub publish_tx_id: &'a str,
    pub updated_tx_id: &'a str,
    pub user_address: &'a str,
    pub file_hash: &'a str,
    pub topic: &'a str,
    pub url: &'a str,
    pub updated_at: chrono::NaiveDateTime,
    pub encryption: &'a str,
    pub hash_alg: &'a str,
}

#[derive(Queryable)]
pub struct Content {
    pub file_hash: String,
    pub url: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted: bool,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "contents"]
pub struct NewContent<'a> {
    pub file_hash: &'a str,
    pub url: &'a str,
    pub content: &'a str,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Debug)]
pub struct LastStatus {
    pub id: i32,
    pub key: String,
    pub val: i64,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "last_status"]
pub struct NewLastStatus<'a> {
    pub key: &'a str,
    pub val: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BlockType {
    EMPTY,
    DATA,
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Block {
    pub id: i32,
    pub block_id: String,
    pub block_num: i64,
    pub block_type: String, // BlockType
    pub block_timestamp: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "blocks"]
pub struct NewBlock<'a> {
    pub block_id: &'a str,
    pub block_num: i64,
    pub block_type: &'a str,
    pub block_timestamp: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Debug)]
pub struct Trx {
    pub id: i32,
    pub block_num: i64,
    pub data_type: String, // e.g.: "PIP:2001"
    pub data: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub trx_id: String,
    pub signature: String,
    pub hash: String,
    pub user_address: String,
    pub processed: bool,
}

impl Trx {
    pub fn get_file_hash(&self) -> Option<String> {
        let data: eos::Pip2001ActionData =
            serde_json::from_str(&self.data).expect("parse trx data failed");
        let inner_data: Value = serde_json::from_str(&data.data).expect("parse inner data failed");
        if !inner_data["file_hash"].is_null() {
            if let Value::String(_v) = &inner_data["file_hash"] {
                return Some(_v.clone());
            }
        }
        None
    }

    pub fn to_post_json_str(&self) -> String {
        let mut result: HashMap<String, String> = HashMap::new();
        let data: eos::Pip2001ActionData =
            serde_json::from_str(&self.data).expect("parse trx data failed");
        let meta: Value = serde_json::from_str(&data.meta).expect("parse data meta failed");
        let inner_data: Value = serde_json::from_str(&data.data).expect("parse inner data failed");

        if !inner_data["file_hash"].is_null() {
            if let Value::String(_v) = &inner_data["file_hash"] {
                result.insert(String::from("file_hash"), _v.clone());

                // the default value is `keccak256`
                result.insert(String::from("hash_alg"), String::from("keccak256"));
                if !inner_data["alg"].is_null() {
                    if let Value::String(_v) = &inner_data["alg"] {
                        *result.get_mut("hash_alg").unwrap() = _v.clone();
                    }
                }
            }
        }

        if !inner_data["updated_tx_id"].is_null() {
            if let Value::String(_v) = &inner_data["updated_tx_id"] {
                result.insert(String::from("updated_tx_id"), _v.clone());
            }
        }

        if !inner_data["topic"].is_null() {
            if let Value::String(_v) = &inner_data["topic"] {
                result.insert(String::from("topic"), _v.clone());
            }
        }

        if !inner_data["allow"].is_null() {
            if let Value::String(_v) = &inner_data["allow"] {
                result.insert(String::from("allow"), _v.clone());
            }
        }

        if !inner_data["deny"].is_null() {
            if let Value::String(_v) = &inner_data["deny"] {
                result.insert(String::from("deny"), _v.clone());
            }
        }

        if !meta["uris"].is_null() {
            if let Value::Array(_v) = &meta["uris"] {
                result.insert(
                    String::from("uris"),
                    serde_json::to_string(_v).expect("meta.uris to json str failed"),
                );
            }
        }

        serde_json::to_string(&result).expect("json dumps post json failed")
    }

    pub fn verify_signature(&self) -> bool {
        let data: eos::Pip2001ActionData =
            serde_json::from_str(&self.data).expect("parse trx data failed");
        let result = qs::json_to_qs(&data.data);
        match result {
            Ok(_s) => {
                let _s_hash = utility::keccak256(&_s)
                    .ok()
                    .expect(&format!("utility::keccak256 failed, _s = {}", &_s));
                if _s_hash == data.hash {
                    let result = utility::recover_user_pubaddress(&data.signature, &_s_hash);
                    match result {
                        Ok(_r) => _r == self.user_address,
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "transactions"]
pub struct NewTrx<'a> {
    pub block_num: i64,
    pub data_type: &'a str,
    pub data: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub trx_id: &'a str,
    pub signature: &'a str,
    pub hash: &'a str,
    pub user_address: &'a str,
}

#[derive(Queryable, Clone, Debug)]
pub struct Notify {
    pub data_id: String,
    pub block_num: i64,
    pub trx_id: String,
    pub success: bool,
    pub retries: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub topic: String,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "notifies"]
pub struct NewNotify<'a> {
    pub data_id: &'a str,
    pub block_num: i64,
    pub trx_id: &'a str,
    pub topic: &'a str,
}

#[derive(Queryable, PartialEq, QueryableByName, Debug)]
#[table_name = "notifies"]
pub struct NotifyPartial {
    pub data_id: String,
    pub block_num: i64,
    pub trx_id: String,
    pub topic: String,
}
