//! Bridge between headless combat, targeting, native rendering and HUD.
use crate::{CameraControl, FollowCamera, Player, RunState, Simulation, SmokeTest};
use bevy::prelude::*;
use leviathan_combat::{Fight, HazardKind, Kind, Outcome, RiderState, Role, ShotKind, spells};
use leviathan_driving::Point;
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct Raid(pub Fight);
impl Default for Raid {
    fn default() -> Self {
        Self(Fight::new(Role::DemoDriver, true))
    }
}
#[derive(Resource, Default)]
pub struct Targeting {
    pub slot: Option<usize>,
    pub point: Option<Point>,
    pub queued: Vec<(usize, Option<Point>)>,
    pub notice: String,
    pub notice_time: f32,
}
#[derive(Component)]
pub struct RaidBody(pub usize);
#[derive(Component)]
pub struct BossBody;
#[derive(Component)]
pub struct PlayerVisual(pub Kind, pub bool);
#[derive(Component)]
pub(super) struct Effect;
#[derive(Resource)]
pub struct Effects {
    pub cube: Handle<Mesh>,
    pub ball: Handle<Mesh>,
    pub ring: Handle<Mesh>,
    pub flame: Handle<Mesh>,
    pub disk: Handle<Mesh>,
    pub colors: Vec<Handle<StandardMaterial>>,
    pub entities: HashMap<u64, Entity>,
}
#[derive(Resource, Default)]
pub struct CombatValidation {
    pub casts: u32,
    pub projection_ok: bool,
    pub air_projection_ok: bool,
    pub last: f32,
}
pub fn smoke_combat(
    smoke: Res<SmokeTest>,
    mut raid: ResMut<Raid>,
    mut validation: ResMut<CombatValidation>,
    camera: Single<(&Camera, &GlobalTransform), With<FollowCamera>>,
) {
    if !smoke.enabled || !std::env::args().any(|a| a == "--smoke-combat") {
        return;
    }
    if raid.0.outcome == Outcome::Ready {
        raid.0.start();
    }
    // Rendering smoke test, not a survival benchmark: keep the scripted, non-kiting player alive.
    raid.0.player_mut().hp = 100.;
    // A long render test must reach Freya/projection/shutdown without ending on an early kill.
    if combat_seconds() > 30 {
        raid.0.boss.hp = raid.0.boss.hp.max(400.);
    }
    let t = raid.0.time;
    let (cam, transform) = *camera;
    let desired = Vec3::new(raid.0.boss.position.x, 0., raid.0.boss.position.z);
    if let Ok(pixel) = cam.world_to_viewport(transform, desired)
        && let Ok(ray) = cam.viewport_to_world(transform, pixel)
        && let Some(d) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
    {
        validation.projection_ok |= ray.get_point(d).distance(desired) < 0.02;
    }
    for c in raid.0.crates.iter().filter(|c| c.air > 0.) {
        let target = Vec3::new(c.position.x, 8., c.position.z);
        if let Ok(pixel) = cam.world_to_viewport(transform, target)
            && let Ok(ray) = cam.viewport_to_world(transform, pixel)
            && let Some(d) = ray.intersect_plane(Vec3::Y * 8., InfinitePlane3d::new(Vec3::Y))
        {
            validation.air_projection_ok |= ray.get_point(d).distance(target) < 0.02;
        }
    }
    if t - validation.last < 1.6 {
        return;
    }
    validation.last = t;
    let slot = if t > 42.
        && t < 50.
        && raid.0.player().passenger
        && raid.0.role == Role::DemoDriver
        && raid.0.player().loaded
    {
        3
    } else if t > 40.
        && t < 45.
        && raid.0.role == Role::DemoPassenger
        && raid.0.player().passenger
        && !raid.0.player().loaded
    {
        4
    } else {
        match raid.0.role {
            Role::DemoDriver => 1,
            Role::DemoPassenger => 0,
            Role::SiegeDriver => 1,
            Role::SiegeGunner => 1,
            Role::Moto => 1,
        }
    };
    let p = raid.0.boss.position;
    let aimed = spells(raid.0.role)[slot].aimed;
    if raid
        .0
        .cast(slot, if aimed { Some(p) } else { None })
        .is_ok()
    {
        validation.casts += 1;
    }
}
pub fn combat_seconds() -> u64 {
    std::env::args()
        .find_map(|a| {
            a.strip_prefix("--combat-seconds=")
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(20)
        .clamp(20, 120)
}

pub fn reset(
    role: Role,
    towers: bool,
    raid: &mut Raid,
    sim: &mut Simulation,
    aim: &mut Targeting,
    camera: &mut CameraControl,
) {
    raid.0 = Fight::new(role, towers);
    sim.current = raid.0.player().body;
    sim.previous = sim.current;
    sim.signed_travel = 0.;
    aim.slot = None;
    aim.point = None;
    aim.queued.clear();
    camera.orbit.yaw = 0.;
    camera.free_look_delay = 0.;
    camera.initialized = false;
}
pub fn request(slot: usize, raid: &Raid, aim: &mut Targeting) {
    if raid.0.outcome != Outcome::Fighting {
        aim.notice = "Entrée : lancer le combat.".into();
        aim.notice_time = 3.;
        return;
    }
    if let Some(spell) = spells(raid.0.role).get(slot) {
        if !raid.0.onboard() {
            aim.notice = "Passager projeté : attendre la récupération par une moto.".into();
            aim.notice_time = 3.;
            return;
        }
        if raid.0.player().cooldowns[slot] > 0. {
            aim.notice = "Sort en recharge.".into();
            aim.notice_time = 2.;
            return;
        }
        if spell.aimed {
            aim.slot = Some(slot);
            aim.notice = format!(
                "{} : {}, clic gauche pour tirer ; clic droit pour annuler.",
                spell.name,
                if air_target(raid.0.role, slot) {
                    "vise une caisse aérienne"
                } else {
                    "vise au sol"
                }
            );
            aim.notice_time = 4.;
        } else {
            aim.queued.push((slot, None));
        }
    }
}
pub fn air_target(role: Role, slot: usize) -> bool {
    matches!(
        (role, slot),
        (Role::DemoPassenger, 1) | (Role::SiegeGunner, 0)
    )
}
pub fn commands(
    keys: Res<ButtonInput<KeyCode>>,
    mut raid: ResMut<Raid>,
    mut aim: ResMut<Targeting>,
    mut sim: ResMut<Simulation>,
    mut camera: ResMut<CameraControl>,
    mut state: ResMut<RunState>,
    smoke: Res<SmokeTest>,
) {
    if smoke.enabled {
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) && raid.0.outcome == Outcome::Ready {
        let role = raid.0.role;
        let towers = raid.0.towers;
        reset(role, towers, &mut raid, &mut sim, &mut aim, &mut camera);
    }
    if keys.just_pressed(KeyCode::F5) {
        let role = raid.0.role;
        let towers = raid.0.towers;
        reset(role, towers, &mut raid, &mut sim, &mut aim, &mut camera);
        state.paused = false;
    }
    if raid.0.outcome != Outcome::Fighting && keys.just_pressed(KeyCode::F2) {
        let index = Role::ALL.iter().position(|r| *r == raid.0.role).unwrap();
        let towers = raid.0.towers;
        reset(
            Role::ALL[(index + 1) % 5],
            towers,
            &mut raid,
            &mut sim,
            &mut aim,
            &mut camera,
        );
    }
    if raid.0.outcome == Outcome::Ready && keys.just_pressed(KeyCode::F7) {
        raid.0.towers = !raid.0.towers;
    }
    if state.paused {
        return;
    }
    if keys.just_pressed(KeyCode::Enter) {
        raid.0.start();
    }
    for (slot, (key, num)) in [
        (KeyCode::Digit1, KeyCode::Numpad1),
        (KeyCode::Digit2, KeyCode::Numpad2),
        (KeyCode::Digit3, KeyCode::Numpad3),
        (KeyCode::Digit4, KeyCode::Numpad4),
        (KeyCode::Digit5, KeyCode::Numpad5),
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) || keys.just_pressed(num) {
            request(slot, &raid, &mut aim);
        }
    }
}

pub fn targeting(
    camera: Single<(&Camera, &GlobalTransform), With<FollowCamera>>,
    window: Single<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time<Real>>,
    state: Res<RunState>,
    raid: Res<Raid>,
    mut aim: ResMut<Targeting>,
    mut gizmos: Gizmos,
) {
    aim.notice_time = (aim.notice_time - time.delta_secs()).max(0.);
    if state.paused || raid.0.outcome != Outcome::Fighting {
        aim.slot = None;
        aim.queued.clear();
        return;
    }
    if let Some(slot) = aim.slot {
        let Some(cursor) = window.cursor_position() else {
            aim.point = None;
            return;
        };
        let (camera, transform) = *camera;
        let air = air_target(raid.0.role, slot);
        let height = if air { 8. } else { 0. };
        aim.point = camera
            .viewport_to_world(transform, cursor)
            .ok()
            .and_then(|ray| {
                ray.intersect_plane(Vec3::Y * height, InfinitePlane3d::new(Vec3::Y))
                    .map(|d| ray.get_point(d))
            })
            .map(|p| Point { x: p.x, z: p.z });
        if let Some(point) = aim.point {
            let valid = raid.0.aim_valid(slot, point);
            let color = if valid {
                Color::srgb(0.4, 1., 0.25)
            } else {
                Color::srgb(1., 0.2, 0.1)
            };
            let spec = spells(raid.0.role)[slot];
            gizmos.circle(
                Isometry3d::new(
                    Vec3::new(point.x, height + 0.12, point.z),
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ),
                spec.radius.max(1.),
                color,
            );
            let p = raid.0.player().body.position;
            let mut old = Vec3::new(p.x, 3., p.z);
            for i in 1..=24 {
                let t = i as f32 / 24.;
                let next = Vec3::new(
                    p.x + (point.x - p.x) * t,
                    3. * (1. - t) + height * t + if air { 1.0 } else { 16.0 } * t * (1. - t),
                    p.z + (point.z - p.z) * t,
                );
                gizmos.line(old, next, color);
                old = next;
            }
            if mouse.just_pressed(MouseButton::Left)
                && cursor.y < window.height() - 175.
                && cursor.y > 125.
                && valid
            {
                aim.queued.push((slot, Some(point)));
                aim.slot = None;
            }
        }
    }
}

pub fn setup_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut colors: Vec<_> = [
        Color::srgb(0.7, 0.54, 0.33),
        Color::srgb(0.45, 0.18, 1.),
        Color::srgb(1., 0.43, 0.05),
        Color::srgb(0.5, 0.85, 1.),
        Color::srgb(0.95, 0.8, 0.18),
        Color::srgb(0.08, 0.065, 0.045),
        Color::srgb(0.25, 0.55, 0.16),
        Color::srgb(0.9, 0.86, 0.65),
    ]
    .into_iter()
    .map(|c| {
        materials.add(StandardMaterial {
            base_color: c,
            emissive: c.to_linear() * 0.3,
            perceptual_roughness: 0.85,
            ..default()
        })
    })
    .collect();
    for alpha in [0.16, 0.36] {
        colors.push(materials.add(StandardMaterial {
            base_color: Color::srgba(0.35, 0.7, 1., alpha),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.3,
            ..default()
        }));
    }
    colors.push(materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.055, 0.025, 0.65),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.,
        ..default()
    }));
    let flame = images.add(crate::textures::flame_image());
    colors.push(materials.add(StandardMaterial {
        base_color_texture: Some(flame),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        ..default()
    }));
    commands.insert_resource(Effects {
        cube: meshes.add(Cuboid::default()),
        ball: meshes.add(Sphere::new(1.).mesh().uv(12, 8)),
        ring: meshes.add(Torus::new(0.94, 1.)),
        flame: meshes.add(Rectangle::new(1., 1.)),
        disk: meshes.add(Circle::new(1.)),
        colors,
        entities: HashMap::new(),
    });
}
pub fn initialize(mut raid: ResMut<Raid>, mut sim: ResMut<Simulation>) {
    let role = std::env::args()
        .find_map(|a| a.strip_prefix("--role=").map(str::to_owned))
        .map(|r| match r.as_str() {
            "demo-passenger" => Role::DemoPassenger,
            "siege" => Role::SiegeDriver,
            "gunner" => Role::SiegeGunner,
            "moto" => Role::Moto,
            _ => Role::DemoDriver,
        })
        .unwrap_or(Role::DemoDriver);
    raid.0 = Fight::new(role, true);
    sim.current = raid.0.player().body;
    sim.previous = sim.current;
}
pub fn render(
    mut commands: Commands,
    raid: Res<Raid>,
    mut effects: ResMut<Effects>,
    mut transforms: Query<
        (
            &mut Transform,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (
            Without<Player>,
            Without<RaidBody>,
            Without<BossBody>,
            With<Effect>,
        ),
    >,
    mut actors: Query<
        (
            &mut Transform,
            &mut Visibility,
            Option<&RaidBody>,
            Has<BossBody>,
        ),
        (Without<Player>, Without<Effect>, Without<PlayerVisual>),
    >,
    mut player_visuals: Query<
        (&PlayerVisual, &mut Visibility),
        (Without<RaidBody>, Without<BossBody>, Without<Effect>),
    >,
    model: Res<crate::models::ModelStatus>,
    camera: Single<&GlobalTransform, With<FollowCamera>>,
) {
    let f = &raid.0;
    for (mut t, mut visible, body, boss) in &mut actors {
        if let Some(index) = body.map(|b| b.0) {
            let v = &f.vehicles[index];
            *visible = if index == f.role.index() && f.onboard() {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
            t.translation = Vec3::new(
                v.body.position.x,
                if v.hp > 0. { 0. } else { -1. },
                v.body.position.z,
            );
            t.rotation = Quat::from_rotation_y(-v.body.heading);
        } else if boss {
            t.translation = Vec3::new(f.boss.position.x, 0., f.boss.position.z);
            let p = f.vehicles[f.boss.target].body.position;
            t.rotation =
                Quat::from_rotation_y(-(p.x - f.boss.position.x).atan2(-(p.z - f.boss.position.z)));
        }
    }
    for (kind, mut visible) in &mut player_visuals {
        *visible =
            if f.onboard() && kind.0 == f.player().kind && (kind.1 != model.has_player(kind.0)) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
    }
    let mut active = HashSet::new();
    let mut draw = |key: u64, p: Vec3, scale: Vec3, shape: u8, color: usize, rotation: Quat| {
        active.insert(key);
        let t = Transform {
            translation: p,
            scale,
            rotation,
        };
        if let Some(entity) = effects.entities.get(&key) {
            if let Ok((mut old, mut mesh, mut material)) = transforms.get_mut(*entity) {
                *old = t;
                let desired = match shape {
                    0 => effects.ball.clone(),
                    1 => effects.cube.clone(),
                    3 => effects.flame.clone(),
                    4 => effects.disk.clone(),
                    _ => effects.ring.clone(),
                };
                if mesh.0 != desired {
                    mesh.0 = desired;
                }
                if material.0 != effects.colors[color] {
                    material.0 = effects.colors[color].clone();
                }
            }
        } else {
            let mesh = match shape {
                0 => effects.ball.clone(),
                1 => effects.cube.clone(),
                3 => effects.flame.clone(),
                4 => effects.disk.clone(),
                _ => effects.ring.clone(),
            };
            let mut entry = commands.spawn((
                Effect,
                Mesh3d(mesh),
                MeshMaterial3d(effects.colors[color].clone()),
                t,
            ));
            if shape == 3 {
                entry.insert(bevy::light::NotShadowCaster);
            }
            let entity = entry.id();
            effects.entities.insert(key, entity);
        }
    };
    for shot in &f.shots {
        let t = (shot.age / shot.flight).min(1.);
        draw(
            shot.id,
            Vec3::new(
                shot.start.x + (shot.end.x - shot.start.x) * t,
                3. * (1. - t)
                    + if shot.kind == ShotKind::Rocket {
                        8. * t + 1. * t * (1. - t)
                    } else {
                        16. * t * (1. - t)
                    },
                shot.start.z + (shot.end.z - shot.start.z) * t,
            ),
            Vec3::splat(if shot.kind == ShotKind::Passenger {
                0.6
            } else {
                0.42
            }),
            0,
            match shot.kind {
                ShotKind::Pyrite => 1,
                ShotKind::Cannon | ShotKind::Mortar => 2,
                ShotKind::Rocket => 4,
                ShotKind::Passenger => 7,
                _ => 0,
            },
            Quat::IDENTITY,
        );
    }
    for impact in &f.impacts {
        draw(
            impact.id,
            Vec3::new(impact.position.x, impact.height, impact.position.z),
            Vec3::new(impact.age * 6. + 0.1, 0.12, impact.age * 6. + 0.1),
            2,
            if impact.hit { 2 } else { 0 },
            Quat::IDENTITY,
        );
    }
    for c in &f.crates {
        let height = if c.air > 0. { 8. } else { 0.8 };
        draw(
            c.id,
            Vec3::new(
                c.position.x,
                if c.air > 0. { 8. } else { 0.8 },
                c.position.z,
            ),
            Vec3::splat(0.9),
            1,
            0,
            Quat::from_rotation_y(0.5),
        );
        draw(
            c.id + (1_u64 << 59),
            Vec3::new(c.position.x, height + 0.75, c.position.z),
            Vec3::splat(0.5),
            0,
            1,
            Quat::IDENTITY,
        );
    }
    for a in &f.adds {
        draw(
            a.id,
            Vec3::new(a.position.x, 1.4, a.position.z),
            Vec3::new(1., 1.5, 1.),
            0,
            6,
            Quat::IDENTITY,
        );
    }
    for r in &f.riders {
        if r.state == RiderState::Ground
            || (r.home == f.role.index()
                && f.role == Role::DemoPassenger
                && r.state == RiderState::Turret)
        {
            draw(
                r.id,
                Vec3::new(
                    r.position.x,
                    if r.state == RiderState::Turret {
                        8.5
                    } else {
                        1.
                    },
                    r.position.z,
                ),
                Vec3::new(0.5, 1.2, 0.5),
                1,
                7,
                Quat::IDENTITY,
            );
        }
    }
    for h in &f.hazards {
        if (h.kind == HazardKind::Fire && h.age > 0.8) || (h.kind == HazardKind::Tar && h.burning) {
            draw(
                h.id,
                Vec3::new(h.position.x, 0.04, h.position.z),
                Vec3::splat(h.radius),
                4,
                10,
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            );
            for j in 0..2 {
                let height = 3.5 + (f.time * 5. + h.position.x + j as f32 * 2.).sin() * 0.45;
                let point = Vec3::new(
                    h.position.x + (j as f32 - 0.5) * h.radius * 0.5,
                    height * 0.5,
                    h.position.z,
                );
                let delta = camera.translation() - point;
                let rotation = Quat::from_rotation_y(delta.x.atan2(delta.z));
                draw(
                    h.id + (1_u64 << (62 + j)),
                    point,
                    Vec3::new(3.2, height, 1.),
                    3,
                    11,
                    rotation,
                );
            }
            continue;
        }
        let color = match h.kind {
            HazardKind::Storm => 4,
            HazardKind::Fire => 2,
            HazardKind::Frost => 3,
            HazardKind::Tar => {
                if h.burning {
                    2
                } else {
                    5
                }
            }
        };
        let shape = if h.kind == HazardKind::Tar { 4 } else { 2 };
        let scale = if shape == 0 {
            Vec3::new(
                h.radius,
                0.7 + (f.time * 6. + h.position.x).sin().abs(),
                h.radius,
            )
        } else {
            Vec3::new(h.radius, 0.08, h.radius)
        };
        draw(
            h.id,
            Vec3::new(
                h.position.x,
                if shape == 0 { 0.6 } else { 0.1 },
                h.position.z,
            ),
            scale,
            shape,
            color,
            if shape == 4 {
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
            } else {
                Quat::IDENTITY
            },
        );
    }
    for (i, v) in f.vehicles.iter().enumerate() {
        if v.shield > 0. {
            draw(
                u64::MAX - (i as u64) * 2,
                Vec3::new(v.body.position.x, 2.5, v.body.position.z),
                Vec3::new(4.5, 3.5, 4.5),
                0,
                8,
                Quat::IDENTITY,
            );
        }
        if v.frozen > 0. {
            draw(
                u64::MAX - 1 - (i as u64) * 2,
                Vec3::new(v.body.position.x, 2.5, v.body.position.z),
                Vec3::new(4.5, 4.5, 4.5),
                0,
                9,
                Quat::IDENTITY,
            );
        }
    }
    if f.onboard() {
        let p = f.player().body.position;
        let radius = if f.player().kind == Kind::Moto {
            2.8
        } else {
            4.2
        };
        draw(
            u64::MAX - 32,
            Vec3::new(p.x, 0.09, p.z),
            Vec3::new(radius, 0.08, radius),
            2,
            6,
            Quat::IDENTITY,
        );
    }
    if f.boss.cast > 0. {
        draw(
            u64::MAX - 33,
            Vec3::new(f.boss.position.x, 0.12, f.boss.position.z),
            Vec3::new(28., 0.08, 28.),
            2,
            2,
            Quat::IDENTITY,
        );
    }
    let obsolete: Vec<_> = effects
        .entities
        .keys()
        .filter(|id| !active.contains(id))
        .copied()
        .collect();
    for id in obsolete {
        if let Some(entity) = effects.entities.remove(&id) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn after_step(raid: &mut Raid, sim: &mut Simulation, dt: f32, aim: &mut Targeting) {
    let index = raid.0.role.index();
    if raid.0.role.drives() {
        raid.0.vehicles[index].body = sim.current;
    }
    let queued = std::mem::take(&mut aim.queued);
    for (slot, p) in queued {
        if let Err(error) = raid.0.cast(slot, p) {
            aim.notice = error.into();
            aim.notice_time = 3.;
        }
    }
    raid.0.tick(dt);
    if !raid.0.role.drives() {
        sim.current = raid.0.player().body;
        if raid.0.role == Role::DemoPassenger
            && !raid.0.onboard()
            && let Some(r) = raid
                .0
                .riders
                .iter()
                .find(|r| r.home == 0 && r.state != RiderState::Lost)
        {
            sim.current.position = r.position;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aimed_spell_selection_waits_for_click_without_firing() {
        let mut raid = Raid::default();
        raid.0.start();
        let mut aim = Targeting::default();
        request(1, &raid, &mut aim);
        assert_eq!(aim.slot, Some(1));
        assert!(aim.queued.is_empty());
        assert!(raid.0.shots.is_empty());
    }
    #[test]
    fn instant_spell_is_queued_without_targeting_mode() {
        let mut raid = Raid(Fight::new(Role::SiegeDriver, false));
        raid.0.start();
        let mut aim = Targeting::default();
        request(1, &raid, &mut aim);
        assert_eq!(aim.queued.len(), 1);
        assert!(aim.slot.is_none());
    }
    #[test]
    fn clicked_point_is_forwarded_unchanged_to_the_projectile() {
        let mut raid = Raid::default();
        raid.0.start();
        let mut sim = Simulation::default();
        sim.current = raid.0.player().body;
        let point = Point { x: 12., z: -15. };
        let mut aim = Targeting::default();
        aim.queued.push((1, Some(point)));
        after_step(&mut raid, &mut sim, 1. / 60., &mut aim);
        let shot = raid.0.shots.iter().find(|s| s.personal).unwrap();
        assert_eq!(shot.end, point);
        assert_ne!(shot.end, raid.0.boss.position);
        assert_eq!(raid.0.stats.shots, 1);
    }
    #[test]
    fn passenger_vehicle_movement_comes_from_allied_driver() {
        let mut raid = Raid(Fight::new(Role::SiegeGunner, false));
        raid.0.start();
        let mut sim = Simulation::default();
        sim.current = raid.0.player().body;
        let original = sim.current.position;
        after_step(&mut raid, &mut sim, 1. / 60., &mut Targeting::default());
        assert_ne!(sim.current.position, original);
        assert_eq!(sim.current.position, raid.0.player().body.position);
    }
}
