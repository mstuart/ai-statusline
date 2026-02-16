use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

impl Theme {
    pub fn get(name: &str) -> Self {
        match name {
            "solarized" => Self::solarized(),
            "nord" => Self::nord(),
            "dracula" => Self::dracula(),
            "gruvbox" => Self::gruvbox(),
            "monokai" => Self::monokai(),
            _ => Self::default_theme(),
        }
    }

    pub fn list() -> Vec<&'static str> {
        vec![
            "default",
            "solarized",
            "nord",
            "dracula",
            "gruvbox",
            "monokai",
        ]
    }

    pub fn color(&self, role: &str) -> Option<&str> {
        self.colors.get(role).map(|s| s.as_str())
    }

    fn default_theme() -> Self {
        Self {
            name: "default".into(),
            colors: HashMap::from([
                ("model".into(), "cyan".into()),
                ("context_ok".into(), "green".into()),
                ("context_warn".into(), "yellow".into()),
                ("context_critical".into(), "red".into()),
                ("git_branch".into(), "magenta".into()),
                ("git_clean".into(), "green".into()),
                ("git_dirty".into(), "yellow".into()),
                ("cost".into(), "yellow".into()),
                ("duration".into(), "white".into()),
                ("separator_fg".into(), "brightBlack".into()),
            ]),
        }
    }

    fn solarized() -> Self {
        Self {
            name: "solarized".into(),
            colors: HashMap::from([
                ("model".into(), "#268bd2".into()),
                ("context_ok".into(), "#859900".into()),
                ("context_warn".into(), "#b58900".into()),
                ("context_critical".into(), "#dc322f".into()),
                ("git_branch".into(), "#6c71c4".into()),
                ("git_clean".into(), "#859900".into()),
                ("git_dirty".into(), "#cb4b16".into()),
                ("cost".into(), "#b58900".into()),
                ("duration".into(), "#93a1a1".into()),
                ("separator_fg".into(), "#586e75".into()),
            ]),
        }
    }

    fn nord() -> Self {
        Self {
            name: "nord".into(),
            colors: HashMap::from([
                ("model".into(), "#88c0d0".into()),
                ("context_ok".into(), "#a3be8c".into()),
                ("context_warn".into(), "#ebcb8b".into()),
                ("context_critical".into(), "#bf616a".into()),
                ("git_branch".into(), "#b48ead".into()),
                ("git_clean".into(), "#a3be8c".into()),
                ("git_dirty".into(), "#d08770".into()),
                ("cost".into(), "#ebcb8b".into()),
                ("duration".into(), "#d8dee9".into()),
                ("separator_fg".into(), "#4c566a".into()),
            ]),
        }
    }

    fn dracula() -> Self {
        Self {
            name: "dracula".into(),
            colors: HashMap::from([
                ("model".into(), "#8be9fd".into()),
                ("context_ok".into(), "#50fa7b".into()),
                ("context_warn".into(), "#f1fa8c".into()),
                ("context_critical".into(), "#ff5555".into()),
                ("git_branch".into(), "#bd93f9".into()),
                ("git_clean".into(), "#50fa7b".into()),
                ("git_dirty".into(), "#ffb86c".into()),
                ("cost".into(), "#f1fa8c".into()),
                ("duration".into(), "#f8f8f2".into()),
                ("separator_fg".into(), "#6272a4".into()),
            ]),
        }
    }

    fn gruvbox() -> Self {
        Self {
            name: "gruvbox".into(),
            colors: HashMap::from([
                ("model".into(), "#83a598".into()),
                ("context_ok".into(), "#b8bb26".into()),
                ("context_warn".into(), "#fabd2f".into()),
                ("context_critical".into(), "#fb4934".into()),
                ("git_branch".into(), "#d3869b".into()),
                ("git_clean".into(), "#b8bb26".into()),
                ("git_dirty".into(), "#fe8019".into()),
                ("cost".into(), "#fabd2f".into()),
                ("duration".into(), "#ebdbb2".into()),
                ("separator_fg".into(), "#665c54".into()),
            ]),
        }
    }

    fn monokai() -> Self {
        Self {
            name: "monokai".into(),
            colors: HashMap::from([
                ("model".into(), "#66d9ef".into()),
                ("context_ok".into(), "#a6e22e".into()),
                ("context_warn".into(), "#e6db74".into()),
                ("context_critical".into(), "#f92672".into()),
                ("git_branch".into(), "#ae81ff".into()),
                ("git_clean".into(), "#a6e22e".into()),
                ("git_dirty".into(), "#fd971f".into()),
                ("cost".into(), "#e6db74".into()),
                ("duration".into(), "#f8f8f2".into()),
                ("separator_fg".into(), "#75715e".into()),
            ]),
        }
    }
}
