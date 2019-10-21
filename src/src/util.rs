use std::collections::HashMap;
use std::env;
use std::process;

use dotenv::dotenv;

pub fn get_topics() -> HashMap<String, String> {
    dotenv().ok();
    let topic_str = env::var("TOPIC").expect("TOPIC must be set");
    let topics: Vec<&str> = topic_str.split(' ').collect();
    let mut res = HashMap::new();
    for item in topics {
        let pair: Vec<&str> = item.split(';').collect();
        if pair.len() != 2 {
            let msg = format!(
                "got invalid TOPIC value from environment variable, TOPIC = {}",
                topic_str
            );
            println!("{}", msg);
            process::exit(-1);
        }
        res.insert(String::from(pair[0]), String::from(pair[1]));
    }
    res
}
