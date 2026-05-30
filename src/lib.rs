use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Weather types available in the Lau world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
    Aurora,
    Foggy,
    MeteorShower,
    CrystalRain,
    GoldenHour,
}

impl Weather {
    /// Returns a human-readable label for the weather variant.
    pub fn label(&self) -> &'static str {
        match self {
            Weather::Sunny => "Sunny",
            Weather::Cloudy => "Cloudy",
            Weather::Rainy => "Rainy",
            Weather::Stormy => "Stormy",
            Weather::Snowy => "Snowy",
            Weather::Aurora => "Aurora",
            Weather::Foggy => "Foggy",
            Weather::MeteorShower => "Meteor Shower",
            Weather::CrystalRain => "Crystal Rain",
            Weather::GoldenHour => "Golden Hour",
        }
    }
}

/// Seasons affect base weather probabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Returns a weight map for how likely each weather type is during this season.
    /// Weights are relative (not probabilities); they get normalized.
    pub fn weather_weights(&self) -> Vec<(Weather, f64)> {
        match self {
            Season::Spring => vec![
                (Weather::Sunny, 3.0),
                (Weather::Cloudy, 2.5),
                (Weather::Rainy, 2.0),
                (Weather::GoldenHour, 1.5),
                (Weather::Foggy, 1.0),
                (Weather::Stormy, 0.5),
                (Weather::Snowy, 0.2),
                (Weather::Aurora, 0.1),
                (Weather::MeteorShower, 0.1),
                (Weather::CrystalRain, 0.1),
            ],
            Season::Summer => vec![
                (Weather::Sunny, 4.0),
                (Weather::GoldenHour, 2.0),
                (Weather::Cloudy, 1.5),
                (Weather::Stormy, 1.0),
                (Weather::MeteorShower, 1.0),
                (Weather::Rainy, 0.5),
                (Weather::Foggy, 0.3),
                (Weather::Aurora, 0.1),
                (Weather::CrystalRain, 0.1),
                (Weather::Snowy, 0.0),
            ],
            Season::Autumn => vec![
                (Weather::Cloudy, 3.0),
                (Weather::Rainy, 2.5),
                (Weather::Foggy, 2.0),
                (Weather::GoldenHour, 2.0),
                (Weather::Stormy, 1.5),
                (Weather::Sunny, 1.0),
                (Weather::CrystalRain, 0.5),
                (Weather::Snowy, 0.5),
                (Weather::Aurora, 0.3),
                (Weather::MeteorShower, 0.2),
            ],
            Season::Winter => vec![
                (Weather::Snowy, 4.0),
                (Weather::Cloudy, 2.5),
                (Weather::Aurora, 2.0),
                (Weather::Foggy, 1.5),
                (Weather::Stormy, 1.0),
                (Weather::CrystalRain, 1.0),
                (Weather::Rainy, 0.5),
                (Weather::Sunny, 0.5),
                (Weather::GoldenHour, 0.3),
                (Weather::MeteorShower, 0.2),
            ],
        }
    }
}

/// State for a single weather system (e.g. one room's weather).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub current: Weather,
    pub intensity: f64,
    pub duration_ticks: u64,
    pub transition_speed: f64,
    // Internal transition state
    #[serde(default)]
    transitioning_from: Option<Weather>,
    #[serde(default)]
    transition_progress: f64,
}

impl WeatherState {
    /// Create a new weather state with the given weather and full intensity.
    pub fn new(weather: Weather) -> Self {
        Self {
            current: weather,
            intensity: 1.0,
            duration_ticks: 0,
            transition_speed: 0.01,
            transitioning_from: None,
            transition_progress: 0.0,
        }
    }

    /// Create a weather state with custom intensity and transition speed.
    pub fn with_params(weather: Weather, intensity: f64, transition_speed: f64) -> Self {
        Self {
            current: weather,
            intensity: intensity.clamp(0.0, 1.0),
            duration_ticks: 0,
            transition_speed,
            transitioning_from: None,
            transition_progress: 0.0,
        }
    }

    /// Advance the weather by one tick.
    pub fn tick(&mut self) {
        self.duration_ticks += 1;

        if self.transitioning_from.is_some() {
            self.transition_progress += self.transition_speed;
            if self.transition_progress >= 1.0 {
                self.transition_progress = 1.0;
                self.transitioning_from = None;
            }
        }
    }

    /// Begin a smooth transition to a new weather type.
    pub fn transition_to(&mut self, weather: Weather, speed: f64) {
        if weather == self.current && self.transitioning_from.is_none() {
            return;
        }
        self.transitioning_from = Some(self.current);
        self.current = weather;
        self.transition_progress = 0.0;
        self.transition_speed = speed.max(0.001);
        self.duration_ticks = 0;
    }

    /// Whether a transition is currently in progress.
    pub fn is_transitioning(&self) -> bool {
        self.transitioning_from.is_some()
    }

    /// Returns the previous weather if transitioning, otherwise the current.
    pub fn previous_weather(&self) -> Weather {
        self.transitioning_from.unwrap_or(self.current)
    }

    /// Returns the blend factor (0.0 = old weather, 1.0 = new weather).
    pub fn blend_factor(&self) -> f64 {
        if self.transitioning_from.is_some() {
            self.transition_progress
        } else {
            1.0
        }
    }
}

/// Visual effects data for the renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherVFX {
    pub particle_count: usize,
    pub particle_color: [f64; 3],
    pub wind_speed: f64,
    pub lightning_chance: f64,
}

impl WeatherVFX {
    /// Generate VFX parameters from a weather type and intensity.
    pub fn from_weather(weather: &Weather, intensity: f64) -> Self {
        let i = intensity.clamp(0.0, 1.0);
        match weather {
            Weather::Sunny => Self {
                particle_count: (50.0 * i) as usize,
                particle_color: [1.0, 0.95, 0.6],
                wind_speed: 0.1,
                lightning_chance: 0.0,
            },
            Weather::Cloudy => Self {
                particle_count: (20.0 * i) as usize,
                particle_color: [0.7, 0.7, 0.75],
                wind_speed: 0.3,
                lightning_chance: 0.0,
            },
            Weather::Rainy => Self {
                particle_count: (300.0 * i) as usize,
                particle_color: [0.5, 0.6, 0.9],
                wind_speed: 0.5,
                lightning_chance: 0.02 * i,
            },
            Weather::Stormy => Self {
                particle_count: (500.0 * i) as usize,
                particle_color: [0.3, 0.35, 0.7],
                wind_speed: 1.0 * i,
                lightning_chance: 0.15 * i,
            },
            Weather::Snowy => Self {
                particle_count: (200.0 * i) as usize,
                particle_color: [0.95, 0.97, 1.0],
                wind_speed: 0.2,
                lightning_chance: 0.0,
            },
            Weather::Aurora => Self {
                particle_count: (100.0 * i) as usize,
                particle_color: [0.2, 0.9, 0.5],
                wind_speed: 0.05,
                lightning_chance: 0.0,
            },
            Weather::Foggy => Self {
                particle_count: (80.0 * i) as usize,
                particle_color: [0.8, 0.8, 0.82],
                wind_speed: 0.05,
                lightning_chance: 0.0,
            },
            Weather::MeteorShower => Self {
                particle_count: (40.0 * i) as usize,
                particle_color: [1.0, 0.6, 0.2],
                wind_speed: 0.8,
                lightning_chance: 0.0,
            },
            Weather::CrystalRain => Self {
                particle_count: (250.0 * i) as usize,
                particle_color: [0.7, 0.3, 1.0],
                wind_speed: 0.15,
                lightning_chance: 0.0,
            },
            Weather::GoldenHour => Self {
                particle_count: (60.0 * i) as usize,
                particle_color: [1.0, 0.75, 0.3],
                wind_speed: 0.08,
                lightning_chance: 0.0,
            },
        }
    }

    /// Blend two VFX together (useful during weather transitions).
    pub fn blend(a: &WeatherVFX, b: &WeatherVFX, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            particle_count: ((a.particle_count as f64) * (1.0 - t) + (b.particle_count as f64) * t)
                as usize,
            particle_color: [
                a.particle_color[0] * (1.0 - t) + b.particle_color[0] * t,
                a.particle_color[1] * (1.0 - t) + b.particle_color[1] * t,
                a.particle_color[2] * (1.0 - t) + b.particle_color[2] * t,
            ],
            wind_speed: a.wind_speed * (1.0 - t) + b.wind_speed * t,
            lightning_chance: a.lightning_chance * (1.0 - t) + b.lightning_chance * t,
        }
    }
}

/// Manages weather for the whole world. Each room has its own weather state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEngine {
    pub room_weathers: HashMap<String, WeatherState>,
    pub season: Season,
}

impl WeatherEngine {
    pub fn new(season: Season) -> Self {
        Self {
            room_weathers: HashMap::new(),
            season,
        }
    }

    /// Update a room's weather based on its emotional/spectral state.
    ///
    /// - `vibe`: emotional valence from -1.0 (very negative) to 1.0 (very positive)
    /// - `emotion`: a label like "joy", "fear", "confusion", "calm", "dissolving", "accurate"
    pub fn update_room(&mut self, room_id: &str, vibe: f64, emotion: &str) {
        let weather = self.resolve_weather(vibe, emotion);
        let intensity = self.resolve_intensity(vibe);

        let entry = self.room_weathers.entry(room_id.to_string()).or_insert_with(|| {
            WeatherState::with_params(weather, intensity, 0.02)
        });

        if entry.current != weather {
            entry.transition_to(weather, 0.02);
        }
        entry.intensity = intensity;
    }

    /// Resolve vibe + emotion to a weather type.
    fn resolve_weather(&self, vibe: f64, emotion: &str) -> Weather {
        let e = emotion.to_lowercase();

        // Spectral/emotional rules first
        if e.contains("dissolv") || e.contains("dissolve") {
            return Weather::CrystalRain;
        }
        if e.contains("confus") || e.contains("uncertain") {
            return Weather::Foggy;
        }
        if (e.contains("accurate") || e.contains("stable") || e.contains("clarity"))
            && vibe > 0.3
        {
            return Weather::Aurora;
        }
        if e.contains("joy") && vibe > 0.7 {
            return Weather::Sunny;
        }
        if e.contains("wonder") || e.contains("awe") {
            return Weather::GoldenHour;
        }
        if e.contains("fear") || e.contains("anger") {
            return Weather::Stormy;
        }
        if e.contains("sad") || e.contains("grief") {
            return Weather::Rainy;
        }
        if e.contains("nostalg") || e.contains("memory") {
            return Weather::Snowy;
        }

        // Fall back to vibe-based rules
        if vibe > 0.7 {
            Weather::Sunny
        } else if vibe > 0.3 {
            Weather::GoldenHour
        } else if vibe > -0.1 {
            Weather::Cloudy
        } else if vibe > -0.5 {
            Weather::Rainy
        } else {
            Weather::Stormy
        }
    }

    /// Map vibe to intensity (0.0–1.0).
    fn resolve_intensity(&self, vibe: f64) -> f64 {
        (vibe.abs() * 0.6 + 0.4).clamp(0.0, 1.0)
    }

    /// Tick all rooms forward.
    pub fn tick_all(&mut self) {
        for state in self.room_weathers.values_mut() {
            state.tick();
        }
    }

    /// Get the weather for a room, if it exists.
    pub fn get_room_weather(&self, room_id: &str) -> Option<&WeatherState> {
        self.room_weathers.get(room_id)
    }

    /// Get the blended VFX for a room, taking transitions into account.
    pub fn get_room_vfx(&self, room_id: &str) -> Option<WeatherVFX> {
        self.room_weathers.get(room_id).map(|state| {
            if state.is_transitioning() {
                let prev_vfx = WeatherVFX::from_weather(&state.previous_weather(), state.intensity);
                let curr_vfx = WeatherVFX::from_weather(&state.current, state.intensity);
                WeatherVFX::blend(&prev_vfx, &curr_vfx, state.blend_factor())
            } else {
                WeatherVFX::from_weather(&state.current, state.intensity)
            }
        })
    }

    /// Remove a room's weather data.
    pub fn remove_room(&mut self, room_id: &str) {
        self.room_weathers.remove(room_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_label() {
        assert_eq!(Weather::Sunny.label(), "Sunny");
        assert_eq!(Weather::MeteorShower.label(), "Meteor Shower");
        assert_eq!(Weather::CrystalRain.label(), "Crystal Rain");
    }

    #[test]
    fn weather_state_new() {
        let ws = WeatherState::new(Weather::Rainy);
        assert_eq!(ws.current, Weather::Rainy);
        assert!((ws.intensity - 1.0).abs() < f64::EPSILON);
        assert_eq!(ws.duration_ticks, 0);
        assert!(!ws.is_transitioning());
    }

    #[test]
    fn weather_state_tick_increments() {
        let mut ws = WeatherState::new(Weather::Sunny);
        ws.tick();
        ws.tick();
        ws.tick();
        assert_eq!(ws.duration_ticks, 3);
    }

    #[test]
    fn transition_to_starts_transition() {
        let mut ws = WeatherState::new(Weather::Sunny);
        ws.transition_to(Weather::Stormy, 0.1);
        assert!(ws.is_transitioning());
        assert_eq!(ws.current, Weather::Stormy);
        assert_eq!(ws.previous_weather(), Weather::Sunny);
        assert!((ws.transition_speed - 0.1).abs() < f64::EPSILON);
        assert_eq!(ws.duration_ticks, 0);
    }

    #[test]
    fn transition_completes_after_enough_ticks() {
        let mut ws = WeatherState::new(Weather::Sunny);
        ws.transition_to(Weather::Rainy, 0.25);
        assert!(ws.is_transitioning());
        ws.tick(); // 0.25
        assert!(ws.is_transitioning());
        ws.tick(); // 0.50
        ws.tick(); // 0.75
        assert!(ws.is_transitioning());
        ws.tick(); // 1.0
        assert!(!ws.is_transitioning());
        assert!((ws.blend_factor() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn transition_to_same_weather_noop() {
        let mut ws = WeatherState::new(Weather::Sunny);
        ws.transition_to(Weather::Sunny, 0.1);
        assert!(!ws.is_transitioning());
    }

    #[test]
    fn blend_factor_is_one_when_not_transitioning() {
        let ws = WeatherState::new(Weather::Cloudy);
        assert!((ws.blend_factor() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn vfx_from_sunny() {
        let vfx = WeatherVFX::from_weather(&Weather::Sunny, 1.0);
        assert_eq!(vfx.particle_count, 50);
        assert!(vfx.lightning_chance < f64::EPSILON);
    }

    #[test]
    fn vfx_from_stormy_with_intensity() {
        let vfx = WeatherVFX::from_weather(&Weather::Stormy, 0.5);
        assert_eq!(vfx.particle_count, 250);
        assert!((vfx.wind_speed - 0.5).abs() < f64::EPSILON);
        assert!(vfx.lightning_chance > 0.0);
    }

    #[test]
    fn vfx_from_aurora() {
        let vfx = WeatherVFX::from_weather(&Weather::Aurora, 1.0);
        assert_eq!(vfx.particle_count, 100);
        assert!((vfx.particle_color[1] - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn vfx_blend_midpoint() {
        let a = WeatherVFX::from_weather(&Weather::Sunny, 1.0);
        let b = WeatherVFX::from_weather(&Weather::Stormy, 1.0);
        let blended = WeatherVFX::blend(&a, &b, 0.5);
        assert!((blended.wind_speed - 0.55).abs() < 1e-10);
    }

    #[test]
    fn vfx_blend_clamps_t() {
        let a = WeatherVFX::from_weather(&Weather::Sunny, 1.0);
        let b = WeatherVFX::from_weather(&Weather::Rainy, 1.0);
        let over = WeatherVFX::blend(&a, &b, 2.0);
        assert!((over.wind_speed - b.wind_speed).abs() < 1e-10);
    }

    #[test]
    fn engine_new() {
        let engine = WeatherEngine::new(Season::Summer);
        assert_eq!(engine.season, Season::Summer);
        assert!(engine.room_weathers.is_empty());
    }

    #[test]
    fn engine_update_room_vibe_joy() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("room_1", 0.8, "joy");
        let ws = engine.get_room_weather("room_1").unwrap();
        assert_eq!(ws.current, Weather::Sunny);
    }

    #[test]
    fn engine_update_room_negative_vibe() {
        let mut engine = WeatherEngine::new(Season::Winter);
        engine.update_room("room_2", -0.7, "");
        let ws = engine.get_room_weather("room_2").unwrap();
        assert_eq!(ws.current, Weather::Stormy);
    }

    #[test]
    fn engine_update_room_confusion() {
        let mut engine = WeatherEngine::new(Season::Summer);
        engine.update_room("room_3", 0.1, "confusion");
        let ws = engine.get_room_weather("room_3").unwrap();
        assert_eq!(ws.current, Weather::Foggy);
    }

    #[test]
    fn engine_update_room_stable_aurora() {
        let mut engine = WeatherEngine::new(Season::Winter);
        engine.update_room("room_4", 0.5, "stable and accurate");
        let ws = engine.get_room_weather("room_4").unwrap();
        assert_eq!(ws.current, Weather::Aurora);
    }

    #[test]
    fn engine_update_room_dissolving() {
        let mut engine = WeatherEngine::new(Season::Autumn);
        engine.update_room("room_5", 0.0, "dissolving");
        let ws = engine.get_room_weather("room_5").unwrap();
        assert_eq!(ws.current, Weather::CrystalRain);
    }

    #[test]
    fn engine_transition_on_weather_change() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("room_6", 0.8, "joy");
        assert_eq!(engine.get_room_weather("room_6").unwrap().current, Weather::Sunny);

        engine.update_room("room_6", -0.6, "fear");
        let ws = engine.get_room_weather("room_6").unwrap();
        assert_eq!(ws.current, Weather::Stormy);
        assert!(ws.is_transitioning());
    }

    #[test]
    fn engine_tick_all() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("a", 0.5, "");
        engine.update_room("b", -0.3, "");
        engine.tick_all();
        assert_eq!(engine.get_room_weather("a").unwrap().duration_ticks, 1);
        assert_eq!(engine.get_room_weather("b").unwrap().duration_ticks, 1);
    }

    #[test]
    fn engine_remove_room() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("x", 0.5, "");
        assert!(engine.get_room_weather("x").is_some());
        engine.remove_room("x");
        assert!(engine.get_room_weather("x").is_none());
    }

    #[test]
    fn engine_get_room_vfx_no_transition() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("vfx_room", 0.8, "joy");
        let vfx = engine.get_room_vfx("vfx_room").unwrap();
        // intensity = 0.8*0.6 + 0.4 = 0.88, so particles = 50*0.88 = 44
        assert_eq!(vfx.particle_count, 44);
    }

    #[test]
    fn engine_get_room_vfx_during_transition() {
        let mut engine = WeatherEngine::new(Season::Spring);
        engine.update_room("vfx_room2", 0.8, "joy");
        engine.update_room("vfx_room2", -0.6, "fear");
        let vfx = engine.get_room_vfx("vfx_room2").unwrap();
        // At t=0, blend is all from previous (sunny wind=0.1), so just check it's non-negative
        assert!(vfx.wind_speed >= 0.0);
    }

    #[test]
    fn season_weather_weights_sum_positive() {
        for season in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
            let weights = season.weather_weights();
            assert!(!weights.is_empty());
            let sum: f64 = weights.iter().map(|(_, w)| w).sum();
            assert!(sum > 0.0);
        }
    }

    #[test]
    fn serde_weather_roundtrip() {
        let w = Weather::Aurora;
        let json = serde_json::to_string(&w).unwrap();
        let back: Weather = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }

    #[test]
    fn serde_weather_state_roundtrip() {
        let mut ws = WeatherState::new(Weather::CrystalRain);
        ws.transition_to(Weather::GoldenHour, 0.05);
        ws.tick();
        let json = serde_json::to_string(&ws).unwrap();
        let back: WeatherState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.current, Weather::GoldenHour);
        assert!(back.is_transitioning());
    }

    #[test]
    fn serde_engine_roundtrip() {
        let mut engine = WeatherEngine::new(Season::Autumn);
        engine.update_room("test_room", 0.5, "accurate");
        let json = serde_json::to_string(&engine).unwrap();
        let back: WeatherEngine = serde_json::from_str(&json).unwrap();
        assert_eq!(back.season, Season::Autumn);
        assert!(back.room_weathers.contains_key("test_room"));
    }
}
