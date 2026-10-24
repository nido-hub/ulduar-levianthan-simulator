use std::{collections::HashSet, path::PathBuf};

use bevy::prelude::*;
use leviathan_driving::DriveConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub driving: DrivingSettings,
    pub camera: CameraSettings,
    pub bindings: Bindings,
    pub video: VideoSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DrivingSettings {
    pub speed: f32,
    pub reverse_factor: f32,
    pub strafe_factor: f32,
    pub turn_degrees: f32,
    pub response_time: f32,
}
impl Default for DrivingSettings {
    fn default() -> Self {
        Self {
            speed: 9.0,
            reverse_factor: 0.55,
            strafe_factor: 0.8,
            turn_degrees: 143.0,
            response_time: 0.0,
        }
    }
}
impl DrivingSettings {
    pub fn physics(&self) -> DriveConfig {
        DriveConfig {
            speed: self.speed,
            reverse_factor: self.reverse_factor,
            strafe_factor: self.strafe_factor,
            turn_rate: self.turn_degrees.to_radians(),
            response_time: self.response_time,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CameraSettings {
    pub sensitivity: f32,
    pub invert_y: bool,
    pub auto_center: bool,
    pub distance: f32,
    pub pitch_degrees: f32,
    pub fov_degrees: f32,
    pub follow_response: f32,
}
impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.003,
            invert_y: false,
            auto_center: true,
            distance: 19.0,
            pitch_degrees: 28.0,
            fov_degrees: 60.0,
            follow_response: 20.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Bindings {
    pub forward: String,
    pub backward: String,
    pub turn_left: String,
    pub turn_right: String,
    pub strafe_left: String,
    pub strafe_right: String,
}
impl Default for Bindings {
    fn default() -> Self {
        Self {
            forward: "W".into(),
            backward: "S".into(),
            turn_left: "A".into(),
            turn_right: "D".into(),
            strafe_left: "Q".into(),
            strafe_right: "E".into(),
        }
    }
}
impl Bindings {
    pub fn resolve(&self) -> Option<[KeyCode; 6]> {
        let values = [
            &self.forward,
            &self.backward,
            &self.turn_left,
            &self.turn_right,
            &self.strafe_left,
            &self.strafe_right,
        ];
        let parsed: Option<Vec<_>> = values.iter().map(|value| parse_key(value)).collect();
        parsed?.try_into().ok()
    }
}
pub fn parse_key(value: &str) -> Option<KeyCode> {
    Some(match value.trim().to_ascii_uppercase().as_str() {
        "A" => KeyCode::KeyA,
        "B" => KeyCode::KeyB,
        "C" => KeyCode::KeyC,
        "D" => KeyCode::KeyD,
        "E" => KeyCode::KeyE,
        "F" => KeyCode::KeyF,
        "G" => KeyCode::KeyG,
        "H" => KeyCode::KeyH,
        "I" => KeyCode::KeyI,
        "J" => KeyCode::KeyJ,
        "K" => KeyCode::KeyK,
        "L" => KeyCode::KeyL,
        "M" => KeyCode::KeyM,
        "N" => KeyCode::KeyN,
        "O" => KeyCode::KeyO,
        "P" => KeyCode::KeyP,
        "Q" => KeyCode::KeyQ,
        "R" => KeyCode::KeyR,
        "S" => KeyCode::KeyS,
        "T" => KeyCode::KeyT,
        "U" => KeyCode::KeyU,
        "V" => KeyCode::KeyV,
        "W" => KeyCode::KeyW,
        "X" => KeyCode::KeyX,
        "Y" => KeyCode::KeyY,
        "Z" => KeyCode::KeyZ,
        "UP" => KeyCode::ArrowUp,
        "DOWN" => KeyCode::ArrowDown,
        "LEFT" => KeyCode::ArrowLeft,
        "RIGHT" => KeyCode::ArrowRight,
        "SPACE" => KeyCode::Space,
        _ => return None,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VideoSettings {
    pub width: u32,
    pub height: u32,
    pub shadows: bool,
    pub vsync: bool,
}
impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            width: 1440,
            height: 900,
            shadows: true,
            vsync: true,
        }
    }
}

fn bounded(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}
impl Settings {
    pub fn sanitize(&mut self) -> bool {
        let original = toml::to_string(self).unwrap_or_default();
        self.driving.speed = bounded(self.driving.speed, 3.0, 18.0, 9.0);
        self.driving.reverse_factor = bounded(self.driving.reverse_factor, 0.2, 1.0, 0.55);
        self.driving.strafe_factor = bounded(self.driving.strafe_factor, 0.2, 1.0, 0.8);
        self.driving.turn_degrees = bounded(self.driving.turn_degrees, 45.0, 360.0, 143.0);
        self.driving.response_time = bounded(self.driving.response_time, 0.0, 0.25, 0.0);
        self.camera.sensitivity = bounded(self.camera.sensitivity, 0.0004, 0.015, 0.003);
        self.camera.distance = bounded(self.camera.distance, 7.0, 38.0, 19.0);
        self.camera.pitch_degrees = bounded(self.camera.pitch_degrees, 5.0, 75.0, 28.0);
        self.camera.fov_degrees = bounded(self.camera.fov_degrees, 45.0, 90.0, 60.0);
        self.camera.follow_response = bounded(self.camera.follow_response, 5.0, 50.0, 20.0);
        self.video.width = self.video.width.clamp(960, 2560);
        self.video.height = self.video.height.clamp(640, 1600);
        if !self.bindings.resolve().is_some_and(|keys| {
            !keys.contains(&KeyCode::KeyR) && keys.into_iter().collect::<HashSet<_>>().len() == 6
        }) {
            self.bindings = Bindings::default();
        }
        original != toml::to_string(self).unwrap_or_default()
    }
}

#[derive(Resource)]
pub struct Configuration {
    pub settings: Settings,
    pub path: PathBuf,
    pub notice: String,
}
impl Configuration {
    pub fn load() -> Self {
        let beside_exe = std::env::current_exe()
            .unwrap_or_default()
            .with_file_name("settings.toml");
        let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../settings.toml");
        let path = if beside_exe.is_file() || !development.is_file() {
            beside_exe
        } else {
            development
        };
        let mut notice = "Reglages prets. Rien n'est ecrit avant Enregistrer.".to_string();
        let mut settings = match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str::<Settings>(&text) {
                Ok(settings) => settings,
                Err(error) => {
                    notice = format!(
                        "Configuration invalide : {error}. Valeurs par defaut, fichier conserve."
                    );
                    Settings::default()
                }
            },
            Err(_) => {
                notice = "Configuration absente : valeurs par defaut.".into();
                Settings::default()
            }
        };
        if settings.sanitize() {
            notice = "Certains reglages hors limites ont ete corriges en memoire.".into();
        }
        Self {
            settings,
            path,
            notice,
        }
    }
    pub fn save(&mut self) {
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let text = toml::to_string_pretty(&self.settings)?;
            let temporary = self.path.with_extension("toml.tmp");
            std::fs::write(&temporary, text)?;
            std::fs::rename(temporary, &self.path)?;
            Ok(())
        })();
        self.notice = match result {
            Ok(()) => "Reglages enregistres dans settings.toml.".into(),
            Err(error) => format!("Enregistrement impossible : {error}"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipped_config_parses() {
        let mut config: Settings = toml::from_str(include_str!("../../../settings.toml")).unwrap();
        assert!(!config.sanitize());
    }
    #[test]
    fn omitted_fields_use_defaults() {
        let settings: Settings = toml::from_str("").unwrap();
        assert_eq!(settings.driving.speed, 9.0);
    }
    #[test]
    fn unknown_fields_are_reported() {
        assert!(toml::from_str::<Settings>("speeed = 900").is_err());
    }
    #[test]
    fn nonfinite_and_extreme_settings_are_safe() {
        let mut s = Settings::default();
        s.driving.speed = f32::NAN;
        s.camera.distance = 9999.0;
        s.sanitize();
        assert_eq!(s.driving.speed, 9.0);
        assert_eq!(s.camera.distance, 38.0);
    }
    #[test]
    fn invalid_binding_falls_back_without_panicking() {
        let mut s = Settings::default();
        s.bindings.forward = "bad".into();
        assert!(s.sanitize());
        assert_eq!(s.bindings.forward, "W");
    }
    #[test]
    fn duplicate_bindings_are_rejected() {
        let mut s = Settings::default();
        s.bindings.forward = "E".into();
        s.sanitize();
        assert_eq!(s.bindings.forward, "W");
    }
    #[test]
    fn global_reset_key_cannot_be_bound_to_movement() {
        let mut s = Settings::default();
        s.bindings.forward = "R".into();
        s.sanitize();
        assert_eq!(s.bindings.forward, "W");
    }
    #[test]
    fn supported_keys_are_case_insensitive() {
        assert_eq!(parse_key(" w "), Some(KeyCode::KeyW));
        assert_eq!(parse_key("up"), Some(KeyCode::ArrowUp));
    }
}
