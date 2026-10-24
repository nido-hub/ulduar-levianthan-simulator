#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod config;
mod models;
mod raid;
mod scene;
mod textures;
mod ui;
mod wow;

use bevy::{
    app::AppExit,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::{
        InputSystems,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::{CursorGrabMode, CursorOptions, PresentMode},
    winit::WinitSettings,
};
use config::Configuration;
use leviathan_driving::{
    Arena, DriveInput, Orbit, Vehicle, camera_wall_fraction, follow_angle, wrap_angle,
};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Resource)]
struct Simulation {
    previous: Vehicle,
    current: Vehicle,
    signed_travel: f32,
    ticks: u64,
}
impl Default for Simulation {
    fn default() -> Self {
        Self {
            previous: Vehicle::default(),
            current: Vehicle::default(),
            signed_travel: 0.0,
            ticks: 0,
        }
    }
}
#[derive(Resource, Default)]
struct Controls(DriveInput);
#[derive(Resource)]
struct CameraControl {
    orbit: Orbit,
    center: Vec3,
    free_look_delay: f32,
    grabbed: bool,
    cursor_before: Option<Vec2>,
    initialized: bool,
}
#[derive(Resource)]
struct RunState {
    paused: bool,
    help: bool,
    debug: bool,
    status: String,
}
#[derive(Resource)]
struct SmokeTest {
    enabled: bool,
    menu: bool,
    menu_opened_at: Option<u32>,
    expected_invert: bool,
    requested: bool,
    captured: bool,
    captured_at: f64,
}
#[derive(Component)]
struct Player;
#[derive(Component)]
struct FollowCamera;
#[derive(Component)]
struct Wheel(f32);

fn output_directory() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf()
}

fn main() {
    let menu_test = std::env::args().any(|argument| argument == "--smoke-menu");
    let art_preview = std::env::args().any(|argument| argument == "--art-preview");
    let smoke = std::env::args().any(|a| a == "--smoke-combat")
        || art_preview
        || menu_test
        || std::env::args()
            .any(|argument| argument == "--smoke-test" || argument == "--smoke-fallback");
    std::panic::set_hook(Box::new(|panic| {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_directory().join("panic.log"))
        {
            let _ = writeln!(file, "{panic}");
        }
    }));
    let config = Configuration::load();
    let expected_invert = !config.settings.camera.invert_y;
    let camera = &config.settings.camera;
    let control = CameraControl {
        orbit: Orbit {
            yaw: 0.0,
            pitch: camera.pitch_degrees.to_radians(),
            distance: camera.distance,
        },
        center: Vec3::new(0.0, 2.5, 34.0),
        free_look_delay: 0.0,
        grabbed: false,
        cursor_before: None,
        initialized: false,
    };
    let video = &config.settings.video;
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: models::asset_directory().to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Leviathan Rust 0.3 - Ulduar, boss et vehicules".into(),
                        resolution: (video.width, video.height).into(),
                        present_mode: if video.vsync {
                            PresentMode::AutoVsync
                        } else {
                            PresentMode::AutoNoVsync
                        },
                        visible: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(WinitSettings::continuous())
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(ClearColor(Color::srgb(0.075, 0.08, 0.13)))
        .insert_resource(config)
        .insert_resource(control)
        .init_resource::<Simulation>()
        .init_resource::<Controls>()
        .init_resource::<models::ModelStatus>()
        .init_resource::<raid::Raid>()
        .init_resource::<raid::Targeting>()
        .init_resource::<raid::CombatValidation>()
        .insert_resource(RunState {
            paused: false,
            help: !smoke,
            debug: false,
            status: "Conduite libre. Aucun combat dans ce premier essai.".into(),
        })
        .insert_resource(SmokeTest {
            enabled: smoke,
            menu: menu_test,
            menu_opened_at: None,
            expected_invert,
            requested: false,
            captured: false,
            captured_at: 0.0,
        })
        .add_message::<ui::UiAction>()
        .add_systems(
            Startup,
            (
                scene::setup,
                ui::setup,
                wow::load,
                models::load,
                raid::setup_effects,
                raid::initialize,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (models::bake, models::check_load, wow::check_load).chain(),
        )
        .add_systems(
            PreUpdate,
            (raid::commands, gather_controls)
                .chain()
                .after(InputSystems),
        )
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (
                ui::actions,
                render_vehicle,
                update_camera,
                ui::update_plates,
                raid::targeting,
                raid::render,
                wow::animate,
                ui::update,
                ui::update_hud,
                ui::button_style,
                screenshot_keys,
                raid::smoke_combat,
                smoke_test,
            )
                .chain(),
        )
        .run();
}

#[allow(clippy::too_many_arguments)]
fn gather_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time<Real>>,
    config: Res<Configuration>,
    mut controls: ResMut<Controls>,
    mut camera: ResMut<CameraControl>,
    mut sim: ResMut<Simulation>,
    mut state: ResMut<RunState>,
    mut window: Single<(&mut Window, &mut CursorOptions)>,
    smoke: Res<SmokeTest>,
    raid: Res<raid::Raid>,
    mut aim: ResMut<raid::Targeting>,
) {
    let (window, cursor) = &mut *window;
    if !smoke.enabled && !window.focused {
        state.paused = true;
        state.status = "Pause : fenetre inactive.".into();
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        if aim.slot.is_some() {
            aim.slot = None;
        } else {
            state.paused = !state.paused;
        }
    }
    if keyboard.just_pressed(KeyCode::F1) {
        state.help = !state.help;
    }
    if keyboard.just_pressed(KeyCode::F4) {
        state.debug = !state.debug;
    }
    if keyboard.just_pressed(KeyCode::Home) {
        camera.orbit.yaw = sim.current.heading;
        camera.orbit.pitch = config.settings.camera.pitch_degrees.to_radians();
        camera.orbit.distance = config.settings.camera.distance;
    }
    if state.paused {
        aim.slot = None;
        aim.queued.clear();
        controls.0 = DriveInput::default();
        sim.current.stop();
        sim.previous = sim.current;
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        camera.grabbed = false;
        if let Some(position) = camera.cursor_before.take()
            && window.focused
        {
            window.set_cursor_position(Some(position));
        }
        return;
    }
    if smoke.enabled {
        let t = sim.ticks as f32 / 60.0;
        controls.0 = match t {
            t if t < 2.0 => DriveInput::default(),
            t if t < 4.0 => DriveInput {
                forward: 1.0,
                ..default()
            },
            t if t < 5.0 => DriveInput {
                forward: 1.0,
                turn: 0.7,
                ..default()
            },
            t if t < 6.0 => DriveInput {
                strafe: 1.0,
                ..default()
            },
            t if t < 6.5 => DriveInput {
                forward: -1.0,
                ..default()
            },
            _ => DriveInput::default(),
        };
        return;
    }
    let over_hud = window
        .cursor_position()
        .is_some_and(|p| p.y > window.height() - 175. || (p.y < 125. && p.x < 350.));
    let targeting = aim.slot.is_some();
    if targeting && mouse.just_pressed(MouseButton::Right) {
        aim.slot = None;
    }
    let left = mouse.pressed(MouseButton::Left) && !targeting && !over_hud;
    let right = mouse.pressed(MouseButton::Right);
    let grab = (left || right) && !targeting;
    if grab != camera.grabbed {
        if grab {
            camera.cursor_before = window.cursor_position();
        } else if let Some(position) = camera.cursor_before.take() {
            window.set_cursor_position(Some(position));
        }
        cursor.visible = !grab;
        cursor.grab_mode = if grab {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        camera.grabbed = grab;
    }
    if grab {
        // Do not apply the cursor reposition motion on the first captured frame.
        if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
            camera.orbit.drag(
                motion.delta.x,
                motion.delta.y,
                config.settings.camera.sensitivity,
                config.settings.camera.invert_y,
            );
        }
        camera.free_look_delay = 0.5;
    } else {
        camera.free_look_delay = (camera.free_look_delay - time.delta_secs()).max(0.0);
    }
    let lines = scroll.delta.y
        / if scroll.unit == MouseScrollUnit::Pixel {
            100.0
        } else {
            1.0
        };
    camera.orbit.zoom(lines);
    let keys = config
        .settings
        .bindings
        .resolve()
        .expect("bindings validated at load");
    let mut input = DriveInput {
        forward: u8::from(
            keyboard.pressed(keys[0]) || keyboard.pressed(KeyCode::ArrowUp) || (left && right),
        ) as f32
            - u8::from(keyboard.pressed(keys[1]) || keyboard.pressed(KeyCode::ArrowDown)) as f32,
        strafe: 0.0,
        turn: u8::from(keyboard.pressed(keys[3]) || keyboard.pressed(KeyCode::ArrowRight)) as f32
            - u8::from(keyboard.pressed(keys[2]) || keyboard.pressed(KeyCode::ArrowLeft)) as f32,
        heading_override: None,
    };
    if right {
        camera.orbit.yaw = wrap_angle(
            camera.orbit.yaw
                + input.turn
                    * config.settings.driving.turn_degrees.to_radians()
                    * time.delta_secs().min(0.05),
        );
        input.turn = 0.0;
        input.heading_override = Some(camera.orbit.yaw);
    }
    controls.0 =
        leviathan_driving::vehicle_controls(input.forward, input.turn, input.heading_override);
    if !raid.0.role.drives()
        || !raid.0.onboard()
        || raid.0.player().frozen > 0.
        || raid.0.player().hp <= 0.
        || matches!(
            raid.0.outcome,
            leviathan_combat::Outcome::Victory | leviathan_combat::Outcome::Defeat
        )
    {
        controls.0 = DriveInput::default();
    }
}

fn simulate(
    time: Res<Time<Fixed>>,
    controls: Res<Controls>,
    config: Res<Configuration>,
    state: Res<RunState>,
    mut sim: ResMut<Simulation>,
    mut raid: ResMut<raid::Raid>,
    mut aim: ResMut<raid::Targeting>,
) {
    if state.paused {
        return;
    }
    sim.previous = sim.current;
    let mut driving = config.settings.driving.physics();
    driving.strafe_factor = 0.;
    driving.speed *= match raid.0.player().kind {
        leviathan_combat::Kind::Moto => 1.5,
        leviathan_combat::Kind::Siege => 1.08,
        _ => 1.,
    } * if raid.0.player().boost > 0. { 1.7 } else { 1. };
    let control = if raid.0.player().frozen > 0. || raid.0.player().hp <= 0. {
        DriveInput::default()
    } else {
        controls.0
    };
    sim.current
        .step_vehicle(control, driving, Arena::default(), time.delta_secs());
    let current = sim.current;
    let projected =
        current.velocity.x * current.heading.sin() - current.velocity.z * current.heading.cos();
    sim.signed_travel +=
        current.velocity.length() * if projected < -0.1 { -1.0 } else { 1.0 } * time.delta_secs();
    sim.ticks += 1;
    raid::after_step(&mut raid, &mut sim, time.delta_secs(), &mut aim);
}

fn render_vehicle(
    time: Res<Time<Fixed>>,
    sim: Res<Simulation>,
    mut player: Single<&mut Transform, With<Player>>,
    mut wheels: Query<(Entity, &mut Transform, &Wheel), Without<Player>>,
    raid: Res<raid::Raid>,
    parents: Query<&ChildOf>,
    bodies: Query<&raid::RaidBody>,
    bosses: Query<(), With<raid::BossBody>>,
) {
    let alpha = time.overstep_fraction();
    player.translation = Vec3::new(
        sim.previous.position.x + (sim.current.position.x - sim.previous.position.x) * alpha,
        if raid.0.role == leviathan_combat::Role::DemoPassenger
            && !raid.0.onboard()
            && raid
                .0
                .riders
                .iter()
                .any(|r| r.home == 0 && r.state == leviathan_combat::RiderState::Turret)
        {
            7.5
        } else {
            0.0
        },
        sim.previous.position.z + (sim.current.position.z - sim.previous.position.z) * alpha,
    );
    let heading =
        sim.previous.heading + wrap_angle(sim.current.heading - sim.previous.heading) * alpha;
    player.rotation = Quat::from_rotation_y(-heading);
    for (entity, mut transform, wheel) in &mut wheels {
        let mut travel = if raid.0.role.drives() {
            sim.signed_travel
        } else {
            raid.0.player().body.distance
        };
        let mut ancestor = entity;
        for _ in 0..12 {
            if let Ok(body) = bodies.get(ancestor) {
                travel = raid.0.vehicles[body.0].body.distance;
                break;
            }
            if bosses.get(ancestor).is_ok() {
                travel = raid.0.boss.travelled;
                break;
            }
            let Ok(parent) = parents.get(ancestor) else {
                break;
            };
            ancestor = parent.parent();
        }
        transform.rotation = Quat::from_rotation_z(-travel / wheel.0);
    }
}

fn update_camera(
    time: Res<Time<Real>>,
    config: Res<Configuration>,
    sim: Res<Simulation>,
    state: Res<RunState>,
    controls: Res<Controls>,
    player: Single<&Transform, (With<Player>, Without<FollowCamera>)>,
    mut camera_transform: Single<
        (&mut Transform, &mut Projection),
        (With<FollowCamera>, Without<Player>),
    >,
    mut control: ResMut<CameraControl>,
) {
    let dt = time.delta_secs().min(0.05);
    if !state.paused
        && !control.grabbed
        && control.free_look_delay <= 0.0
        && config.settings.camera.auto_center
        && (sim.current.velocity.length() > 0.2 || controls.0.turn.abs() > 0.01)
    {
        control.orbit.yaw = follow_angle(control.orbit.yaw, sim.current.heading, 5.0, dt);
    }
    let target = player.translation + Vec3::Y * 2.6;
    if !control.initialized {
        control.center = target;
        control.initialized = true;
    }
    control.center = control.center.lerp(
        target,
        1.0 - (-config.settings.camera.follow_response * dt).exp(),
    );
    let desired = control.center + Vec3::from_array(control.orbit.offset());
    let fraction = camera_wall_fraction(control.center.to_array(), desired.to_array(), 58.0);
    let position = control.center.lerp(desired, fraction);
    let (transform, projection) = &mut *camera_transform;
    **transform = Transform::from_translation(position).looking_at(control.center, Vec3::Y);
    if let Projection::Perspective(perspective) = &mut **projection {
        perspective.fov = config.settings.camera.fov_degrees.to_radians();
    }
}

fn capture(commands: &mut Commands, path: PathBuf) {
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_capture: On<ScreenshotCaptured>,
             mut smoke: ResMut<SmokeTest>,
             time: Res<Time<Real>>| {
                if smoke.enabled {
                    smoke.captured = true;
                    smoke.captured_at = time.elapsed_secs_f64();
                }
            },
        );
}

fn screenshot_keys(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<RunState>,
) {
    if keys.just_pressed(KeyCode::F12) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = output_directory().join(format!("capture-{stamp}.png"));
        state.status = format!("Capture : {}", path.display());
        capture(&mut commands, path);
    }
}

fn smoke_test(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut smoke: ResMut<SmokeTest>,
    mut window: Single<&mut Window>,
    mut frames: Local<u32>,
    sim: Res<Simulation>,
    mut exit: MessageWriter<AppExit>,
    diagnostics: Res<DiagnosticsStore>,
    mut state: ResMut<RunState>,
    config: Res<Configuration>,
    mut actions: MessageWriter<ui::UiAction>,
    model: Res<models::ModelStatus>,
    mut camera: ResMut<CameraControl>,
    raid: Res<raid::Raid>,
    combat_validation: Res<raid::CombatValidation>,
) {
    *frames += 1;
    if *frames == 3 {
        window.visible = true;
    }
    if !smoke.enabled {
        return;
    }
    if sim.ticks > 450 && std::env::args().any(|a| a == "--art-preview") {
        camera.orbit.yaw = sim.current.heading - 0.85;
        camera.orbit.pitch = 0.34;
        camera.orbit.distance = 15.5;
        camera.free_look_delay = 1.0;
    }
    let combat = std::env::args().any(|a| a == "--smoke-combat");
    if !smoke.requested
        && sim.ticks
            > if combat {
                raid::combat_seconds() * 60
            } else {
                540
            }
        && *frames > 180
    {
        if smoke.menu && smoke.menu_opened_at.is_none() {
            state.paused = true;
            smoke.menu_opened_at = Some(*frames);
            actions.write(ui::UiAction::Invert);
            actions.write(ui::UiAction::SpeedUp);
        }
        if !smoke.menu
            || smoke
                .menu_opened_at
                .is_some_and(|frame| *frames > frame + 4)
        {
            smoke.requested = true;
            capture(
                &mut commands,
                output_directory().join(if smoke.menu {
                    "validation-menu.png"
                } else if combat {
                    "validation-combat.png"
                } else if std::env::args().any(|a| a == "--smoke-fallback") {
                    "validation-fallback.png"
                } else if std::env::args().any(|a| a == "--art-preview") {
                    "validation-art.png"
                } else {
                    "validation.png"
                }),
            );
        }
    }
    if smoke.captured && time.elapsed_secs_f64() - smoke.captured_at > 1.0 {
        let model_valid = if std::env::args().any(|a| a == "--smoke-fallback") {
            model.fallback && !model.loaded
        } else if model.converted {
            model.scenes_loaded == 10 && model.animation_players == 10
        } else {
            model.loaded && model.animated_wheels == 4
        };
        let valid = model_valid
            && (!combat
                || (combat_validation.casts > 0
                    && combat_validation.projection_ok
                    && combat_validation.air_projection_ok
                    && raid.0.time > 15.))
            && (!smoke.menu
                || (state.paused && config.settings.camera.invert_y == smoke.expected_invert))
            && sim.current.position.x.is_finite()
            && sim.current.position.z.is_finite()
            && sim.current.position.x.abs() <= 53.5
            && sim.current.position.z.abs() <= 53.5
            && (combat || sim.current.distance > 10.0);
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.smoothed())
            .unwrap_or_default();
        let stem = if smoke.menu {
            "validation-menu"
        } else if combat {
            "validation-combat"
        } else if std::env::args().any(|a| a == "--smoke-fallback") {
            "validation-fallback"
        } else if std::env::args().any(|a| a == "--art-preview") {
            "validation-art"
        } else {
            "validation"
        };
        let report = format!(
            "native_smoke={}\nversion={}\nmodel_glb_loaded={}\nmodel_animated_wheels={}\nmodel_fallback={}\nframes={}\nticks={}\ndistance={:.3}\nposition={:.3},{:.3}\nfps_smoothed={:.1}\nscreenshot={stem}.png\nrole={:?}\ncombat_time={:.1}\ncombat_casts={}\nprojection_ok={}\nair_projection_ok={}\npersonal_shots={}\nimpacts={}\nmisses={}\nshutdowns={}\nactive_hazards={}\nactive_adds={}\n",
            if valid { "PASS" } else { "FAIL" },
            env!("CARGO_PKG_VERSION"),
            model.loaded,
            model.animated_wheels,
            model.fallback,
            *frames,
            sim.ticks,
            sim.current.distance,
            sim.current.position.x,
            sim.current.position.z,
            fps,
            raid.0.role,
            raid.0.time,
            combat_validation.casts,
            combat_validation.projection_ok,
            combat_validation.air_projection_ok,
            raid.0.stats.shots,
            raid.0.stats.hits,
            raid.0.stats.misses,
            raid.0.stats.shutdowns,
            raid.0.hazards.len(),
            raid.0.adds.len()
        );
        let report = format!(
            "{report}wow_models={}\nwow_scenes_loaded={}\nwow_animation_players={}\nasset_directory={}\nplayer_cooldowns={:?}\n",
            model.converted,
            model.scenes_loaded,
            model.animation_players,
            models::asset_directory().display(),
            raid.0.player().cooldowns
        );
        let _ = std::fs::write(output_directory().join(format!("{stem}.txt")), report);
        exit.write(if valid {
            AppExit::Success
        } else {
            AppExit::error()
        });
    } else if time.elapsed_secs()
        > if combat {
            raid::combat_seconds() as f32 + 45.
        } else {
            60.
        }
    {
        let _ = std::fs::write(
            output_directory().join(if smoke.menu {
                "validation-menu.txt"
            } else if combat {
                "validation-combat.txt"
            } else if std::env::args().any(|a| a == "--smoke-fallback") {
                "validation-fallback.txt"
            } else if std::env::args().any(|a| a == "--art-preview") {
                "validation-art.txt"
            } else {
                "validation.txt"
            }),
            "native_smoke=FAIL\nreason=screenshot_timeout\n",
        );
        exit.write(AppExit::error());
    }
}
