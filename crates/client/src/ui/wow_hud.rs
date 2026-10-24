use super::{DynamicText, FontStyle, UiAction, gold, text};
use crate::raid::{Raid, Targeting};
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
};
use leviathan_combat::Role;

fn icon_id(role: Role, slot: usize) -> Option<u32> {
    let icons: &[u32] = match role {
        Role::DemoDriver => &[3760, 3779, 3762, 3768],
        Role::DemoPassenger => &[3761, 2589, 2735, 3766, 3772],
        Role::SiegeDriver => &[3763, 2102, 3774],
        Role::SiegeGunner => &[2589, 3761, 3845],
        Role::Moto => &[3769, 3770, 3773, 3771, 104],
    };
    icons.get(slot).copied()
}

fn fill_percent(value: f32, maximum: f32) -> f32 {
    if !value.is_finite() || !maximum.is_finite() || maximum <= 0. {
        return 0.;
    }
    (value / maximum * 100.).clamp(0., 100.)
}

#[derive(Component, Clone, Copy)]
pub enum HudBar {
    Health,
    Resource,
    Boss,
    Cast,
    Pyrite,
}
impl HudBar {
    fn percent(self, fight: &leviathan_combat::Fight) -> f32 {
        match self {
            Self::Health => fight.health_fraction() * 100.,
            Self::Resource => fill_percent(fight.resource(), 100.),
            Self::Boss => fill_percent(fight.boss.hp, fight.boss.max_hp),
            Self::Cast => fill_percent(fight.boss.cast, leviathan_combat::data::VENTS_DURATION),
            Self::Pyrite => fill_percent(fight.player().stacks as f32, 10.),
        }
    }
}
#[derive(Component)]
pub enum HudText {
    Bar(HudBar),
    Name(usize),
    Cooldown(usize),
    Hint,
}
#[derive(Component)]
pub struct AbilitySlot(pub usize);
#[derive(Component)]
pub struct AbilityIcon(pub usize);
#[derive(Component)]
pub struct CooldownShade(pub usize);
#[derive(Component)]
pub struct Portrait;

fn image(assets: &AssetServer, name: &str) -> ImageNode {
    if crate::models::asset_directory()
        .join(format!("wow/ui/{name}.png"))
        .is_file()
    {
        ImageNode::new(assets.load(format!("wow/ui/{name}.png")))
    } else {
        ImageNode {
            color: Color::NONE,
            ..default()
        }
    }
}

fn positioned(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        top: px(top),
        width: px(width),
        height: px(height),
        ..default()
    }
}

fn bar(
    p: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    font: &FontStyle,
    kind: HudBar,
    rect: [f32; 4],
    color: Color,
) {
    p.spawn((
        positioned(rect[0], rect[1], rect[2], rect[3]),
        BackgroundColor(Color::srgb(0.035, 0.025, 0.02)),
        Pickable::IGNORE,
    ))
    .with_children(|p| {
        let mut texture = image(assets, "statusbar");
        texture.color = color;
        p.spawn((
            HudBar::clone(&kind),
            texture,
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
        ));
        p.spawn((
            HudText::Bar(kind),
            text("", font, 11., Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
    });
}

pub(super) fn spawn(commands: &mut Commands, font: &FontStyle, assets: &AssetServer) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(8),
                top: px(10),
                width: px(384),
                height: px(178),
                ..default()
            },
            GlobalZIndex(2),
            Pickable::IGNORE,
        ))
        .with_children(|p| {
            p.spawn((
                image(assets, "vehicle-frame"),
                positioned(0., 0., 384., 192.),
                Pickable::IGNORE,
            ));
            p.spawn((
                Portrait,
                image(assets, "seat-demo"),
                positioned(44., 35., 89., 89.),
                Pickable::IGNORE,
            ));
            p.spawn((
                text("", font, 12., gold()),
                DynamicText::Role,
                positioned(155., 36., 176., 44.),
                Pickable::IGNORE,
            ));
            bar(
                p,
                assets,
                font,
                HudBar::Health,
                [157., 85., 170., 17.],
                Color::srgb(0.16, 0.72, 0.14),
            );
            bar(
                p,
                assets,
                font,
                HudBar::Resource,
                [157., 106., 170., 12.],
                Color::srgb(0.15, 0.55, 0.95),
            );
            p.spawn((
                text("", font, 11., Color::srgb(0.8, 0.76, 0.65)),
                DynamicText::Status,
                positioned(48., 153., 290., 20.),
                Pickable::IGNORE,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(37),
                top: px(8),
                width: px(384),
                height: px(190),
                ..default()
            },
            GlobalZIndex(2),
            Pickable::IGNORE,
        ))
        .with_children(|p| {
            p.spawn((
                image(assets, "target-frame"),
                positioned(0., 0., 384., 192.),
                Pickable::IGNORE,
            ));
            p.spawn((
                image(assets, "boss"),
                positioned(232., 33., 90., 90.),
                Pickable::IGNORE,
            ));
            p.spawn((
                text("Léviathan des flammes", font, 14., gold()),
                positioned(41., 5., 220., 24.),
                Pickable::IGNORE,
            ));
            bar(
                p,
                assets,
                font,
                HudBar::Boss,
                [45., 37., 176., 23.],
                Color::srgb(0.72, 0.09, 0.06),
            );
            bar(
                p,
                assets,
                font,
                HudBar::Cast,
                [45., 64., 176., 13.],
                Color::srgb(1., 0.5, 0.07),
            );
            bar(
                p,
                assets,
                font,
                HudBar::Pyrite,
                [45., 81., 176., 13.],
                Color::srgb(0.2, 0.65, 0.85),
            );
            p.spawn((
                text("", font, 12., gold()),
                DynamicText::Boss,
                positioned(43., 127., 330., 56.),
                Pickable::IGNORE,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(26),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(2),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px(640),
                    max_width: percent(98),
                    height: px(126),
                    padding: UiRect::horizontal(px(50)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(2),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.06, 0.055, 0.04)),
                BorderColor::all(Color::srgb(0.35, 0.3, 0.19)),
            ))
            .with_children(|p| {
                p.spawn((
                    image(assets, "dock"),
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                p.spawn((
                    image(assets, "trim"),
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(-5),
                        width: percent(100),
                        height: px(14),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
                for slot in 0..5 {
                    p.spawn((
                        AbilitySlot(slot),
                        Button,
                        Hovered::default(),
                        Node {
                            width: px(104),
                            flex_shrink: 1.,
                            height: px(110),
                            border: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.2)),
                        BorderColor::all(Color::NONE),
                    ))
                    .observe(
                        move |_event: On<Activate>, mut writer: MessageWriter<UiAction>| {
                            writer.write(UiAction::Ability(slot));
                        },
                    )
                    .with_children(|p| {
                        p.spawn((
                            AbilityIcon(slot),
                            image(assets, "icon-3760"),
                            positioned(24., 12., 56., 56.),
                            Pickable::IGNORE,
                        ));
                        p.spawn((positioned(24., 12., 56., 56.), Pickable::IGNORE))
                            .with_child((
                                CooldownShade(slot),
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: px(0),
                                    width: percent(100),
                                    height: percent(0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
                                Pickable::IGNORE,
                            ));
                        p.spawn((
                            image(assets, "slot-frame"),
                            positioned(21., 9., 62., 62.),
                            Pickable::IGNORE,
                        ));
                        p.spawn((
                            text(format!("{}", slot + 1), font, 13., Color::WHITE),
                            positioned(67., 10., 16., 18.),
                            Pickable::IGNORE,
                        ));
                        p.spawn((
                            HudText::Cooldown(slot),
                            text("", font, 22., Color::WHITE),
                            positioned(25., 29., 54., 30.),
                            TextLayout::justify(Justify::Center),
                            Pickable::IGNORE,
                        ));
                        p.spawn((
                            HudText::Name(slot),
                            text("", font, 12., gold()),
                            positioned(3., 77., 98., 32.),
                            TextLayout::justify(Justify::Center),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
        });
    commands.spawn((
        HudText::Hint,
        text("", font, 13., gold()),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(161),
            left: percent(26),
            width: percent(48),
            ..default()
        },
        TextLayout::justify(Justify::Center),
        BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.7)),
        Pickable::IGNORE,
    ));
    commands.spawn((text("Entrée : combat   ·   F2 : poste   ·   F5 : recommencer   ·   F7 : tours   ·   F1 : aide   ·   Échap : options",font,12.,Color::srgb(0.8,0.75,0.62)),
        Node { position_type: PositionType::Absolute, bottom: px(5), width: percent(100), ..default() }, TextLayout::justify(Justify::Center), Pickable::IGNORE));
}

pub fn update(
    raid: Res<Raid>,
    aim: Res<Targeting>,
    assets: Res<AssetServer>,
    mut nodes: Query<(&mut Node, Option<&HudBar>, Option<&CooldownShade>)>,
    mut labels: Query<(&HudText, &mut Text)>,
    mut icons: Query<(&AbilityIcon, &mut ImageNode), Without<Portrait>>,
    mut portraits: Query<&mut ImageNode, With<Portrait>>,
    mut buttons: Query<(
        &AbilitySlot,
        &Hovered,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    let fight = &raid.0;
    let spells = fight.active_spells();
    let mut hovered_slot = None;
    for (slot, hovered, mut border, mut background) in &mut buttons {
        if hovered.get() && slot.0 < spells.len() {
            hovered_slot = Some(slot.0);
        }
        let selected = aim.slot == Some(slot.0);
        border.set_all(if selected {
            Color::srgb(0.3, 1., 0.25)
        } else if hovered.get() {
            gold()
        } else {
            Color::NONE
        });
        background.0 = Color::srgba(
            0.02,
            0.02,
            0.02,
            if hovered.get() || selected { 0.75 } else { 0.2 },
        );
    }
    for (mut node, bar, cooldown) in &mut nodes {
        if let Some(bar) = bar {
            node.width = percent(bar.percent(fight));
        }
        if let Some(slot) = cooldown {
            node.height = percent(spells.get(slot.0).map_or(0., |spell| {
                if fight.onboard() {
                    fill_percent(fight.cooldowns()[slot.0], spell.cooldown.max(spell.gcd))
                } else {
                    0.
                }
            }));
        }
    }
    for (kind, mut label) in &mut labels {
        label.0 = match kind {
            HudText::Bar(HudBar::Health) => format!("PV {:.0} %", fight.health_fraction() * 100.),
            HudText::Bar(HudBar::Resource) => format!(
                "{} {:.0}",
                if matches!(fight.role, Role::DemoDriver | Role::DemoPassenger) {
                    "Pyrite"
                } else {
                    "Énergie"
                },
                fight.resource()
            ),
            HudText::Bar(HudBar::Boss) => format!(
                "{:.1} % · {:.2} M PV",
                fill_percent(fight.boss.hp, fight.boss.max_hp),
                fight.boss.hp / 1_000_000.
            ),
            HudText::Bar(HudBar::Cast) => {
                if fight.boss.cast > 0. {
                    format!("Décharges {:.1}s", fight.boss.cast)
                } else {
                    String::new()
                }
            }
            HudText::Bar(HudBar::Pyrite) => format!("Pyrite {} / 10", fight.player().stacks),
            HudText::Name(slot) => spells.get(*slot).map_or("—", |s| s.name).into(),
            HudText::Cooldown(slot) => {
                if *slot < spells.len() && fight.onboard() && fight.cooldowns()[*slot] > 0.05 {
                    format!("{:.1}", fight.cooldowns()[*slot])
                } else {
                    String::new()
                }
            }
            HudText::Hint
                if fight
                    .player_rider()
                    .is_some_and(|r| r.state == leviathan_combat::RiderState::Turret) =>
            {
                format!(
                    "Tourelles : {:.0} / {:.0} PV · {} · surcharge automatique ensuite",
                    fight.boss.turrets[0],
                    fight.boss.turrets[1],
                    if fight.attacking_turrets() {
                        "attaque en cours"
                    } else {
                        "1 ou clic pour attaquer"
                    }
                )
            }
            HudText::Hint => hovered_slot
                .or(aim.slot)
                .and_then(|slot| spells.get(slot))
                .map_or(String::new(), |spell| {
                    format!(
                        "{} · coût {:.0} · recharge {:.1}s · GCD {:.1}s · {}",
                        spell.name,
                        spell.cost,
                        spell.cooldown,
                        spell.gcd,
                        if spell.aimed {
                            format!("portée {:.0}–{:.0}", spell.min, spell.max)
                        } else {
                            "instantané / devant".into()
                        }
                    )
                }),
        };
    }
    for (slot, mut icon) in &mut icons {
        if let Some(id) = icon_id(fight.role, slot.0).filter(|_| slot.0 < spells.len()) {
            let handle = assets.load(format!("wow/ui/icon-{id}.png"));
            if icon.image != handle {
                icon.image = handle;
            }
            icon.color = if fight.resource() < spells[slot.0].cost {
                Color::srgb(0.45, 0.25, 0.2)
            } else {
                Color::WHITE
            };
        } else {
            icon.color = Color::NONE;
        }
    }
    let portrait = match fight.player().kind {
        leviathan_combat::Kind::Demo => "seat-demo",
        leviathan_combat::Kind::Siege => "seat-siege",
        leviathan_combat::Kind::Moto => "seat-moto",
    };
    for mut image in &mut portraits {
        let handle = assets.load(format!("wow/ui/{portrait}.png"));
        if image.image != handle {
            image.image = handle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hm_health_bars_use_maximums_not_normalized_constants() {
        let mut f = leviathan_combat::Fight::new(Role::DemoDriver, true);
        f.boss.hp *= 0.5;
        f.player_mut().hp *= 0.25;
        assert_eq!(HudBar::Boss.percent(&f), 50.);
        assert_eq!(HudBar::Health.percent(&f), 25.);
    }
    #[test]
    fn every_real_spell_has_a_copied_icon_and_empty_slots_have_none() {
        let assets = crate::models::asset_directory();
        for role in Role::ALL {
            let count = leviathan_combat::spells(role).len();
            for slot in 0..5 {
                let icon = icon_id(role, slot);
                assert_eq!(icon.is_some(), slot < count);
                if let Some(id) = icon {
                    assert!(assets.join(format!("wow/ui/icon-{id}.png")).is_file());
                }
            }
        }
        assert_eq!(icon_id(Role::Moto, usize::MAX), None);
    }
    #[test]
    fn bars_and_cooldowns_handle_depleted_and_invalid_values() {
        assert_eq!(fill_percent(50., 100.), 50.);
        assert_eq!(fill_percent(-3., 100.), 0.);
        assert_eq!(fill_percent(1000., 100.), 100.);
        assert_eq!(fill_percent(5., 0.), 0.);
        assert_eq!(fill_percent(f32::NAN, 100.), 0.);
        assert_eq!(fill_percent(100., f32::INFINITY), 0.);
    }

    #[test]
    fn cast_bar_starts_full_and_gunner_bar_uses_its_own_reserve() {
        let mut fight = leviathan_combat::Fight::new(Role::SiegeGunner, true);
        fight.boss.cast = 10.;
        assert_eq!(HudBar::Cast.percent(&fight), 100.);
        fight.boss.cast = 5.;
        assert_eq!(HudBar::Cast.percent(&fight), 50.);
        fight.player_mut().energy = 80.;
        fight.player_mut().gun_energy = 25.;
        assert_eq!(HudBar::Resource.percent(&fight), 25.);
    }
}
