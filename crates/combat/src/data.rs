//! WotLK 3.3.5 / AzerothCore reference, provenance and exceptions: docs/wotlk-reference.md.
use crate::Kind;

pub const ITEM_LEVEL: u16 = 215;
pub const GEAR_SCALE: f32 = (ITEM_LEVEL as f32 * 15. - 2500.) / 500.;
pub const BOSS_HP: f32 = 23_009_250.;
pub const TURRET_HP: f32 = 64_000.;
pub const RIDER_HP: f32 = 22_000.;
// shortcut: one-button ranged training profile, replace with class stats if class selection is added.
pub const RIDER_DPS: f32 = 3_500.;
pub const BASE_SPEED: f32 = 7.;
pub const ARENA_HALF: f32 = 108.;
pub const EDGE: f32 = ARENA_HALF - 5.;
pub const VENTS_DURATION: f32 = 10.;
pub const PYRITE_TICK: f32 = 12_000.;

pub fn vehicle_hp(kind: Kind) -> f32 {
    (12_600.
        * match kind {
            Kind::Demo => 50.,
            Kind::Siege => 90.,
            Kind::Moto => 40.,
        }
        * GEAR_SCALE)
        .round()
}

pub fn boss_hp(towers: bool) -> f32 {
    BOSS_HP * if towers { 1.4_f32.powi(4) } else { 1. }
}

// Source world X/Y translated by (-270, +34), one world unit per yard.
pub const FIRE_PATH: [(f32, f32); 7] = [
    (94.4, 24.3),
    (61.674, -34.6878),
    (4.578, -58.1829),
    (-43.567, -32.6652),
    (-63.908, -0.7447),
    (-29.792, 35.10346),
    (67.199, 45.7051),
];
pub const LIFE_CORNERS: [(f32, f32); 4] = [
    (104., -107.),
    (112.9, 108.),
    (-110.6, 98.1),
    (-112.3, -106.26),
];
