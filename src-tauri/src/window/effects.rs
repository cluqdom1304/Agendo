use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectMode {
    Mica,
    Acrylic,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EffectApplied {
    pub requested: EffectMode,
    pub applied: EffectMode,
}

pub trait EffectBackend {
    fn apply_mica(&self) -> Result<(), String>;
    fn apply_acrylic(&self) -> Result<(), String>;
    fn clear(&self) -> Result<(), String>;
}

/// Applies `requested` and degrades Mica -> Acrylic -> none when the OS rejects an effect.
pub fn apply_with_fallback(
    backend: &impl EffectBackend,
    requested: EffectMode,
) -> Result<EffectApplied, String> {
    let applied = match requested {
        EffectMode::Mica => {
            if backend.apply_mica().is_ok() {
                EffectMode::Mica
            } else if backend.apply_acrylic().is_ok() {
                EffectMode::Acrylic
            } else {
                clear_best_effort(backend)
            }
        }
        EffectMode::Acrylic => {
            if backend.apply_acrylic().is_ok() {
                EffectMode::Acrylic
            } else {
                clear_best_effort(backend)
            }
        }
        EffectMode::None => {
            backend
                .clear()
                .map_err(|error| format!("INTERNAL: {error}"))?;
            EffectMode::None
        }
    };
    Ok(EffectApplied { requested, applied })
}

fn clear_best_effort(backend: &impl EffectBackend) -> EffectMode {
    let _ = backend.clear();
    EffectMode::None
}

#[cfg(windows)]
mod native {
    use super::EffectBackend;

    pub struct NativeBackend<'a, R: tauri::Runtime> {
        window: &'a tauri::WebviewWindow<R>,
    }

    impl<'a, R: tauri::Runtime> NativeBackend<'a, R> {
        pub fn new(window: &'a tauri::WebviewWindow<R>) -> Self {
            Self { window }
        }
    }

    impl<R: tauri::Runtime> EffectBackend for NativeBackend<'_, R> {
        fn apply_mica(&self) -> Result<(), String> {
            window_vibrancy::apply_mica(self.window, None).map_err(|error| error.to_string())
        }

        fn apply_acrylic(&self) -> Result<(), String> {
            window_vibrancy::apply_acrylic(self.window, None).map_err(|error| error.to_string())
        }

        fn clear(&self) -> Result<(), String> {
            let mica = window_vibrancy::clear_mica(self.window);
            let acrylic = window_vibrancy::clear_acrylic(self.window);
            mica.and(acrylic).map_err(|error| error.to_string())
        }
    }
}

#[cfg(not(windows))]
mod native {
    use super::EffectBackend;

    pub struct NativeBackend<'a, R: tauri::Runtime> {
        _window: &'a tauri::WebviewWindow<R>,
    }

    impl<'a, R: tauri::Runtime> NativeBackend<'a, R> {
        pub fn new(window: &'a tauri::WebviewWindow<R>) -> Self {
            Self { _window: window }
        }
    }

    impl<R: tauri::Runtime> EffectBackend for NativeBackend<'_, R> {
        fn apply_mica(&self) -> Result<(), String> {
            Err("native window effects are only available on Windows".to_string())
        }

        fn apply_acrylic(&self) -> Result<(), String> {
            Err("native window effects are only available on Windows".to_string())
        }

        fn clear(&self) -> Result<(), String> {
            Ok(())
        }
    }
}

pub use native::NativeBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeBackend {
        mica_ok: bool,
        acrylic_ok: bool,
        clear_ok: bool,
        calls: RefCell<Vec<&'static str>>,
    }

    impl FakeBackend {
        fn new(mica_ok: bool, acrylic_ok: bool, clear_ok: bool) -> Self {
            Self {
                mica_ok,
                acrylic_ok,
                clear_ok,
                calls: RefCell::new(Vec::new()),
            }
        }

        fn outcome(&self, name: &'static str, ok: bool) -> Result<(), String> {
            self.calls.borrow_mut().push(name);
            if ok {
                Ok(())
            } else {
                Err(format!("{name} rejected"))
            }
        }

        fn calls(&self) -> Vec<&'static str> {
            self.calls.borrow().clone()
        }
    }

    impl EffectBackend for FakeBackend {
        fn apply_mica(&self) -> Result<(), String> {
            self.outcome("mica", self.mica_ok)
        }

        fn apply_acrylic(&self) -> Result<(), String> {
            self.outcome("acrylic", self.acrylic_ok)
        }

        fn clear(&self) -> Result<(), String> {
            self.outcome("clear", self.clear_ok)
        }
    }

    #[test]
    fn mica_is_applied_when_supported() {
        let backend = FakeBackend::new(true, true, true);
        let result = apply_with_fallback(&backend, EffectMode::Mica).expect("must succeed");
        assert_eq!(result.applied, EffectMode::Mica);
        assert_eq!(result.requested, EffectMode::Mica);
        assert_eq!(backend.calls(), vec!["mica"]);
    }

    #[test]
    fn mica_falls_back_to_acrylic() {
        let backend = FakeBackend::new(false, true, true);
        let result = apply_with_fallback(&backend, EffectMode::Mica).expect("must succeed");
        assert_eq!(result.applied, EffectMode::Acrylic);
        assert_eq!(result.requested, EffectMode::Mica);
        assert_eq!(backend.calls(), vec!["mica", "acrylic"]);
    }

    #[test]
    fn mica_and_acrylic_failing_fall_back_to_none_and_clear() {
        let backend = FakeBackend::new(false, false, true);
        let result = apply_with_fallback(&backend, EffectMode::Mica).expect("must succeed");
        assert_eq!(result.applied, EffectMode::None);
        assert_eq!(backend.calls(), vec!["mica", "acrylic", "clear"]);
    }

    #[test]
    fn fallback_to_none_succeeds_even_if_clear_fails() {
        let backend = FakeBackend::new(false, false, false);
        let result = apply_with_fallback(&backend, EffectMode::Mica).expect("must succeed");
        assert_eq!(result.applied, EffectMode::None);
    }

    #[test]
    fn acrylic_request_does_not_try_mica() {
        let backend = FakeBackend::new(true, true, true);
        let result = apply_with_fallback(&backend, EffectMode::Acrylic).expect("must succeed");
        assert_eq!(result.applied, EffectMode::Acrylic);
        assert_eq!(backend.calls(), vec!["acrylic"]);
    }

    #[test]
    fn acrylic_request_falls_back_to_none() {
        let backend = FakeBackend::new(true, false, true);
        let result = apply_with_fallback(&backend, EffectMode::Acrylic).expect("must succeed");
        assert_eq!(result.applied, EffectMode::None);
        assert_eq!(backend.calls(), vec!["acrylic", "clear"]);
    }

    #[test]
    fn none_request_only_clears() {
        let backend = FakeBackend::new(true, true, true);
        let result = apply_with_fallback(&backend, EffectMode::None).expect("must succeed");
        assert_eq!(result.applied, EffectMode::None);
        assert_eq!(backend.calls(), vec!["clear"]);
    }

    #[test]
    fn none_request_reports_clear_failure_with_internal_code() {
        let backend = FakeBackend::new(true, true, false);
        let error = apply_with_fallback(&backend, EffectMode::None).expect_err("must fail");
        assert!(error.starts_with("INTERNAL: "), "unexpected error: {error}");
    }

    #[test]
    fn modes_serialize_in_lowercase() {
        let json = serde_json::to_string(&EffectMode::Acrylic).expect("serializes");
        assert_eq!(json, "\"acrylic\"");
        let mode: EffectMode = serde_json::from_str("\"none\"").expect("deserializes");
        assert_eq!(mode, EffectMode::None);
        assert!(serde_json::from_str::<EffectMode>("\"blur\"").is_err());
    }
}
