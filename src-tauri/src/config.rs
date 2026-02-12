use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::Serialize;

#[derive(Serialize)]
pub struct Theme {
    pub background: String,
    pub foreground: String,
    pub palette: Vec<String>,
    pub font_family: String,
    pub font_size: f32,
    pub selection_background: String,
}

pub fn load_theme(path: &Path) -> Theme {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut theme = Theme {
        background: "#191724".to_string(),
        foreground: "#e0def4".to_string(),
        palette: vec![],
        font_family: "Monospace".to_string(),
        font_size: 14.0,
        selection_background: "#403d52".to_string(),
    };
    let mut colors = HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let key = parts[0];
        let val = parts[1].to_string();
        match key {
            "background" => theme.background = val,
            "foreground" => theme.foreground = val,
            "selection_background" => theme.selection_background = val,
            "font_family" => theme.font_family = parts[1..].join(" "),
            "font_size" => theme.font_size = val.parse().unwrap_or(14.0),
            k if k.starts_with("color") => { colors.insert(k.to_string(), val); }
            _ => {}
        }
    }

    for i in 0..=15 {
        if let Some(c) = colors.get(&format!("color{}", i)) {
            theme.palette.push(c.clone());
        }
    }
    
    if theme.palette.len() < 10 {
        theme.palette = vec![
            "#9ccfd8".into(), "#c4a7e7".into(), "#ebbcba".into(),
            "#f6c177".into(), "#ea9d34".into(), "#d7827e".into(),
            "#907aa9".into(), "#b4637a".into(), "#88a096".into(),
            "#9bb1d6".into(), "#c2d1b2".into(), "#e8d1c5".into(),
            "#d4b5d8".into(), "#adcbe3".into(), "#e1e1e1".into()
        ];
    }
    theme
}