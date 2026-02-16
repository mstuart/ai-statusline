use claudeline::config::Config;

#[test]
fn default_config_has_sensible_values() {
    let config = Config::default();
    assert_eq!(config.lines.len(), 1);
    assert_eq!(config.lines[0].len(), 4);
    assert_eq!(config.theme, "default");
    assert_eq!(config.color_level, "auto");
    assert_eq!(config.default_padding, " ");
    assert_eq!(config.flex_mode, "full-minus-40");
    assert_eq!(config.compact_threshold, 60);
    assert!(!config.global_bold);
    assert!(!config.inherit_separator_colors);
    assert_eq!(config.default_separator, " | ");
}

#[test]
fn default_config_widget_types() {
    let config = Config::default();
    let types: Vec<&str> = config.lines[0]
        .iter()
        .map(|w| w.widget_type.as_str())
        .collect();
    assert_eq!(
        types,
        vec![
            "model",
            "context-percentage",
            "session-cost",
            "session-duration"
        ]
    );
}

#[test]
fn toml_roundtrip() {
    let original = Config::default();
    let toml_str = original.to_toml();
    assert!(!toml_str.is_empty());
    let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize TOML");
    assert_eq!(deserialized.theme, original.theme);
    assert_eq!(deserialized.color_level, original.color_level);
    assert_eq!(deserialized.default_padding, original.default_padding);
    assert_eq!(deserialized.flex_mode, original.flex_mode);
    assert_eq!(deserialized.compact_threshold, original.compact_threshold);
    assert_eq!(deserialized.global_bold, original.global_bold);
    assert_eq!(deserialized.lines.len(), original.lines.len());
    assert_eq!(deserialized.lines[0].len(), original.lines[0].len());
}

#[test]
fn loading_nonexistent_config_returns_default() {
    let config = Config::load(Some("/nonexistent/path/to/config.toml"));
    assert_eq!(config.theme, "default");
    assert_eq!(config.lines.len(), 1);
    assert_eq!(config.lines[0].len(), 4);
}

#[test]
fn config_powerline_defaults() {
    let config = Config::default();
    assert!(!config.powerline.enabled);
    assert_eq!(config.powerline.separator, "\u{E0B0}");
    assert!(!config.powerline.separator_invert_background);
    assert!(config.powerline.start_cap.is_none());
    assert!(config.powerline.end_cap.is_none());
    assert!(!config.powerline.auto_align);
}

#[test]
fn config_from_toml_with_custom_theme() {
    // Build custom config programmatically (lines is Vec<Vec<LineWidgetConfig>>,
    // so direct TOML [[lines]] won't map correctly). Verify via roundtrip instead.
    let mut config = Config::default();
    config.theme = "solarized".into();
    config.color_level = "truecolor".into();
    config.global_bold = true;
    config.compact_threshold = 80;
    config.default_separator = " :: ".into();
    config.powerline.enabled = true;
    config.powerline.auto_align = true;

    let serialized = config.to_toml();
    let deserialized: Config =
        toml::from_str(&serialized).expect("Failed to parse roundtripped TOML");
    assert_eq!(deserialized.theme, "solarized");
    assert_eq!(deserialized.color_level, "truecolor");
    assert!(deserialized.global_bold);
    assert_eq!(deserialized.compact_threshold, 80);
    assert_eq!(deserialized.default_separator, " :: ");
    assert!(deserialized.powerline.enabled);
    assert!(deserialized.powerline.auto_align);
}

#[test]
fn config_to_widget_config_conversion() {
    let config = Config::default();
    let lwc = &config.lines[0][0]; // model widget
    let wc = Config::to_widget_config(lwc);
    assert_eq!(wc.widget_type, "model");
    assert_eq!(wc.color, Some("cyan".into()));
    assert!(!wc.raw_value);
    assert!(!wc.merge_next);
}

#[test]
fn config_to_toml_is_valid() {
    let config = Config::default();
    let toml_str = config.to_toml();
    // Verify it contains expected sections
    assert!(toml_str.contains("theme"));
    assert!(toml_str.contains("default"));
    assert!(toml_str.contains("model"));
}
