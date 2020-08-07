use crate::impl2001_rs::pip::pip2001::Pip2001;
use crate::impl2001_rs::pip::pip2001::Pip2001MessageType;
use crate::impl2001_rs::pip::Pip;
use anyhow::{anyhow, Result};
use curl::easy::{Easy, List};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

use super::SETTINGS;
use crate::settings;
use crate::url::URL;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainInfo {
    pub errors: Option<String>,
    pub success: bool,
    pub head_block_num: i64,
    pub last_irreversible_block_num: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pip2001ActionData {
    pub id: String,
    pub data: String, // json.dumps string
    pub hash: String,
    pub meta: String, // json.dumps string
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(skip)]
    pub caller: String,
    pub signature: String,
    pub user_address: String,
    #[serde(skip)]
    pub unpacked_data: Value,
    #[serde(skip)]
    pub unpacked_meta: Value,
}

impl Pip2001ActionData {
    pub fn to_post_json_str(&self) -> Result<String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let meta: Value = serde_json::from_str(&self.meta)?;
        let inner_data: Value = serde_json::from_str(&self.data)?;

        if !inner_data["file_hash"].is_null() {
            if let Value::String(_v) = &inner_data["file_hash"] {
                result.insert(String::from("file_hash"), _v.clone());
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
                result.insert(String::from("uris"), serde_json::to_string(_v)?);
            }
        }

        if !meta["hash_alg"].is_null() {
            if let Value::String(_v) = &meta["hash_alg"] {
                result.insert(String::from("hash_alg"), _v.clone());
            }
        }
        // the default value is `keccak256`
        result
            .entry(String::from("hash_alg"))
            .or_insert(String::from("keccak256"));

        Ok(serde_json::to_string(&result)?)
    }

    pub fn get_encryption(&self) -> String {
        let v: Value = serde_json::from_str(&self.meta).expect("parse meta failed");
        if !v["encryption"].is_null() {
            if let Value::String(_v) = &v["encryption"] {
                return _v.clone();
            }
        }
        String::from("")
    }

    pub fn get_hash_alg(&self) -> Result<String> {
        let v: Value = serde_json::from_str(&self.meta)?;
        let hash_alg = match v["hash_alg"].as_str() {
            None => "".to_string(),
            Some(v) => v.to_string(),
        };
        Ok(hash_alg)
    }

    pub fn get_file_hash(&self) -> Result<String> {
        let inner_data: Value = serde_json::from_str(&self.data)?;
        let file_hash = match inner_data["file_hash"].as_str() {
            None => "".to_string(),
            Some(v) => v.to_string(),
        };
        Ok(file_hash)
    }

    pub fn get_topic(&self) -> Result<String> {
        let inner_data: Value = serde_json::from_str(&self.data)?;
        let topic = match inner_data["topic"].as_str() {
            None => "".to_string(),
            Some(v) => v.to_string(),
        };
        Ok(topic)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub block_num: i64,
    pub data_type: String,
    pub data: Pip2001ActionData,
    pub trx_id: String,
    pub signature: String,
    pub hash: String,
    pub user_address: String,
}

impl Transaction {
    pub fn get_topic(&self) -> String {
        if let Ok(topic) = self.data.get_topic() {
            return topic;
        }

        "".to_string()
    }

    pub fn has_invalid_topic(&self) -> bool {
        settings::Settings::contains_topic(&SETTINGS, &self.get_topic())
    }

    pub fn get_notify_payload(&self) -> Result<Option<NotifyPayload>> {
        let mut p: Pip2001 = Pip2001::new();
        let json_post_str = match self.data.to_post_json_str() {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow!(
                    "data_id = {}, data to_post_json_str failed: {}",
                    self.data.id,
                    e
                ));
            }
        };

        let post = p.from_json(&json_post_str);
        match post {
            Ok(Some(pipobject)) => {
                debug!(
                    "get_notify_payloads block_num = {}, msg_type = {:?}",
                    self.block_num, &pipobject.msg_type
                );
                match pipobject.msg_type {
                    Pip2001MessageType::PUBLISH => {
                        let payload = NotifyPayload {
                            block: NotifyBlock {
                                data_id: self.data.id.clone(),
                                block_num: self.block_num,
                                trx_id: self.trx_id.clone(),
                            },
                        };
                        return Ok(Some(payload));
                    }
                    Pip2001MessageType::PUBLISH_MANAGEMENT => return Ok(None),
                    _ => {
                        return Err(anyhow!("unsupport action data = {:?}", &self.data));
                    }
                }
            }
            Ok(None) => {
                return Err(anyhow!(
                    "from_json return None, block_num = {}, action data = {:?}",
                    self.block_num,
                    &self.data
                ));
            }
            Err(e) => {
                return Err(anyhow!(
                    "from_json failed: {}, block_num = {}, action data = {:?}",
                    e,
                    self.block_num,
                    &self.data
                ));
            }
        }
    }
}

pub fn fetch_transactions_by_topic(
    easy: &mut Easy,
    topic: &str,
    block_num: i64,
    count: usize,
) -> Result<Vec<Transaction>> {
    let url_suffix = format!(
        "/transactions?topic={}&blocknum={}&type=PIP:2001&count={}",
        &topic, block_num, count
    );
    let url = URL::new().get_url(&url_suffix);

    debug!("access url = {}", url);
    easy.url(&url)?;
    let mut response_content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    };

    let body: Value = serde_json::from_slice(&response_content)?;
    let mut transactions: Vec<Transaction> = vec![];

    if let Value::Array(data_lst) = &body["data"] {
        for data in data_lst {
            let block_num: i64 = match data["block_num"].as_str() {
                None => {
                    error!("can not get block_num, data = {:?}", data);
                    continue;
                }
                Some(v) => v.to_string().parse()?,
            };
            let data_type = match data["transactions_trx_transaction_actions_data_type"].as_str() {
                None => {
                    error!(
                        "can not get transactions_trx_transaction_actions_data_type, data = {:?}",
                        data
                    );
                    continue;
                }
                Some(v) => v.to_string(),
            };
            let trx_id = match data["transactions_trx_id"].as_str() {
                None => {
                    error!("can not get transactions_trx_id, data = {:?}", data);
                    continue;
                }
                Some(v) => v.to_string(),
            };
            let user_address = match data["transactions_trx_transaction_actions_data_user_address"]
                .as_str()
            {
                None => {
                    error!("can not get transactions_trx_transaction_actions_data_user_address, data = {:?}", data);
                    continue;
                }
                Some(v) => v.to_string(),
            };

            if let Value::Array(_transactions) = &data["block"]["transactions"] {
                for _transaction in _transactions {
                    if let Value::Array(_actions) = &_transaction["trx"]["transaction"]["actions"] {
                        for _action in _actions {
                            let action_data = _action["data"].clone();
                            let adata: Pip2001ActionData = match serde_json::from_value(
                                action_data.clone(),
                            ) {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("parse action data to Pip2001ActionData failed, action_data = {:?}, error = {}", &action_data, e);
                                    continue;
                                }
                            };

                            let hash = adata.hash.clone();
                            let signature = adata.signature.clone();

                            let trx = Transaction {
                                block_num,
                                data_type: data_type.clone(),
                                data: adata.clone(),
                                trx_id: trx_id.clone(),
                                hash,
                                signature,
                                user_address: user_address.clone(),
                            };
                            transactions.push(trx);
                        }
                    }
                }
            }
        }
    };

    Ok(transactions)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyPayload {
    pub block: NotifyBlock,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyBlock {
    #[serde(rename = "id")]
    pub data_id: String,
    #[serde(rename = "blockNum")]
    pub block_num: i64,
    #[serde(rename = "blockTransactionId")]
    pub trx_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncPost {
    pub session: String,
    pub content: String,
}

pub fn get_curl_easy() -> Result<Easy> {
    // keep alive
    let timeout = 60;
    let mut easy = Easy::new();
    easy.connect_timeout(Duration::from_secs(timeout))?;
    easy.timeout(Duration::from_secs(timeout))?;
    easy.accept_encoding("gzip")?;

    Ok(easy)
}

pub fn get_start_block_num_by_topic(topic: &str) -> Result<u64> {
    let url_suffix = format!("/transactions?topic={}&type=PIP:2001&count=1", topic);
    let url = URL::new().get_url(&url_suffix);
    debug!("access url: {}", url);

    let mut easy = get_curl_easy()?;
    easy.url(&url)?;
    let mut response_content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    };

    let res: Value = serde_json::from_slice(&response_content)?;

    let block_num: u64 = match res["data"][0]["block_num"].as_str() {
        None => 0, // try to get from last_irreversible_block_num
        Some(v) => v.parse()?,
    };

    if block_num > 0 {
        return Ok(block_num);
    } else {
        return Err(anyhow!("get_start_block_num for topic: {} failed", topic));
    }
}

pub fn get_info(easy: &mut Easy) -> Result<ChainInfo> {
    let url = URL::new().get_base_url();
    debug!("get chain url = {}", url);
    easy.url(&url)?;
    let mut response_content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    };

    let res: Value = serde_json::from_slice(&response_content)?;

    // check if response success
    if let Value::Bool(success) = &res["success"] {
        if !success {
            let body = String::from_utf8_lossy(&response_content);
            return Err(anyhow!("invalid body = {}", body));
        }
        let errors: Option<String>;
        let mut head_block_num: i64 = 0;
        let mut last_irreversible_block_num: i64 = 0;

        if let Value::String(_v) = &res["errors"] {
            errors = Some(_v.clone());
        } else {
            errors = None;
        }

        if !res["data"].is_null() {
            let data = &res["data"];
            if let Value::Number(_v) = &data["head_block_num"] {
                if let Some(_vv) = _v.as_i64() {
                    head_block_num = _vv;
                }
            }
            if let Value::Number(_v) = &data["last_irreversible_block_num"] {
                if let Some(_vv) = _v.as_i64() {
                    last_irreversible_block_num = _vv;
                }
            }

            if head_block_num == 0 || last_irreversible_block_num == 0 {
                return Err(anyhow!(
                    "invalid block num, head_block_num = {} last_irreversible_block_num = {}",
                    head_block_num,
                    last_irreversible_block_num
                ));
            }

            let info = ChainInfo {
                errors,
                success: *success,
                head_block_num,
                last_irreversible_block_num,
            };
            return Ok(info);
        }
    }
    Err(anyhow!("error body: {:?}", &response_content))
}

pub fn notify_webhook(payload: &NotifyPayload, url: &str) -> Result<u32> {
    debug!("notify webhook url = {}", url);
    let mut easy = get_curl_easy().expect("get curl easy failed");
    easy.url(&url)
        .expect(&format!("easy.url failed, url = {}", url));
    let mut headers = List::new();
    headers.append("Content-Type: application/json")?;
    let err_msg = format!("easy.http_headers failed, headers = {:?}", &headers);
    easy.http_headers(headers).expect(&err_msg);
    easy.post(true)?;
    let payload = serde_json::to_string(&payload).expect(&format!(
        "serde_json::to_string failed, payload = {:?}",
        payload
    ));
    debug!(
        "curl -X POST -H 'Content-Type: application/json' -d '{}' {}",
        payload, url
    );
    let mut payload_bytes = payload.as_bytes();
    easy.post_field_size(payload_bytes.len() as u64)?;
    let mut response_content = Vec::new();

    {
        let mut transfer = easy.transfer();
        transfer.read_function(|buf| Ok(payload_bytes.read(buf).unwrap_or(0)))?;
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    let status_code = easy.response_code()?;
    Ok(status_code)
}
