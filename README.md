# lau-weather

**Dynamic weather system for the Lau voxel game — weather reflects the emotional/spectral state of PLATO rooms.**

Weather isn't random. It's a mirror. When a room feels joy, the sun comes out. When confusion sets in, fog rolls in. When things dissolve, crystal rain falls. Each room in the Lau world has its own weather, driven by the room's emotional valence and the spectral state of the agents within it.

---

## What This Does

`lau-weather` maps `(vibe, emotion) → Weather` for each room in the world. Vibe is a continuous valence from -1.0 (very negative) to 1.0 (very positive). Emotion is a label string (joy, fear, confusion, calm, dissolving, accurate, etc.). The system resolves these into one of 10 weather types, manages smooth transitions between them, and generates visual effects data (particle count, color, wind speed, lightning chance) for the renderer.

A `WeatherEngine` manages per-room weather states and a global season, which influences (but doesn't dictate) weather probabilities.

---

## Key Idea

Weather is a **semantic mapping** from emotional state to atmospheric state. The mapping has two layers:

1. **Emotion-first rules**: specific emotions (dissolving, confusion, fear, joy, etc.) map directly to specific weather types, regardless of vibe.
2. **Vibe fallback**: when no specific emotion matches, the valence (-1.0 to 1.0) determines weather through thresholds.

Transitions between weather types are smooth — the system blends VFX parameters (particle count, color, wind, lightning) over time so the sky doesn't abruptly change.

---

## Install

```toml
[dependencies]
lau-weather = "0.1.0"
```

```bash
cargo add lau-weather
```

Requires Rust 2021 edition. Only external dependency: `serde` (with `derive`).

---

## Quick Start

```rust
use lau_weather::*;

// Create an engine (it's summer)
let mut engine = WeatherEngine::new(Season::Summer);

// Room "library" is happy and stable
engine.update_room("library", 0.8, "joy");
// → Sunny

// Room "dungeon" is scary
engine.update_room("dungeon", -0.7, "fear");
// → Stormy

// Room "puzzle" is confusing
engine.update_room("puzzle", 0.1, "confusion");
// → Foggy

// Room "forge" is stable and accurate
engine.update_room("forge", 0.5, "stable and accurate");
// → Aurora

// Get VFX for rendering
let vfx = engine.get_room_vfx("library").unwrap();
println!("Particles: {}, Wind: {:.1}", vfx.particle_count, vfx.wind_speed);

// Advance time
engine.tick_all();

// Weather changes as emotions shift
engine.update_room("library", -0.3, "sad");
// → Rainy, with a smooth transition from Sunny
let state = engine.get_room_weather("library").unwrap();
assert!(state.is_transitioning());
```

---

## API Reference

### Weather

```rust
pub enum Weather {
    Sunny, Cloudy, Rainy, Stormy, Snowy,
    Aurora, Foggy, MeteorShower, CrystalRain, GoldenHour,
}
```

Each variant has a `.label()` method returning a human-readable string.

### Season

```rust
pub enum Season { Spring, Summer, Autumn, Winter }
```

Each season provides `.weather_weights()` — a `Vec<(Weather, f64)>` of relative weights for weather probabilities. (The engine uses emotion-first rules by default; season weights are available for procedural weather generation.)

| Season | Dominant Weather | Notable |
|--------|-----------------|---------|
| Spring | Sunny (3.0) | Balanced, some rain |
| Summer | Sunny (4.0) | Meteor showers (1.0), no snow (0.0) |
| Autumn | Cloudy (3.0) | Foggy (2.0), golden hour (2.0) |
| Winter | Snowy (4.0) | Aurora (2.0), crystal rain (1.0) |

### WeatherState

Per-room weather with transition support:

```rust
let mut state = WeatherState::new(Weather::Sunny);
state.transition_to(Weather::Stormy, 0.1); // speed = 0.1 per tick

// During transition:
state.is_transitioning();     // true
state.previous_weather();     // Sunny
state.blend_factor();         // 0.0 → 1.0 as transition progresses
state.tick();                 // advances by transition_speed
```

| Method | Description |
|--------|-------------|
| `new(weather)` | Full intensity, no transition |
| `with_params(weather, intensity, speed)` | Custom intensity and transition speed |
| `tick()` | Advance by one tick (increments duration, progresses transition) |
| `transition_to(weather, speed)` | Begin smooth transition |
| `is_transitioning()` | Currently transitioning? |
| `previous_weather()` | The weather we're transitioning from |
| `blend_factor()` | 0.0 = old, 1.0 = new |

### WeatherVFX

Visual effects data for the renderer:

```rust
pub struct WeatherVFX {
    pub particle_count: usize,
    pub particle_color: [f64; 3],
    pub wind_speed: f64,
    pub lightning_chance: f64,
}
```

| Method | Description |
|--------|-------------|
| `from_weather(weather, intensity)` | Generate VFX from weather type + intensity |
| `blend(a, b, t)` | Linear interpolation between two VFX (for transitions) |

**VFX by weather type (at full intensity):**

| Weather | Particles | Color (RGB) | Wind | Lightning |
|---------|-----------|-------------|------|-----------|
| Sunny | 50 | (1.0, 0.95, 0.6) | 0.1 | 0 |
| Cloudy | 20 | (0.7, 0.7, 0.75) | 0.3 | 0 |
| Rainy | 300 | (0.5, 0.6, 0.9) | 0.5 | 2% |
| Stormy | 500 | (0.3, 0.35, 0.7) | 1.0 | 15% |
| Snowy | 200 | (0.95, 0.97, 1.0) | 0.2 | 0 |
| Aurora | 100 | (0.2, 0.9, 0.5) | 0.05 | 0 |
| Foggy | 80 | (0.8, 0.8, 0.82) | 0.05 | 0 |
| MeteorShower | 40 | (1.0, 0.6, 0.2) | 0.8 | 0 |
| CrystalRain | 250 | (0.7, 0.3, 1.0) | 0.15 | 0 |
| GoldenHour | 60 | (1.0, 0.75, 0.3) | 0.08 | 0 |

### WeatherEngine

Manages per-room weather with a global season:

```rust
let mut engine = WeatherEngine::new(Season::Spring);
engine.update_room("room_id", vibe, emotion);
engine.tick_all();
engine.get_room_weather("room_id");  // Option<&WeatherState>
engine.get_room_vfx("room_id");      // Option<WeatherVFX> (blended during transitions)
engine.remove_room("room_id");
```

---

## How It Works

### Weather Resolution

The engine resolves weather in two passes:

**Pass 1 — Emotion rules** (priority order):

| Emotion keyword | Weather |
|----------------|---------|
| dissolv* | CrystalRain |
| confus* / uncertain | Foggy |
| accurate / stable / clarity (vibe > 0.3) | Aurora |
| joy (vibe > 0.7) | Sunny |
| wonder / awe | GoldenHour |
| fear / anger | Stormy |
| sad / grief | Rainy |
| nostalg* / memory | Snowy |

**Pass 2 — Vibe fallback**:

| Vibe range | Weather |
|-----------|---------|
| > 0.7 | Sunny |
| > 0.3 | GoldenHour |
| > -0.1 | Cloudy |
| > -0.5 | Rainy |
| ≤ -0.5 | Stormy |

### Intensity

$$\text{intensity} = |vibe| \times 0.6 + 0.4$$

Clamped to [0, 1]. This means even a neutral vibe produces weather at 40% intensity — the world is never dead still.

### Transitions

When weather changes, the system begins a smooth transition:

$$\text{progress}_{t+1} = \text{progress}_t + \text{speed}$$

VFX parameters are linearly blended:

$$\text{VFX}_{\text{blended}} = (1 - t) \cdot \text{VFX}_{\text{old}} + t \cdot \text{VFX}_{\text{new}}$$

The default transition speed is 0.02 per tick, meaning a full transition takes 50 ticks.

---

## The Math

### VFX Blending

$$p_{\text{blend}} = (1-t) \cdot p_a + t \cdot p_b$$

Applied independently to each VFX parameter (particle count, color channels, wind speed, lightning chance). `t` is clamped to [0, 1].

### Intensity Mapping

$$I(v) = \text{clamp}(|v| \cdot 0.6 + 0.4,\ 0,\ 1)$$

This ensures minimum intensity of 0.4 at neutral vibe (0.0), scaling to 1.0 at extreme vibes (±1.0).

### Particle Scaling

Particle counts scale linearly with intensity:

$$n_{\text{particles}} = \lfloor n_{\text{base}} \cdot I \rfloor$$

---

## Test Coverage

**27 tests** covering:

- **Weather**: labels
- **WeatherState**: construction, tick increments, transitions (start, completion after enough ticks, same-weather noop), blend factors
- **WeatherVFX**: from_weather for Sunny/Stormy/Aurora with intensity, blend midpoint, blend clamp
- **WeatherEngine**: construction, update_room with joy/negative/confusion/stable/dissolving, transition on weather change, tick_all, remove_room, get_room_vfx (with and without transitions)
- **Season**: weather weights sum positive for all seasons
- **Serde**: Weather, WeatherState (mid-transition), WeatherEngine roundtrips

```bash
cargo test
```

---

## License

MIT
