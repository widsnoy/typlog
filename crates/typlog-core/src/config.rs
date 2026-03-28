use std::fs;
use std::path::Path;

pub fn read_site_title() -> String {
    let path = Path::new("config.toml");
    let Ok(content) = fs::read_to_string(path) else {
        return "Typlog".to_string();
    };
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.starts_with("title") {
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let val = line[eq + 1..].trim();
        if let Some(inner) = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return inner.to_string();
        }
    }
    "Typlog".to_string()
}
