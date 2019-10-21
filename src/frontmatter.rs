use std::collections::HashMap;

pub struct MarkdownAttrs {
    pub title: String,
    pub author: String,
    pub avatar: String,
    pub published: String,
}

pub fn parse(mdtext: &str) -> MarkdownAttrs {
    let mut frontmatter_flag = false;
    let mut frontmatter_fields = HashMap::new();
    frontmatter_fields.insert("title".to_string(), "".to_string());
    frontmatter_fields.insert("author".to_string(), "".to_string());
    frontmatter_fields.insert("avatar".to_string(), "".to_string());
    frontmatter_fields.insert("published".to_string(), "".to_string());

    for (index, line) in mdtext.lines().enumerate() {
        if index == 0 && line.trim() == "---" {
            frontmatter_flag = true;
        }
        if index > 0 && line.trim() == "---" && frontmatter_flag {
            break;
        }

        if frontmatter_flag && index > 0 {
            let mut s = String::from(line);
            let offset = s.find(':').unwrap_or_else(|| s.len());
            let key: String = s.drain(..offset).collect();
            let val: String = s[1..].trim().to_string();
            frontmatter_fields.insert(key, val);
        }
    }
    MarkdownAttrs {
        title: frontmatter_fields.get("title").unwrap().to_string(),
        author: frontmatter_fields.get("author").unwrap().to_string(),
        avatar: frontmatter_fields.get("avatar").unwrap().to_string(),
        published: frontmatter_fields.get("published").unwrap().to_string(),
    }
}
