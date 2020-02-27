use anyhow::Result;
use rand::{thread_rng, Rng};
use std::env;

pub struct URL {
    urls: Vec<String>,
}

impl URL {
    pub fn new() -> URL {
        let url = env::var("PRS_BASE_URL").expect("get prs base url");

        URL::from(&url)
    }

    pub fn from(url: &str) -> URL {
        let urls = URL::parse_url(&url, '[', ']').expect("parse prs base url");
        URL { urls }
    }

    fn parse_url(url: &str, left_sep: char, right_sep: char) -> Result<Vec<String>> {
        let mut urls = Vec::new();
        let origin_urls = vec![String::from(url)];

        let start_at = match url.find(left_sep) {
            Some(v) => v,
            None => return Ok(origin_urls),
        };

        let end_at = match url.find(right_sep) {
            Some(v) => v,
            None => return Ok(origin_urls),
        };

        let match_part = match url.get(start_at + 1..end_at) {
            Some(v) => v,
            None => return Ok(origin_urls),
        };

        let parts: Vec<&str> = match_part.split('-').collect();
        if parts.len() != 2 {
            return Ok(origin_urls);
        }

        let start: u32 = parts[0].parse()?;
        let end: u32 = parts[1].parse()?;
        let prefix = match url.get(..start_at) {
            Some(v) => v,
            None => return Ok(origin_urls),
        };
        let suffix = match url.get(end_at + 1..) {
            Some(v) => v,
            None => return Ok(origin_urls),
        };
        for idx in start..end + 1 {
            let new_url = format!("{}{}{}", prefix, idx, suffix);
            urls.push(new_url);
        }

        Ok(urls)
    }

    pub fn get_all_urls(&self) -> Vec<String> {
        self.urls.clone()
    }

    fn get_random_url(&self) -> String {
        let mut rng = thread_rng();
        let urls = self.get_all_urls();
        let high = urls.len() as usize;
        let n: usize = rng.gen_range(0, high);
        urls[n].clone()
    }

    pub fn get_base_url(&self) -> String {
        self.get_random_url()
    }

    pub fn get_url(&self, suffix: &str) -> String {
        let base_url = self.get_base_url();
        format!("{}{}", base_url, String::from(suffix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let key = "PRS_BASE_URL";
        let value = "https://prs-bp[1-2].press.one/api/chain";

        env::set_var(key, value);
        let url = URL::new();

        assert_eq!(
            url.get_all_urls(),
            vec![
                String::from("https://prs-bp1.press.one/api/chain"),
                String::from("https://prs-bp2.press.one/api/chain"),
            ]
        );
    }

    #[test]
    fn single_url() {
        let value = "https://prs-bp1.press.one/api/chain";

        let url = URL::from(value);
        assert_eq!(url.get_all_urls(), vec![String::from(value)]);
    }

    #[test]
    fn mutiple_urls() {
        let value = "https://prs-bp[1-2].press.one/api/chain";

        let url = URL::from(value);
        assert_eq!(
            url.get_all_urls(),
            vec![
                String::from("https://prs-bp1.press.one/api/chain"),
                String::from("https://prs-bp2.press.one/api/chain"),
            ]
        );
    }
}
