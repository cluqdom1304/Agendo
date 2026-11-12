#[cfg(test)]
mod tauri_conf {
    use serde_json::Value;

    fn load_conf() -> Value {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");
        let raw = std::fs::read_to_string(path).expect("tauri.conf.json must be readable");
        serde_json::from_str(&raw).expect("tauri.conf.json must be valid JSON")
    }

    fn main_window(conf: &Value) -> Value {
        conf["app"]["windows"]
            .as_array()
            .and_then(|windows| windows.iter().find(|w| w["label"] == "main"))
            .cloned()
            .expect("a window labelled `main` must exist")
    }

    #[test]
    fn main_window_is_frameless_and_transparent() {
        let window = main_window(&load_conf());
        assert_eq!(window["decorations"], false);
        assert_eq!(window["transparent"], true);
        assert_eq!(window["shadow"], true);
    }

    #[test]
    fn main_window_has_spec_sizes() {
        let window = main_window(&load_conf());
        assert_eq!(window["width"], 1100);
        assert_eq!(window["height"], 720);
        assert_eq!(window["minWidth"], 720);
        assert_eq!(window["minHeight"], 480);
    }

    #[test]
    fn identifier_is_stable_and_non_default() {
        let conf = load_conf();
        let identifier = conf["identifier"]
            .as_str()
            .expect("identifier must be a string");
        assert!(!identifier.is_empty());
        assert_ne!(identifier, "com.tauri.dev");
        assert_eq!(identifier, "com.cristian.agendo");
    }

    #[test]
    fn installers_are_nsis_and_msi_with_metadata_and_a_windows_icon() {
        let conf = load_conf();
        let bundle = &conf["bundle"];
        let targets: Vec<&str> = bundle["targets"]
            .as_array()
            .expect("targets must list the installers")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(targets, ["nsis", "msi"]);
        for field in [
            "publisher",
            "copyright",
            "shortDescription",
            "longDescription",
        ] {
            assert!(
                bundle[field].as_str().is_some_and(|text| !text.is_empty()),
                "bundle.{field} must be set"
            );
        }
        let icons: Vec<&str> = bundle["icon"]
            .as_array()
            .expect("icons")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert!(icons.contains(&"icons/icon.ico"));
        for icon in icons {
            let path = format!("{}/{icon}", env!("CARGO_MANIFEST_DIR"));
            assert!(std::path::Path::new(&path).is_file(), "missing {icon}");
        }
        assert_eq!(bundle["windows"]["nsis"]["installMode"], "currentUser");
    }
}
