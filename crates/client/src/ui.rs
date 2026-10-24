use crate::{CameraControl, RunState, Simulation, config::Configuration};
mod wow_hud;
use bevy::{
    app::AppExit,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, Button},
    window::PresentMode,
};
pub use wow_hud::update as update_hud;

#[derive(Message, Clone, Copy, Debug)]
pub enum UiAction {
    StartFight,
    RestartFight,
    CycleRole,
    ToggleTowers,
    Ability(usize),
    Resume,
    Save,
    ResetVehicle,
    Quit,
    SpeedDown,
    SpeedUp,
    TurnDown,
    TurnUp,
    SensitivityDown,
    SensitivityUp,
    FovDown,
    FovUp,
    Response,
    AutoCenter,
    Invert,
    Shadows,
    Vsync,
    Defaults,
}
#[derive(Component)]
pub(super) enum DynamicText {
    Role,
    Boss,
    CombatLog,
    Aim,
    CombatStats,
    Status,
    Settings,
    Notice,
    Diagnostic,
    Button(UiAction),
}
#[derive(Component)]
pub(super) struct MenuPanel;
#[derive(Component)]
pub(super) struct HelpPanel;
#[derive(Component)]
pub(super) struct DiagnosticPanel;
#[derive(Component)]
pub(super) struct UnitPlate(usize);

#[derive(Clone)]
struct FontStyle {
    source: Option<Handle<Font>>,
}
impl FontStyle {
    fn at(&self, size: f32) -> TextFont {
        let mut font = TextFont {
            font_size: FontSize::Px(size),
            ..default()
        };
        if let Some(source) = &self.source {
            font.font = source.clone().into();
        }
        font
    }
}
fn gold() -> Color {
    Color::srgb(0.91, 0.76, 0.47)
}
fn ink() -> Color {
    Color::srgba(0.055, 0.05, 0.065, 0.9)
}
fn text(value: impl Into<String>, font: &FontStyle, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        font.at(size),
        TextColor(color),
        TextShadow::default(),
    )
}
fn label(action: UiAction, config: &Configuration) -> String {
    use UiAction::*;
    match action {
        StartFight => "Entrée · Démarrer".into(),
        RestartFight => "F5 · Recommencer".into(),
        CycleRole => "F2 · Rôle suivant".into(),
        ToggleTowers => "F7 · Tours oui/non".into(),
        Ability(index) => format!("{}", index + 1),
        Resume => "Reprendre".into(),
        Save => "Enregistrer".into(),
        ResetVehicle => "Replacer le véhicule".into(),
        Quit => "Quitter".into(),
        SpeedDown => "Vitesse −".into(),
        SpeedUp => "Vitesse +".into(),
        TurnDown => "Rotation −".into(),
        TurnUp => "Rotation +".into(),
        SensitivityDown => "Souris −".into(),
        SensitivityUp => "Souris +".into(),
        FovDown => "Champ de vision −".into(),
        FovUp => "Champ de vision +".into(),
        Response => format!(
            "Réponse : {}",
            if config.settings.driving.response_time == 0.0 {
                "directe"
            } else {
                "adoucie"
            }
        ),
        AutoCenter => format!(
            "Retour caméra : {}",
            if config.settings.camera.auto_center {
                "oui"
            } else {
                "non"
            }
        ),
        Invert => format!(
            "Axe vertical : {}",
            if config.settings.camera.invert_y {
                "inversé"
            } else {
                "normal"
            }
        ),
        Shadows => format!(
            "Ombres : {}",
            if config.settings.video.shadows {
                "oui"
            } else {
                "non"
            }
        ),
        Vsync => format!(
            "VSync : {}",
            if config.settings.video.vsync {
                "oui"
            } else {
                "non"
            }
        ),
        Defaults => "Réglages par défaut".into(),
    }
}
fn button(
    parent: &mut ChildSpawnerCommands,
    action: UiAction,
    font: &FontStyle,
    config: &Configuration,
    width: f32,
) {
    parent
        .spawn((
            Button,
            Hovered::default(),
            Node {
                width: percent(width),
                height: px(37),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.5, 0.43, 0.3)),
            BackgroundColor(Color::srgb(0.14, 0.12, 0.12)),
        ))
        .observe(
            move |_event: On<Activate>, mut writer: MessageWriter<UiAction>| {
                writer.write(action);
            },
        )
        .with_child((
            text(label(action, config), font, 13.0, gold()),
            DynamicText::Button(action),
        ));
}

pub fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    config: Res<Configuration>,
    assets: Res<AssetServer>,
) {
    // Prefer the copied client font; read the Windows fallback in place without redistributing it.
    let source = if crate::models::asset_directory()
        .join("wow/ui/frizqt.ttf")
        .is_file()
    {
        Some(assets.load("wow/ui/frizqt.ttf"))
    } else {
        std::fs::read("C:/Windows/Fonts/segoeui.ttf")
            .ok()
            .map(Font::from_bytes)
            .map(|font| fonts.add(font))
    };
    let font = FontStyle { source };
    for index in 0..7 {
        commands.spawn((
            UnitPlate(index),
            Node {
                position_type: PositionType::Absolute,
                width: px(190),
                display: Display::None,
                ..default()
            },
            text(
                "",
                &font,
                12.,
                if index == 6 {
                    Color::srgb(1., 0.45, 0.3)
                } else {
                    Color::srgb(0.75, 0.9, 0.6)
                },
            ),
            GlobalZIndex(1),
            Pickable::IGNORE,
        ));
    }
    wow_hud::spawn(&mut commands, &font, &assets);
    commands.spawn((
        HelpPanel,
        Node { position_type: PositionType::Absolute, right: px(20), top: px(190), width: px(300), padding: UiRect::all(px(16)), border: UiRect::all(px(1)), flex_direction: FlexDirection::Column, row_gap: px(9), ..default() },
        BackgroundColor(ink().with_alpha(0.82)), BorderColor::all(gold().with_alpha(0.4)), GlobalZIndex(3),
    )).with_children(|p| {
        p.spawn(text("VISÉE MANUELLE", &font, 15.0, gold()));
        p.spawn(text("1–5 : choisir un sort.\nBoules : vise au sol puis clique à gauche.\nRoquettes : vise les caisses en l'air.\nVert : portée valide ; rouge : invalide.\nClic droit : annuler la visée.\nLe boss peut bouger pendant le vol : anticipe !\nÉlectrochoc et bélier : cône devant le véhicule.", &font, 14.0, Color::srgb(0.84, 0.82, 0.77)));
        p.spawn(text("F2 change le poste avant le combat.\nPassager : conducteur contrôlé par l'IA.\nF1 masque cette aide.", &font, 12.0, Color::srgb(0.62, 0.64, 0.67)));
        p.spawn(text("ZQSD / WASD ou flèches : conduire.\nClic gauche maintenu : regarder autour.\nClic droit maintenu : diriger.\nDeux boutons maintenus : avancer.\nMolette : zoom · Début : recentrer.", &font, 12.0, Color::srgb(0.78,0.75,0.68)));
    });
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(190),
            left: percent(30),
            right: px(20),
            ..default()
        },
        text("", &font, 14., gold()),
        DynamicText::Aim,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(175),
            left: px(20),
            width: percent(25),
            ..default()
        },
        BackgroundColor(ink().with_alpha(0.7)),
        text("", &font, 11., Color::srgb(0.8, 0.85, 0.65)),
        DynamicText::CombatLog,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(175),
            right: px(20),
            width: percent(22),
            ..default()
        },
        BackgroundColor(ink().with_alpha(0.7)),
        text("", &font, 12., Color::WHITE),
        DynamicText::CombatStats,
    ));
    commands.spawn((
        DynamicText::Diagnostic,
        DiagnosticPanel,
        text("", &font, 12.0, Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(118),
            left: px(22),
            display: Display::None,
            ..default()
        },
    ));
    commands.spawn((
        MenuPanel,
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), display: Display::None, align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
        GlobalZIndex(10), BackgroundColor(Color::srgba(0.02, 0.025, 0.04, 0.83)),
    )).with_children(|p| {
        p.spawn((
            Node { width: px(730), max_width: percent(96), padding: UiRect::all(px(22)), border: UiRect::all(px(2)), flex_direction: FlexDirection::Column, row_gap: px(10), ..default() },
            BackgroundColor(Color::srgb(0.075, 0.065, 0.075)), BorderColor::all(gold().with_alpha(0.7)),
            if crate::models::asset_directory().join("wow/ui/dialog.png").is_file() { ImageNode::new(assets.load("wow/ui/dialog.png")) } else { ImageNode { color: Color::NONE, ..default() } },
        )).with_children(|p| {
            p.spawn(text("Léviathan · Combat et véhicules", &font, 25.0, gold()));
            p.spawn(text("PAUSE  ·  Les changements s'appliquent immédiatement.\nEnregistrer conserve les valeurs dans settings.toml ; rien n'est écrit automatiquement.", &font, 13.0, Color::srgb(0.79, 0.77, 0.73)));
            p.spawn((text("", &font, 15.0, Color::WHITE), DynamicText::Settings));
            for actions in [
                vec![UiAction::CycleRole, UiAction::StartFight, UiAction::RestartFight, UiAction::ToggleTowers],
                vec![UiAction::SpeedDown, UiAction::SpeedUp, UiAction::TurnDown, UiAction::TurnUp],
                vec![UiAction::SensitivityDown, UiAction::SensitivityUp, UiAction::FovDown, UiAction::FovUp],
                vec![UiAction::Response, UiAction::AutoCenter, UiAction::Invert],
                vec![UiAction::Shadows, UiAction::Vsync, UiAction::Defaults],
                vec![UiAction::Resume, UiAction::Save, UiAction::ResetVehicle, UiAction::Quit],
            ] {
                p.spawn(Node { width: percent(100), column_gap: px(6), ..default() }).with_children(|row| {
                    let width = 100.0 / actions.len() as f32;
                    for action in actions { button(row, action, &font, &config, width); }
                });
            }
            p.spawn((text("", &font, 12.0, Color::srgb(0.65, 0.77, 0.65)), DynamicText::Notice));
            p.spawn(text("Clic droit : orientation uniquement, aucun strafe. Les sorts projetés exigent un point d'impact.\nF2 et F7 avant le pull ; F5 remet toute la tentative à zéro. HM 10 joueurs / ilvl 215.\nSur le boss : 1 ou Attaquer les tourelles, puis surcharge automatique. Fidélité exacte non certifiée.", &font, 12.0, Color::srgb(0.64, 0.62, 0.61)));
        });
    });
}

pub fn actions(
    mut reader: MessageReader<UiAction>,
    mut config: ResMut<Configuration>,
    mut state: ResMut<RunState>,
    mut sim: ResMut<Simulation>,
    mut camera: ResMut<CameraControl>,
    mut exit: MessageWriter<AppExit>,
    mut light: Single<&mut DirectionalLight>,
    mut window: Single<&mut Window>,
    mut raid: ResMut<crate::raid::Raid>,
    mut aim: ResMut<crate::raid::Targeting>,
) {
    for action in reader.read() {
        use UiAction::*;
        match action {
            StartFight => {
                raid.0.start();
                state.paused = false;
            }
            RestartFight => {
                let role = raid.0.role;
                let towers = raid.0.towers;
                crate::raid::reset(role, towers, &mut raid, &mut sim, &mut aim, &mut camera);
                state.paused = false;
            }
            CycleRole => {
                if raid.0.outcome != leviathan_combat::Outcome::Fighting {
                    let index = leviathan_combat::Role::ALL
                        .iter()
                        .position(|r| *r == raid.0.role)
                        .unwrap();
                    let towers = raid.0.towers;
                    crate::raid::reset(
                        leviathan_combat::Role::ALL[(index + 1) % 5],
                        towers,
                        &mut raid,
                        &mut sim,
                        &mut aim,
                        &mut camera,
                    );
                }
            }
            ToggleTowers => {
                if raid.0.outcome == leviathan_combat::Outcome::Ready {
                    raid.0.towers = !raid.0.towers;
                }
            }
            Ability(slot) => {
                if !state.paused {
                    crate::raid::request(*slot, &raid, &mut aim);
                }
            }
            Resume => state.paused = false,
            Save => config.save(),
            ResetVehicle => {
                if raid.0.outcome == leviathan_combat::Outcome::Ready {
                    let role = raid.0.role;
                    let towers = raid.0.towers;
                    crate::raid::reset(role, towers, &mut raid, &mut sim, &mut aim, &mut camera);
                }
            }
            Quit => {
                exit.write(AppExit::Success);
            }
            SpeedDown => config.settings.driving.speed -= 0.5,
            SpeedUp => config.settings.driving.speed += 0.5,
            TurnDown => config.settings.driving.turn_degrees -= 10.0,
            TurnUp => config.settings.driving.turn_degrees += 10.0,
            SensitivityDown => config.settings.camera.sensitivity -= 0.0003,
            SensitivityUp => config.settings.camera.sensitivity += 0.0003,
            FovDown => config.settings.camera.fov_degrees -= 5.0,
            FovUp => config.settings.camera.fov_degrees += 5.0,
            Response => {
                config.settings.driving.response_time =
                    if config.settings.driving.response_time == 0.0 {
                        0.12
                    } else {
                        0.0
                    }
            }
            AutoCenter => config.settings.camera.auto_center = !config.settings.camera.auto_center,
            Invert => config.settings.camera.invert_y = !config.settings.camera.invert_y,
            Shadows => config.settings.video.shadows = !config.settings.video.shadows,
            Vsync => config.settings.video.vsync = !config.settings.video.vsync,
            Defaults => {
                config.settings = Default::default();
                camera.orbit.pitch = config.settings.camera.pitch_degrees.to_radians();
                camera.orbit.distance = config.settings.camera.distance;
                camera.orbit.yaw = sim.current.heading;
            }
        }
        config.settings.sanitize();
        light.shadow_maps_enabled = config.settings.video.shadows;
        window.present_mode = if config.settings.video.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
    }
}

pub fn update(
    state: Res<RunState>,
    config: Res<Configuration>,
    sim: Res<Simulation>,
    camera: Res<CameraControl>,
    diagnostics: Res<DiagnosticsStore>,
    model: Res<crate::models::ModelStatus>,
    raid: Res<crate::raid::Raid>,
    aim: Res<crate::raid::Targeting>,
    mut text_query: Query<(&DynamicText, &mut Text)>,
    mut nodes: Query<(
        &mut Node,
        Has<MenuPanel>,
        Has<HelpPanel>,
        Has<DiagnosticPanel>,
    )>,
) {
    let speed = sim.current.velocity.length();
    for (kind, mut text) in &mut text_query {
        text.0 = match kind {
            DynamicText::Role => raid.0.role.name().into(),
            DynamicText::Boss => format!(
                "{} · {} tours\n{}",
                match raid.0.outcome {
                    leviathan_combat::Outcome::Ready => "Entrée : lancer",
                    leviathan_combat::Outcome::Fighting => "Combat en cours",
                    leviathan_combat::Outcome::Victory => "VICTOIRE",
                    leviathan_combat::Outcome::Defeat => "DÉFAITE",
                },
                if raid.0.towers { "4" } else { "0" },
                if raid.0.boss.shutdown > 0. {
                    format!("Surcharge {:.1} s", raid.0.boss.shutdown)
                } else if raid.0.boss.cast > 0. {
                    format!("DÉCHARGES DE FLAMMES {:.1} s", raid.0.boss.cast)
                } else {
                    format!(
                        "Poursuite : {} {} · {:.0} s · Vitesse +{} % · ilvl 215",
                        raid.0.vehicles[raid.0.boss.target].kind.name(),
                        raid.0.boss.target % 2 + 1,
                        raid.0.boss.pursuit,
                        raid.0.boss.speed_stacks as u32 * 5
                    )
                }
            ),
            DynamicText::CombatLog => raid.0.log.join("\n"),
            DynamicText::Aim => {
                if aim.slot.is_some() {
                    "VISÉE : clic gauche au sol · clic droit / Échap : annuler".into()
                } else if aim.notice_time > 0. {
                    aim.notice.clone()
                } else {
                    String::new()
                }
            }
            DynamicText::CombatStats => format!(
                "PV véhicule : {:.0} % · Ressource : {:.0}\nPyrite : {} / 10 · {:.1} s\nTirs : {} · Impacts : {} · Ratés : {}\nInterruptions : {} · Décharges ratées : {}\nSecours : {} · Dégâts évitables : {:.0}\n{}",
                raid.0.health_fraction() * 100.,
                raid.0.resource(),
                raid.0.player().stacks,
                raid.0.player().stack_time.max(0.),
                raid.0.stats.shots,
                raid.0.stats.hits,
                raid.0.stats.misses,
                raid.0.stats.interrupts,
                raid.0.stats.vents_missed,
                raid.0.stats.rescued,
                raid.0.stats.avoidable,
                if raid.0.onboard() {
                    "À bord"
                } else {
                    "Passager projeté : retour par moto"
                }
            ),
            DynamicText::Status => format!(
                "{}  ·  {:.1} u/s{}",
                if state.paused {
                    "En pause"
                } else if speed > 0.05 {
                    "En mouvement"
                } else {
                    "À l'arrêt"
                },
                speed,
                if sim.current.touching_wall {
                    "  ·  Bord de cour"
                } else {
                    ""
                }
            ),
            DynamicText::Settings => format!(
                "Vitesse : {:.1} u/s   |   Rotation : {:.0}°/s\nSouris : {:.4} rad/pixel   |   Champ de vision : {:.0}°\nZoom actuel : {:.1}   |   Réponse : {}\nÉcran : {} × {} (résolution au prochain lancement)",
                config.settings.driving.speed,
                config.settings.driving.turn_degrees,
                config.settings.camera.sensitivity,
                config.settings.camera.fov_degrees,
                camera.orbit.distance,
                if config.settings.driving.response_time == 0.0 {
                    "directe"
                } else {
                    "adoucie (0,12 s)"
                },
                config.settings.video.width,
                config.settings.video.height
            ),
            DynamicText::Notice => config.notice.clone(),
            DynamicText::Button(UiAction::Ability(slot)) => {
                if let Some(spell) = raid.0.active_spells().get(*slot) {
                    let cd = if raid.0.onboard() {
                        raid.0.cooldowns()[*slot]
                    } else {
                        0.
                    };
                    format!(
                        "{} · {}{}\n{} · coût {:.0}",
                        slot + 1,
                        spell.name,
                        if cd > 0. {
                            format!(" [{cd:.1}s]")
                        } else {
                            String::new()
                        },
                        if crate::raid::air_target(raid.0.role, *slot) {
                            "Visée aérienne"
                        } else if spell.aimed {
                            "Visée au sol"
                        } else {
                            "Devant / instantané"
                        },
                        spell.cost
                    )
                } else {
                    "—".into()
                }
            }
            DynamicText::Button(action) => label(*action, &config),
            DynamicText::Diagnostic => {
                let fps = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                    .unwrap_or_default();
                format!(
                    "{fps:.0} FPS · 60 ticks/s · {:.1} u parcourues\nPosition {:.1} / {:.1} · Cap {:.0}°\nModèle : {}\n{}",
                    sim.current.distance,
                    sim.current.position.x,
                    sim.current.position.z,
                    sim.current.heading.to_degrees(),
                    if model.converted {
                        "WoW GLB · textures copiées · animations"
                    } else if model.loaded {
                        "GLB original texturé"
                    } else if model.fallback {
                        model.error.as_str()
                    } else {
                        "Chargement GLB…"
                    },
                    state.status
                )
            }
        };
    }
    for (mut node, menu, help, debug) in &mut nodes {
        if menu {
            node.display = if state.paused {
                Display::Flex
            } else {
                Display::None
            };
        }
        if help {
            node.display = if state.help && !state.paused {
                Display::Flex
            } else {
                Display::None
            };
        }
        if debug {
            node.display = if state.debug {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

pub fn button_style(
    mut query: Query<
        (
            &Hovered,
            Has<Pressed>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<wow_hud::AbilitySlot>),
    >,
) {
    for (hovered, pressed, mut background, mut border) in &mut query {
        background.0 = if pressed {
            Color::srgb(0.35, 0.23, 0.13)
        } else if hovered.get() {
            Color::srgb(0.24, 0.2, 0.17)
        } else {
            Color::srgb(0.14, 0.12, 0.12)
        };
        border.set_all(gold().with_alpha(if hovered.get() { 1.0 } else { 0.5 }));
    }
}

pub fn update_plates(
    raid: Res<crate::raid::Raid>,
    models: Res<crate::models::ModelStatus>,
    camera: Single<(&Camera, &GlobalTransform), With<crate::FollowCamera>>,
    window: Single<&Window>,
    mut plates: Query<(&UnitPlate, &mut Node, &mut Text)>,
) {
    let (camera, transform) = *camera;
    for (plate, mut node, mut text) in &mut plates {
        let (position, height, hp, label) = if plate.0 == 6 {
            (
                raid.0.boss.position,
                if models.boss_loaded { 23. } else { 10. },
                raid.0.boss.hp / raid.0.boss.max_hp * 100.,
                "Léviathan des flammes".to_string(),
            )
        } else {
            let v = &raid.0.vehicles[plate.0];
            (
                v.body.position,
                match v.kind {
                    leviathan_combat::Kind::Demo => 8.,
                    leviathan_combat::Kind::Siege => 6.,
                    leviathan_combat::Kind::Moto => 4.,
                },
                v.hp / v.max_hp * 100.,
                format!(
                    "{} {}{}",
                    v.kind.name(),
                    plate.0 % 2 + 1,
                    if plate.0 == raid.0.role.index() {
                        " · assigné"
                    } else {
                        ""
                    }
                ),
            )
        };
        let visible = camera
            .world_to_viewport(transform, Vec3::new(position.x, height, position.z))
            .ok()
            .filter(|p| {
                p.x > 10.
                    && p.x < window.width() - 10.
                    && p.y > 125.
                    && p.y < window.height() - 175.
            });
        if let Some(point) = visible {
            node.display = Display::Flex;
            node.left = px(point.x - 80.);
            node.top = px(point.y - 20.);
            text.0 = format!("{label} · {hp:.0} %");
        } else {
            node.display = Display::None;
        }
    }
}
