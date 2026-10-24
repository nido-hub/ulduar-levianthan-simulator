//! WotLK safety container in the air, liquid fuel cell on the ground.
use crate::{
    RunState, models,
    raid::{Effect, Effects, Raid},
};
use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady},
};
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct Pyrite {
    scenes: [Option<(
        Handle<WorldAsset>,
        Handle<AnimationGraph>,
        Option<AnimationNodeIndex>,
    )>; 2],
    roots: HashMap<u64, Entity>,
    pub loaded: [u32; 2],
}
#[derive(Component)]
pub struct PyriteBody(bool);
#[derive(Component)]
struct PyriteScene {
    air: bool,
    fallback: Entity,
    graph: Handle<AnimationGraph>,
    idle: Option<AnimationNodeIndex>,
}
#[derive(Component)]
pub struct PyriteAnimation;

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scenes = ["wow/pyrite-ground.glb", "wow/pyrite-air.glb"].map(|path| {
        if std::env::args().any(|a| a == "--smoke-fallback" || a == "--bake-assets") {
            return None;
        }
        let clips: &[&str] = if path.ends_with("-air.glb") {
            &["Idle"]
        } else {
            &[]
        };
        if let Err(error) = models::validate_wow_clips(&models::asset_directory().join(path), clips)
        {
            warn!("Pyrite GLB indisponible, secours conservé : {error}");
            return None;
        }
        let (graph, nodes) = AnimationGraph::from_clips(
            (0..clips.len()).map(|i| assets.load(GltfAssetLabel::Animation(i).from_asset(path))),
        );
        Some((
            assets.load(GltfAssetLabel::Scene(0).from_asset(path)),
            graphs.add(graph),
            nodes.first().copied(),
        ))
    });
    commands.insert_resource(Pyrite {
        scenes,
        roots: HashMap::new(),
        loaded: [0; 2],
    });
}

pub fn sync(
    mut commands: Commands,
    raid: Res<Raid>,
    mut pyrite: ResMut<Pyrite>,
    effects: Res<Effects>,
    mut bodies: Query<(&PyriteBody, &mut Transform)>,
) {
    let active: HashSet<_> = raid.0.crates.iter().map(|c| c.id).collect();
    pyrite.roots.retain(|id, entity| {
        if active.contains(id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
    for c in &raid.0.crates {
        let air = c.air > 0.;
        let transform = Transform::from_xyz(c.position.x, if air { 8. } else { 0. }, c.position.z);
        if let Some(&entity) = pyrite.roots.get(&c.id) {
            if let Ok((body, mut old)) = bodies.get_mut(entity)
                && body.0 == air
            {
                *old = transform;
                continue;
            }
            commands.entity(entity).despawn();
        }
        let root = commands
            .spawn((Effect, PyriteBody(air), transform, Visibility::default()))
            .id();
        let fallback = commands
            .spawn((Transform::default(), Visibility::Inherited, ChildOf(root)))
            .id();
        let height = if air { 0. } else { 0.8 };
        commands.spawn((
            Mesh3d(effects.cube.clone()),
            MeshMaterial3d(effects.colors[0].clone()),
            Transform::from_xyz(0., height, 0.)
                .with_scale(Vec3::splat(0.9))
                .with_rotation(Quat::from_rotation_y(0.5)),
            ChildOf(fallback),
        ));
        commands.spawn((
            Mesh3d(effects.ball.clone()),
            MeshMaterial3d(effects.colors[1].clone()),
            Transform::from_xyz(0., height + 0.75, 0.).with_scale(Vec3::splat(0.5)),
            ChildOf(fallback),
        ));
        if let Some((scene, graph, idle)) = &pyrite.scenes[usize::from(air)] {
            commands
                .spawn((
                    WorldAssetRoot(scene.clone()),
                    PyriteScene {
                        air,
                        fallback,
                        graph: graph.clone(),
                        idle: *idle,
                    },
                    Transform::default(),
                    Visibility::Hidden,
                    ChildOf(root),
                ))
                .observe(ready);
        }
        pyrite.roots.insert(c.id, root);
    }
}

fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    scenes: Query<&PyriteScene>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    mut pyrite: ResMut<Pyrite>,
) {
    let Ok(scene) = scenes.get(event.entity) else {
        return;
    };
    let mut animated = scene.idle.is_none();
    for child in children.iter_descendants(event.entity) {
        if let Some(idle) = scene.idle
            && let Ok(mut player) = players.get_mut(child)
        {
            player.play(idle).repeat();
            commands
                .entity(child)
                .insert((AnimationGraphHandle(scene.graph.clone()), PyriteAnimation));
            animated = true;
        }
    }
    if animated {
        commands.entity(scene.fallback).insert(Visibility::Hidden);
        commands.entity(event.entity).insert(Visibility::Inherited);
        pyrite.loaded[usize::from(scene.air)] += 1;
    }
}

pub fn animate(
    state: Res<RunState>,
    mut players: Query<&mut AnimationPlayer, With<PyriteAnimation>>,
) {
    for mut player in &mut players {
        if state.paused {
            player.pause_all();
        } else {
            player.resume_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leviathan_combat::Crate;
    use leviathan_driving::Point;

    #[test]
    fn crates_switch_to_ground_and_cleanup_on_collection_or_reset() {
        let mut app = App::new();
        app.init_resource::<Raid>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<Image>>()
            .insert_resource(Pyrite {
                scenes: [None, None],
                roots: HashMap::new(),
                loaded: [0; 2],
            })
            .add_systems(Startup, crate::raid::setup_effects)
            .add_systems(Update, sync);
        app.world_mut().resource_mut::<Raid>().0.crates.push(Crate {
            id: 77,
            position: Point { x: 12., z: -5. },
            air: 180.,
            life: 180.,
        });
        app.update();
        let air = app.world().resource::<Pyrite>().roots[&77];
        assert!(app.world().get::<PyriteBody>(air).unwrap().0);
        assert_eq!(app.world().get::<Transform>(air).unwrap().translation.y, 8.);
        app.world_mut().resource_mut::<Raid>().0.crates[0].air = 0.;
        app.update();
        let ground = app.world().resource::<Pyrite>().roots[&77];
        assert_ne!(air, ground);
        assert!(app.world().get_entity(air).is_err());
        assert!(!app.world().get::<PyriteBody>(ground).unwrap().0);
        assert_eq!(
            app.world().get::<Transform>(ground).unwrap().translation.y,
            0.
        );
        // A reset may reuse an ID before rendering observes an empty crate list.
        app.world_mut().resource_mut::<Raid>().0.crates[0]
            .position
            .x = 42.;
        app.update();
        assert_eq!(
            app.world().get::<Transform>(ground).unwrap().translation.x,
            42.
        );
        let children: Vec<_> = app
            .world()
            .get::<Children>(ground)
            .unwrap()
            .iter()
            .collect();
        app.world_mut().resource_mut::<Raid>().0.crates.clear();
        app.update();
        assert!(app.world().resource::<Pyrite>().roots.is_empty());
        assert!(app.world().get_entity(ground).is_err());
        assert!(
            children
                .into_iter()
                .all(|e| app.world().get_entity(e).is_err())
        );
    }
}
