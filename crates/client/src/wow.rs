//! Copied GLB scenes only: no client/archive lookup is permitted at runtime.
use crate::{Player, RunState, models, raid};
use bevy::{
    asset::LoadState,
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady},
};
use leviathan_combat::Kind;

pub const MODELS: [&str; 4] = [
    "wow/demolisher.glb",
    "wow/siege.glb",
    "wow/moto.glb",
    "wow/leviathan.glb",
];

pub fn kind_index(kind: Kind) -> usize {
    match kind {
        Kind::Demo => 0,
        Kind::Siege => 1,
        Kind::Moto => 2,
    }
}

#[derive(Component)]
pub struct ImportedActor {
    actor: Entity,
    fallback: Vec<Entity>,
    player_kind: Option<Kind>,
    scene: Handle<WorldAsset>,
    graph: Handle<AnimationGraph>,
    clips: [AnimationNodeIndex; 2],
    ready: bool,
    failed: bool,
}

#[derive(Component)]
pub struct DriveAnimation {
    actor: Entity,
    clips: [AnimationNodeIndex; 2],
    previous: Option<Vec3>,
    moving: bool,
}

pub fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    player: Single<Entity, With<Player>>,
    visuals: Query<(Entity, &raid::PlayerVisual)>,
    actors: Query<(
        Entity,
        &Children,
        Option<&raid::RaidBody>,
        Has<raid::BossBody>,
    )>,
    mut status: ResMut<models::ModelStatus>,
) {
    if std::env::args().any(|a| a == "--bake-assets" || a == "--smoke-fallback") {
        return;
    }
    for path in MODELS {
        if let Err(error) = models::validate_wow(&models::asset_directory().join(path)) {
            warn!("Ressources WoW indisponibles ({path}): {error}. Visuels originaux conserves.");
            return;
        }
    }
    status.converted = true;
    let scenes: Vec<_> = MODELS
        .iter()
        .map(|path| {
            let scene: Handle<WorldAsset> = assets.load(GltfAssetLabel::Scene(0).from_asset(*path));
            let (graph, nodes) = AnimationGraph::from_clips(
                (0..2).map(|i| assets.load(GltfAssetLabel::Animation(i).from_asset(*path))),
            );
            (scene, graphs.add(graph), [nodes[0], nodes[1]])
        })
        .collect();
    let mut spawn =
        |actor: Entity, fallback: Vec<Entity>, index: usize, player_kind: Option<Kind>| {
            let (scene, graph, clips) = &scenes[index];
            let mut entity = commands.spawn((
                WorldAssetRoot(scene.clone()),
                ImportedActor {
                    actor,
                    fallback,
                    player_kind,
                    scene: scene.clone(),
                    graph: graph.clone(),
                    clips: *clips,
                    ready: false,
                    failed: false,
                },
                Transform::default(),
                Visibility::Hidden,
                ChildOf(actor),
            ));
            if let Some(kind) = player_kind {
                entity.insert(raid::PlayerVisual(kind, false));
            }
            entity.observe(ready);
        };
    for (entity, visual) in &visuals {
        spawn(*player, vec![entity], kind_index(visual.0), Some(visual.0));
    }
    for (entity, children, body, boss) in &actors {
        let index = if boss {
            3
        } else if let Some(body) = body {
            body.0 / 2
        } else {
            continue;
        };
        spawn(entity, children.iter().collect(), index, None);
    }
}

fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    mut imports: Query<&mut ImportedActor>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    mut status: ResMut<models::ModelStatus>,
) {
    let Ok(mut imported) = imports.get_mut(event.entity) else {
        return;
    };
    let mut animated = false;
    for child in children.iter_descendants(event.entity) {
        if let Ok(mut player) = players.get_mut(child) {
            player.play(imported.clips[0]).repeat();
            commands.entity(child).insert((
                AnimationGraphHandle(imported.graph.clone()),
                DriveAnimation {
                    actor: imported.actor,
                    clips: imported.clips,
                    previous: None,
                    moving: false,
                },
            ));
            status.animation_players += 1;
            animated = true;
        }
    }
    if !animated {
        status.error = "GLB sans lecteur d'animation : secours conserve.".into();
        return;
    }
    for &fallback in &imported.fallback {
        commands.entity(fallback).insert(Visibility::Hidden);
    }
    if let Some(kind) = imported.player_kind {
        status.player_loaded[kind_index(kind)] = true;
        if kind == Kind::Demo {
            status.loaded = true;
        }
    } else {
        commands.entity(event.entity).insert(Visibility::Inherited);
    }
    imported.ready = true;
    status.scenes_loaded += 1;
}

pub fn check_load(
    assets: Res<AssetServer>,
    mut imports: Query<&mut ImportedActor>,
    mut status: ResMut<models::ModelStatus>,
) {
    for mut imported in &mut imports {
        if !imported.ready
            && !imported.failed
            && let Some(LoadState::Failed(error)) = assets.get_load_state(imported.scene.id())
        {
            status.error = format!("GLB WoW non charge : {error}. Secours conserve.");
            warn!("{}", status.error);
            imported.failed = true;
        }
    }
}

pub fn animate(
    time: Res<Time>,
    state: Res<RunState>,
    actors: Query<&Transform>,
    mut animations: Query<(&mut AnimationPlayer, &mut DriveAnimation)>,
) {
    for (mut player, mut animation) in &mut animations {
        let Ok(transform) = actors.get(animation.actor) else {
            continue;
        };
        let position = transform.translation;
        let speed = animation
            .previous
            .map_or(0., |p| position.distance(p) / time.delta_secs().max(0.001));
        animation.previous = Some(position);
        if state.paused {
            player.pause_all();
            continue;
        }
        player.resume_all();
        let moving = speed > 0.2;
        if moving != animation.moving {
            player.stop_all();
            player.play(animation.clips[usize::from(moving)]).repeat();
            animation.moving = moving;
        }
        if moving && let Some(active) = player.animation_mut(animation.clips[1]) {
            active.set_speed((speed / 9.).clamp(0.25, 2.5));
        }
    }
}
