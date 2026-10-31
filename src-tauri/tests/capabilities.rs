#[cfg(test)]
mod capabilities {
    use serde_json::Value;

    fn load_capability() -> Value {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities/default.json");
        let raw = std::fs::read_to_string(path).expect("default.json must be readable");
        serde_json::from_str(&raw).expect("default.json must be valid JSON")
    }

    fn permissions(capability: &Value) -> Vec<Value> {
        capability["permissions"]
            .as_array()
            .cloned()
            .expect("permissions must be an array")
    }

    fn identifier_of(permission: &Value) -> String {
        match permission {
            Value::String(id) => id.clone(),
            other => other["identifier"]
                .as_str()
                .expect("scoped permission must have an identifier")
                .to_string(),
        }
    }

    #[test]
    fn capability_targets_only_main_window() {
        let capability = load_capability();
        assert_eq!(capability["windows"], serde_json::json!(["main"]));
    }

    #[test]
    fn permissions_are_exactly_the_minimal_set() {
        let mut actual: Vec<String> = permissions(&load_capability())
            .iter()
            .map(identifier_of)
            .collect();
        actual.sort();
        let mut expected = vec![
            "core:window:allow-close",
            "core:window:allow-minimize",
            "core:window:allow-set-effects",
            "core:window:allow-start-dragging",
            "core:window:allow-toggle-maximize",
            "opener:allow-open-url",
        ];
        expected.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn no_broad_default_permission_sets() {
        for permission in permissions(&load_capability()) {
            let id = identifier_of(&permission);
            assert!(
                !id.ends_with(":default"),
                "broad permission set not allowed: {id}"
            );
        }
    }

    #[test]
    fn opener_only_allows_http_https_and_mailto() {
        let capability = load_capability();
        let opener = permissions(&capability)
            .into_iter()
            .find(|p| identifier_of(p) == "opener:allow-open-url")
            .expect("opener:allow-open-url must be present");
        let mut urls: Vec<String> = opener["allow"]
            .as_array()
            .expect("opener scope must list allowed urls")
            .iter()
            .map(|entry| entry["url"].as_str().unwrap_or_default().to_string())
            .collect();
        urls.sort();
        assert_eq!(urls, vec!["http://*", "https://*", "mailto:*"]);
    }
}
