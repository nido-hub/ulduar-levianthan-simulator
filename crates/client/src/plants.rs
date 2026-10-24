//! Freya's copied plants, with a procedural fallback until the scene and clips load.
use crate::{RunState, models, raid::Raid};
use bevy::{
    asset::LoadState,
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady},
};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct Plants {
    scenes: [Option<(
        Handle<WorldAsset>,
        Handle<AnimationGraph>,
        [AnimationNodeIndex; 3],
    )>; 2],
    stems: Handle<Mesh>,
    leaves: Handle<Mesh>,
    green: Handle<StandardMaterial>,
    brown: Handle<StandardMaterial>,
    roots: HashMap<u64, Entity>,
    pub loaded: u32,
}
#[derive(Component)]
pub struct PlantBody {
    attacking: bool,
}
#[derive(Component)]
pub struct PlantScene {
    root: Entity,
    fallback: Entity,
    scene: Handle<WorldAsset>,
    graph: Handle<AnimationGraph>,
    clips: [AnimationNodeIndex; 3],
    ready: bool,
    failed: bool,
}
#[derive(Component)]
pub struct PlantAnimation {
    root: Entity,
    clips: [AnimationNodeIndex; 3],
    previous: Vec3,
    slot: usize,
}

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scenes = ["wow/lasher.glb", "wow/ward.glb"].map(|path| {
        if std::env::args().any(|a| a == "--smoke-fallback")
            || models::validate_wow_clips(
                &models::asset_directory().join(path),
                &["Idle", "Drive", "Attack"],
            )
            .is_err()
        {
            return None;
        }
        let (graph, nodes) = AnimationGraph::from_clips(
            (0..3).map(|i| assets.load(GltfAssetLabel::Animation(i).from_asset(path))),
        );
        Some((
            assets.load(GltfAssetLabel::Scene(0).from_asset(path)),
            graphs.add(graph),
            [nodes[0], nodes[1], nodes[2]],
        ))
    });
    commands.insert_resource(Plants {
        scenes,
        stems: meshes.add(Cylinder::new(0.24, 1.)),
        leaves: meshes.add(Sphere::new(1.).mesh().uv(8, 6)),
        green: materials.add(Color::srgb(0.2, 0.46, 0.12)),
        brown: materials.add(Color::srgb(0.25, 0.13, 0.05)),
        roots: HashMap::new(),
        loaded: 0,
    });
}

pub fn sync(
    mut commands: Commands,
    raid: Res<Raid>,
    mut plants: ResMut<Plants>,
    mut roots: Query<(&mut Transform, &mut PlantBody)>,
    assets: Res<AssetServer>,
    mut scenes: Query<&mut PlantScene>,
) {
    let active: HashSet<_> = raid.0.adds.iter().map(|a| a.id).collect();
    plants.roots.retain(|id, entity| {
        if active.contains(id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
    for a in &raid.0.adds {
        if let Some(entity) = plants.roots.get(&a.id) {
            if let Ok((mut transform, mut body)) = roots.get_mut(*entity) {
                let next = Vec3::new(a.position.x, 0., a.position.z);
                let delta = next - transform.translation;
                if delta.length_squared() > 0.00001 {
                    transform.rotation = Quat::from_rotation_y((-delta.x).atan2(-delta.z));
                }
                transform.translation = next;
                body.attacking = a.attack > 1.5;
            }
            continue;
        }
        let root = commands
            .spawn((
                PlantBody { attacking: false },
                Transform::from_xyz(a.position.x, 0., a.position.z),
                Visibility::default(),
            ))
            .id();
        let scale = if a.ward { 3. } else { 1. };
        let fallback = commands
            .spawn((
                Transform::from_scale(Vec3::splat(scale)),
                Visibility::Inherited,
                ChildOf(root),
            ))
            .id();
        commands.spawn((
            Mesh3d(plants.stems.clone()),
            MeshMaterial3d(plants.brown.clone()),
            Transform::from_xyz(0., 1.3, 0.).with_scale(Vec3::new(1., 2.6, 1.)),
            ChildOf(fallback),
        ));
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.;
            commands.spawn((
                Mesh3d(plants.leaves.clone()),
                MeshMaterial3d(plants.green.clone()),
                Transform::from_xyz(angle.sin() * 0.85, 2.2, angle.cos() * 0.85)
                    .with_scale(Vec3::new(0.3, 0.16, 1.1))
                    .with_rotation(Quat::from_rotation_y(angle)),
                ChildOf(fallback),
            ));
        }
        if let Some((scene, graph, clips)) = &plants.scenes[usize::from(a.ward)] {
            commands
                .spawn((
                    WorldAssetRoot(scene.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                    ChildOf(root),
                    PlantScene {
                        root,
                        fallback,
                        scene: scene.clone(),
                        graph: graph.clone(),
                        clips: *clips,
                        ready: false,
                        failed: false,
                    },
                ))
                .observe(ready);
        }
        plants.roots.insert(a.id, root);
    }
    for mut scene in &mut scenes {
        if !scene.ready
            && !scene.failed
            && let Some(LoadState::Failed(error)) = assets.get_load_state(scene.scene.id())
        {
            warn!("Plante GLB indisponible, secours conservé : {error}");
            scene.failed = true;
        }
    }
}

fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    mut scenes: Query<&mut PlantScene>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    mut plants: ResMut<Plants>,
) {
    let Ok(mut scene) = scenes.get_mut(event.entity) else {
        return;
    };
    let mut animated = false;
    for entity in children.iter_descendants(event.entity) {
        if let Ok(mut player) = players.get_mut(entity) {
            player.play(scene.clips[0]).repeat();
            commands.entity(entity).insert((
                AnimationGraphHandle(scene.graph.clone()),
                PlantAnimation {
                    root: scene.root,
                    clips: scene.clips,
                    previous: Vec3::splat(f32::NAN),
                    slot: 0,
                },
            ));
            animated = true;
        }
    }
    if animated {
        commands.entity(scene.fallback).insert(Visibility::Hidden);
        commands.entity(event.entity).insert(Visibility::Inherited);
        scene.ready = true;
        plants.loaded += 1;
    }
}

pub fn animate(
    state: Res<RunState>,
    roots: Query<(&Transform, &PlantBody)>,
    mut players: Query<(&mut AnimationPlayer, &mut PlantAnimation)>,
) {
    for (mut player, mut animation) in &mut players {
        let Ok((transform, body)) = roots.get(animation.root) else {
            continue;
        };
        if state.paused {
            player.pause_all();
            continue;
        }
        player.resume_all();
        let slot = if body.attacking {
            2
        } else if transform.translation.distance_squared(animation.previous) > 0.00001 {
            1
        } else {
            0
        };
        animation.previous = transform.translation;
        if slot != animation.slot {
            player.stop_all();
            player.play(animation.clips[slot]).repeat();
            animation.slot = slot;
        }
    }
}
