//! Headless, renderer-independent vehicle and camera rules.

use std::f32::consts::{PI, TAU};

pub const FIXED_DT: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub z: f32,
}

impl Point {
    pub fn length(self) -> f32 {
        self.x.hypot(self.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Arena {
    pub half_width: f32,
    pub half_depth: f32,
    pub vehicle_radius: f32,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half_width: 108.0,
            half_depth: 108.0,
            vehicle_radius: 2.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DriveConfig {
    pub speed: f32,
    pub reverse_factor: f32,
    pub strafe_factor: f32,
    pub turn_rate: f32,
    /// Zero is immediate response, not a wheeled-car inertia simulation.
    pub response_time: f32,
}

impl Default for DriveConfig {
    fn default() -> Self {
        Self {
            speed: 7.0,
            reverse_factor: 0.55,
            strafe_factor: 0.8,
            turn_rate: 2.5,
            response_time: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DriveInput {
    pub forward: f32,
    pub strafe: f32,
    /// Positive turns right, negative turns left.
    pub turn: f32,
    pub heading_override: Option<f32>,
}

/// Vehicle controls deliberately never turn rotation input into sideways motion.
pub fn vehicle_controls(forward: f32, turn: f32, mouse_heading: Option<f32>) -> DriveInput {
    DriveInput {
        forward,
        turn: if mouse_heading.is_some() { 0.0 } else { turn },
        strafe: 0.0,
        heading_override: mouse_heading,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vehicle {
    pub position: Point,
    pub heading: f32,
    pub velocity: Point,
    pub distance: f32,
    pub touching_wall: bool,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            position: Point { x: 0.0, z: 34.0 },
            heading: 0.0,
            velocity: Point::default(),
            distance: 0.0,
            touching_wall: false,
        }
    }
}

fn axis(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

pub fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

pub fn follow_angle(current: f32, target: f32, response: f32, dt: f32) -> f32 {
    wrap_angle(current + wrap_angle(target - current) * (1.0 - (-response * dt.max(0.0)).exp()))
}

impl Vehicle {
    pub fn stop(&mut self) {
        self.velocity = Point::default();
    }

    /// Ease longitudinal speed only: a mouse turn must never introduce lateral inertia.
    pub fn step_vehicle(
        &mut self,
        mut input: DriveInput,
        config: DriveConfig,
        arena: Arena,
        dt: f32,
    ) {
        if !dt.is_finite() || dt <= 0.0 || dt > 0.1 {
            return;
        }
        input.strafe = 0.0;
        let old_speed = self.velocity.x * self.heading.sin() - self.velocity.z * self.heading.cos();
        let forward = axis(input.forward);
        let target = forward
            * config.speed
            * if forward < 0.0 {
                config.reverse_factor
            } else {
                1.0
            };
        let factor = if config.response_time <= 0.0 || forward == 0.0 {
            1.0
        } else {
            1.0 - (-dt / config.response_time).exp()
        };
        let speed = old_speed + (target - old_speed) * factor;
        input.forward = if speed < 0.0 {
            speed / (config.speed * config.reverse_factor).max(0.001)
        } else {
            speed / config.speed.max(0.001)
        };
        self.step(
            input,
            DriveConfig {
                response_time: 0.0,
                strafe_factor: 0.0,
                ..config
            },
            arena,
            dt,
        );
    }

    pub fn step(&mut self, input: DriveInput, config: DriveConfig, arena: Arena, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 || dt > 0.1 {
            return;
        }
        let old_heading = self.heading;
        if let Some(heading) = input.heading_override.filter(|h| h.is_finite()) {
            self.heading = wrap_angle(heading);
        } else {
            self.heading = wrap_angle(self.heading + axis(input.turn) * config.turn_rate * dt);
        }
        let move_heading = if input.heading_override.is_some() {
            self.heading
        } else {
            old_heading + wrap_angle(self.heading - old_heading) * 0.5
        };
        let forward = axis(input.forward);
        let strafe = axis(input.strafe);
        let normalizer = forward.hypot(strafe).max(1.0);
        let forward_speed = forward
            * config.speed
            * if forward < 0.0 {
                config.reverse_factor
            } else {
                1.0
            };
        let sideways = strafe * config.speed * config.strafe_factor;
        let target = Point {
            x: (move_heading.sin() * forward_speed + move_heading.cos() * sideways) / normalizer,
            z: (-move_heading.cos() * forward_speed + move_heading.sin() * sideways) / normalizer,
        };
        let factor = if config.response_time <= 0.0 {
            1.0
        } else {
            1.0 - (-dt / config.response_time).exp()
        };
        self.velocity.x += (target.x - self.velocity.x) * factor;
        self.velocity.z += (target.z - self.velocity.z) * factor;
        let old = self.position;
        let desired = Point {
            x: old.x + self.velocity.x * dt,
            z: old.z + self.velocity.z * dt,
        };
        let max_x = (arena.half_width - arena.vehicle_radius).max(0.0);
        let max_z = (arena.half_depth - arena.vehicle_radius).max(0.0);
        self.position.x = desired.x.clamp(-max_x, max_x);
        self.position.z = desired.z.clamp(-max_z, max_z);
        self.touching_wall = self.position != desired;
        if self.position.x != desired.x {
            self.velocity.x = 0.0;
        }
        if self.position.z != desired.z {
            self.velocity.z = 0.0;
        }
        self.distance += (self.position.x - old.x).hypot(self.position.z - old.z);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.48,
            distance: 19.0,
        }
    }
}

impl Orbit {
    /// Mouse motion is already a per-frame displacement: do not multiply by dt.
    pub fn drag(&mut self, dx: f32, dy: f32, sensitivity: f32, invert_y: bool) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.yaw = wrap_angle(self.yaw + dx * sensitivity);
        self.pitch =
            (self.pitch + dy * sensitivity * if invert_y { -1.0 } else { 1.0 }).clamp(0.08, 1.35);
    }

    pub fn zoom(&mut self, lines: f32) {
        if lines.is_finite() {
            self.distance = (self.distance - lines * 1.6).clamp(7.0, 38.0);
        }
    }

    pub fn offset(self) -> [f32; 3] {
        let horizontal = self.distance * self.pitch.cos();
        [
            -self.yaw.sin() * horizontal,
            self.distance * self.pitch.sin(),
            self.yaw.cos() * horizontal,
        ]
    }
}

/// Clip the camera boom to the inside of the courtyard. No graphics dependency.
pub fn camera_wall_fraction(pivot: [f32; 3], camera: [f32; 3], half_extent: f32) -> f32 {
    let mut fraction: f32 = 1.0;
    for index in [0, 2] {
        let direction = camera[index] - pivot[index];
        if direction > 0.0001 {
            fraction = fraction.min((half_extent - pivot[index]) / direction);
        } else if direction < -0.0001 {
            fraction = fraction.min((-half_extent - pivot[index]) / direction);
        }
    }
    fraction.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vehicle_soft_response_cannot_slide_after_mouse_turn() {
        let mut v = Vehicle::default();
        let cfg = DriveConfig {
            response_time: 0.12,
            ..Default::default()
        };
        for _ in 0..60 {
            v.step_vehicle(
                vehicle_controls(1.0, 0.0, None),
                cfg,
                Arena::default(),
                FIXED_DT,
            );
        }
        let old = v.position;
        v.step_vehicle(
            vehicle_controls(1.0, 0.0, Some(PI / 2.0)),
            cfg,
            Arena::default(),
            FIXED_DT,
        );
        assert!((v.position.z - old.z).abs() < 0.00001);
        assert!(v.position.x > old.x);
        let old = v.position;
        v.step_vehicle(
            vehicle_controls(0.0, 0.0, Some(0.0)),
            cfg,
            Arena::default(),
            FIXED_DT,
        );
        assert_eq!(v.position, old);
    }
    #[test]
    fn vehicle_soft_response_can_change_from_forward_to_reverse() {
        let mut v = Vehicle::default();
        let cfg = DriveConfig {
            response_time: 0.12,
            ..Default::default()
        };
        for _ in 0..60 {
            v.step_vehicle(
                vehicle_controls(1.0, 0.0, None),
                cfg,
                Arena::default(),
                FIXED_DT,
            );
        }
        for _ in 0..120 {
            v.step_vehicle(
                vehicle_controls(-1.0, 0.0, None),
                cfg,
                Arena::default(),
                FIXED_DT,
            );
        }
        assert!(v.velocity.z > 3.8 && v.velocity.length() <= cfg.speed * cfg.reverse_factor + 0.01);
    }
    #[test]
    fn right_mouse_and_rotation_never_generate_strafe() {
        let input = vehicle_controls(0.0, 1.0, Some(0.8));
        let mut vehicle = Vehicle::default();
        let original = vehicle.position;
        vehicle.step(input, DriveConfig::default(), Arena::default(), FIXED_DT);
        assert_eq!(input.strafe, 0.0);
        assert_eq!(vehicle.position, original);
    }
    #[test]
    fn keyboard_rotation_does_not_move_without_forward() {
        let input = vehicle_controls(0.0, -1.0, None);
        assert_eq!(input.strafe, 0.0);
        assert_eq!(input.turn, -1.0);
    }
    fn drive(v: &mut Vehicle, input: DriveInput, seconds: f32) {
        for _ in 0..(seconds / FIXED_DT).round() as usize {
            v.step(input, DriveConfig::default(), Arena::default(), FIXED_DT);
        }
    }
    #[test]
    fn forward_follows_heading() {
        let mut v = Vehicle::default();
        drive(
            &mut v,
            DriveInput {
                forward: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!((v.position.z - 27.0).abs() < 0.01 && v.position.x.abs() < 0.001);
    }
    #[test]
    fn reverse_is_slower() {
        let mut v = Vehicle::default();
        drive(
            &mut v,
            DriveInput {
                forward: -1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!((v.position.z - 37.85).abs() < 0.01);
    }
    #[test]
    fn turning_in_place_does_not_slide() {
        let mut v = Vehicle::default();
        let position = v.position;
        drive(
            &mut v,
            DriveInput {
                turn: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert_eq!(v.position, position);
        assert!(v.heading > 2.4);
    }
    #[test]
    fn strafe_does_not_turn() {
        let mut v = Vehicle::default();
        drive(
            &mut v,
            DriveInput {
                strafe: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert!((v.position.x - 5.6).abs() < 0.01 && v.heading == 0.0 && v.position.z == 34.0);
    }
    #[test]
    fn diagonal_does_not_gain_speed() {
        let mut v = Vehicle::default();
        v.step(
            DriveInput {
                forward: 1.0,
                strafe: 1.0,
                ..Default::default()
            },
            DriveConfig::default(),
            Arena::default(),
            FIXED_DT,
        );
        assert!(v.velocity.length() <= 7.001);
    }
    #[test]
    fn direct_mode_stops_immediately() {
        let mut v = Vehicle::default();
        drive(
            &mut v,
            DriveInput {
                forward: 1.0,
                ..Default::default()
            },
            1.0,
        );
        let p = v.position;
        v.step(
            DriveInput::default(),
            DriveConfig::default(),
            Arena::default(),
            FIXED_DT,
        );
        assert_eq!(v.position, p);
    }
    #[test]
    fn mouse_heading_has_priority() {
        let mut v = Vehicle::default();
        drive(
            &mut v,
            DriveInput {
                forward: 1.0,
                heading_override: Some(PI / 2.0),
                ..Default::default()
            },
            1.0,
        );
        assert!((v.position.x - 7.0).abs() < 0.01 && (v.position.z - 34.0).abs() < 0.001);
    }
    #[test]
    fn bounds_hold_for_every_corner() {
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let mut v = Vehicle {
                    position: Point {
                        x: x * 105.5,
                        z: z * 105.5,
                    },
                    ..Default::default()
                };
                drive(
                    &mut v,
                    DriveInput {
                        forward: -z,
                        strafe: x,
                        ..Default::default()
                    },
                    5.0,
                );
                assert!(v.position.x.abs() <= 105.5 && v.position.z.abs() <= 105.5);
            }
        }
    }
    #[test]
    fn wall_collision_allows_sliding() {
        let mut v = Vehicle {
            position: Point { x: 105.5, z: 0.0 },
            ..Default::default()
        };
        drive(
            &mut v,
            DriveInput {
                forward: 1.0,
                strafe: 1.0,
                ..Default::default()
            },
            1.0,
        );
        assert_eq!(v.position.x, 105.5);
        assert!(v.position.z < -4.5 && v.touching_wall);
    }
    #[test]
    fn frame_rate_independent_forward() {
        let mut a = Vehicle::default();
        let mut b = a;
        let input = DriveInput {
            forward: 1.0,
            ..Default::default()
        };
        for _ in 0..60 {
            a.step(input, DriveConfig::default(), Arena::default(), 1.0 / 60.0);
        }
        for _ in 0..120 {
            b.step(input, DriveConfig::default(), Arena::default(), 1.0 / 120.0);
        }
        assert!((a.position.z - b.position.z).abs() < 0.001);
    }
    #[test]
    fn invalid_time_and_input_cannot_corrupt_position() {
        let mut v = Vehicle::default();
        let p = v.position;
        v.step(
            DriveInput::default(),
            DriveConfig::default(),
            Arena::default(),
            f32::NAN,
        );
        v.step(
            DriveInput {
                forward: f32::NAN,
                turn: f32::INFINITY,
                ..Default::default()
            },
            DriveConfig::default(),
            Arena::default(),
            FIXED_DT,
        );
        assert_eq!(v.position, p);
    }
    #[test]
    fn smooth_mode_is_bounded() {
        let mut v = Vehicle::default();
        let config = DriveConfig {
            response_time: 0.12,
            ..Default::default()
        };
        v.step(
            DriveInput {
                forward: 1.0,
                ..Default::default()
            },
            config,
            Arena::default(),
            FIXED_DT,
        );
        assert!(v.velocity.length() > 0.0 && v.velocity.length() < config.speed);
    }
    #[test]
    fn zoom_and_pitch_are_bounded() {
        let mut o = Orbit::default();
        o.zoom(10000.0);
        assert_eq!(o.distance, 7.0);
        o.zoom(-10000.0);
        assert_eq!(o.distance, 38.0);
        o.drag(0.0, 10000.0, 0.003, false);
        assert_eq!(o.pitch, 1.35);
    }
    #[test]
    fn camera_drag_sums_without_frame_time() {
        let mut a = Orbit::default();
        let mut b = a;
        a.drag(100.0, 50.0, 0.003, false);
        for _ in 0..10 {
            b.drag(10.0, 5.0, 0.003, false);
        }
        assert!((a.yaw - b.yaw).abs() < 0.0001 && (a.pitch - b.pitch).abs() < 0.0001);
    }
    #[test]
    fn camera_boom_stays_inside_walls() {
        let fraction = camera_wall_fraction([0.0, 2.0, 50.0], [0.0, 10.0, 70.0], 58.0);
        assert!((fraction - 0.4).abs() < 0.0001);
        assert_eq!(
            camera_wall_fraction([0.0, 2.0, 0.0], [0.0, 10.0, 20.0], 58.0),
            1.0
        );
    }
    #[test]
    fn angle_follow_uses_shortest_path() {
        let angle = follow_angle(PI - 0.01, -PI + 0.01, 5.0, 0.1);
        assert!(wrap_angle(angle - (PI - 0.01)).abs() < 0.02);
    }
}
