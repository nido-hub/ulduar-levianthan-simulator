use crate::textures::{self, Surface};
use crate::{CameraControl, FollowCamera, Player, Wheel, config::Configuration};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI},
};

struct Art<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    cube: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cone: Handle<Mesh>,
    ring: Handle<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    palette: HashMap<u32, Handle<StandardMaterial>>,
    textures: HashMap<Surface, Handle<Image>>,
}
impl<'a, 'w, 's> Art<'a, 'w, 's> {
    fn new(
        commands: &'a mut Commands<'w, 's>,
        meshes: &mut Assets<Mesh>,
        materials: &'a mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        Self {
            commands,
            cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            cylinder: meshes.add(Cylinder::new(1.0, 1.0)),
            sphere: meshes.add(Sphere::new(1.0).mesh().uv(12, 8)),
            cone: meshes.add(Cone::new(1.0, 1.0)),
            ring: meshes.add(Torus::new(0.84, 0.96)),
            materials,
            palette: HashMap::new(),
            textures: [
                Surface::Wood,
                Surface::Iron,
                Surface::Bronze,
                Surface::Stone,
                Surface::Trim,
                Surface::Leather,
                Surface::Cloth,
                Surface::Horn,
            ]
            .into_iter()
            .map(|kind| (kind, images.add(textures::image(kind))))
            .collect(),
        }
    }
    fn group(&mut self, parent: Option<Entity>, transform: Transform) -> Entity {
        let mut entity = self.commands.spawn((transform, Visibility::default()));
        if let Some(parent) = parent {
            entity.insert(ChildOf(parent));
        }
        entity.id()
    }
    fn material(&mut self, hex: u32) -> Handle<StandardMaterial> {
        let surface = textures::surface(hex);
        let texture = surface.and_then(|s| self.textures.get(&s)).cloned();
        self.palette
            .entry(hex)
            .or_insert_with(|| {
                self.materials.add(StandardMaterial {
                    base_color: if texture.is_some() {
                        Color::srgb(0.94, 0.94, 0.94)
                    } else {
                        Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
                    },
                    base_color_texture: texture,
                    perceptual_roughness: 0.92,
                    metallic: if matches!(surface, Some(Surface::Iron | Surface::Bronze)) {
                        0.18
                    } else {
                        0.0
                    },
                    ..default()
                })
            })
            .clone()
    }
    fn piece(
        &mut self,
        parent: Entity,
        mesh: Handle<Mesh>,
        color: u32,
        position: Vec3,
        size: Vec3,
        rotation: Quat,
    ) -> Entity {
        let material = self.material(color);
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform {
                    translation: position,
                    scale: size,
                    rotation,
                },
                ChildOf(parent),
            ))
            .id()
    }
    fn block(&mut self, g: Entity, color: u32, p: [f32; 3], s: [f32; 3]) -> Entity {
        self.piece(
            g,
            self.cube.clone(),
            color,
            p.into(),
            s.into(),
            Quat::IDENTITY,
        )
    }
    fn ball(&mut self, g: Entity, color: u32, p: [f32; 3], s: [f32; 3]) {
        self.piece(
            g,
            self.sphere.clone(),
            color,
            p.into(),
            s.into(),
            Quat::IDENTITY,
        );
    }
    fn beam(&mut self, g: Entity, color: u32, a: [f32; 3], b: [f32; 3], width: f32) {
        let a = Vec3::from(a);
        let b = Vec3::from(b);
        let delta = b - a;
        self.piece(
            g,
            self.cube.clone(),
            color,
            (a + b) * 0.5,
            Vec3::new(width, delta.length(), width),
            Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
        );
    }
    fn wheel(&mut self, parent: Entity, x: f32, z: f32, radius: f32) {
        let wheel = self.group(Some(parent), Transform::from_xyz(x, radius, z));
        self.commands
            .entity(wheel)
            .insert((Wheel(radius), Name::new(format!("Wheel:{radius}"))));
        self.piece(
            wheel,
            self.cylinder.clone(),
            0x343934,
            Vec3::ZERO,
            Vec3::new(radius, 0.55, radius),
            Quat::from_rotation_x(FRAC_PI_2),
        );
        let sign = z.signum();
        self.piece(
            wheel,
            self.cylinder.clone(),
            0x87693c,
            Vec3::new(0.0, 0.0, sign * 0.25),
            Vec3::new(radius * 0.8, 0.24, radius * 0.8),
            Quat::from_rotation_x(FRAC_PI_2),
        );
        self.piece(
            wheel,
            self.ring.clone(),
            0xa9ad95,
            Vec3::new(0.0, 0.0, sign * 0.4),
            Vec3::splat(radius),
            Quat::from_rotation_x(FRAC_PI_2),
        );
        for i in 0..8 {
            let angle = i as f32 * PI / 4.0;
            self.piece(
                wheel,
                self.cube.clone(),
                0xbea571,
                Vec3::new(
                    angle.cos() * radius * 0.35,
                    angle.sin() * radius * 0.35,
                    sign * 0.41,
                ),
                Vec3::new(radius * 0.7, 0.12, 0.1),
                Quat::from_rotation_z(angle),
            );
            self.ball(
                wheel,
                0xddddba,
                [
                    angle.cos() * radius * 0.87,
                    angle.sin() * radius * 0.87,
                    sign * 0.42,
                ],
                [0.065; 3],
            );
        }
        self.piece(
            wheel,
            self.cone.clone(),
            0xb8b395,
            Vec3::new(0.0, 0.0, sign * 0.65),
            Vec3::new(0.3, 0.7, 0.3),
            Quat::from_rotation_x(sign * FRAC_PI_2),
        );
    }
    fn demolisher(&mut self, player: bool) -> Entity {
        let root = self.group(None, Transform::from_xyz(0.0, 0.0, 34.0));
        if player {
            self.commands
                .entity(root)
                .insert((Player, Name::new("Demolisseur pilotable")));
        }
        let g = self.group(
            Some(root),
            Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        );
        if player {
            self.commands.entity(g).insert((
                crate::models::ProceduralVisual,
                crate::raid::PlayerVisual(leviathan_combat::Kind::Demo, true),
            ));
        }
        for x in [-2.2, 2.0] {
            for z in [-1.9, 1.9] {
                self.wheel(g, x, z, if x < 0.0 { 1.35 } else { 1.2 });
            }
        }
        self.block(g, 0x6f5531, [-0.2, 1.7, 0.0], [5.8, 0.8, 3.0]);
        for z in [-1.5, 1.5] {
            self.block(g, 0x777d68, [-0.2, 2.2, z], [5.9, 0.35, 0.3]);
            self.beam(
                g,
                0x927443,
                [-2.4, 2.0, z * 0.7],
                [-1.1, 5.3, z * 0.7],
                0.35,
            );
            self.beam(g, 0xb1945d, [1.5, 2.0, z * 0.7], [-1.1, 5.3, z * 0.7], 0.35);
        }
        for i in 0..6 {
            self.block(
                g,
                0x9d7e4c,
                [-2.5 + i as f32 * 0.8, 2.14, 0.0],
                [0.08, 0.1, 2.8],
            );
        }
        self.block(g, 0x78572f, [-2.6, 3.7, 0.0], [0.4, 3.1, 2.5]);
        for y in [2.3, 4.0, 5.3] {
            self.block(g, 0x707969, [-2.65, y, 0.0], [0.55, 0.25, 2.8]);
        }
        for z in [-1.15, 1.15] {
            self.piece(
                g,
                self.cone.clone(),
                0xcfc49d,
                Vec3::new(-2.6, 5.8, z),
                Vec3::new(0.25, 1.2, 0.25),
                Quat::from_rotation_z(0.2),
            );
        }
        self.piece(
            g,
            self.cylinder.clone(),
            0x7e8375,
            Vec3::new(-1.1, 4.9, 0.0),
            Vec3::new(0.35, 3.0, 0.35),
            Quat::from_rotation_x(FRAC_PI_2),
        );
        self.beam(g, 0xae8a4c, [-2.4, 4.1, 0.0], [1.5, 6.2, 0.0], 0.4);
        self.piece(
            g,
            self.cylinder.clone(),
            0x706b4c,
            Vec3::new(1.5, 6.0, 0.0),
            Vec3::new(0.95, 0.3, 0.95),
            Quat::IDENTITY,
        );
        self.piece(
            g,
            self.ring.clone(),
            0x8c9480,
            Vec3::new(1.5, 6.2, 0.0),
            Vec3::splat(1.05),
            Quat::IDENTITY,
        );
        self.ball(g, 0x929b9d, [1.5, 6.25, 0.0], [0.6, 0.5, 0.6]);
        self.block(g, 0x66705d, [2.95, 1.8, 0.0], [0.7, 1.5, 3.6]);
        for z in [-1.4, -0.45, 0.45, 1.4] {
            self.piece(
                g,
                self.cone.clone(),
                0xd3c59e,
                Vec3::new(3.5, 1.8, z),
                Vec3::new(0.3, 1.0, 0.3),
                Quat::from_rotation_z(-FRAC_PI_2),
            );
        }
        // Driver, deliberately small relative to the vehicle.
        self.block(g, 0x885540, [-0.4, 3.0, -1.2], [0.65, 0.8, 0.65]);
        self.ball(g, 0xceb08b, [-0.25, 3.7, -1.2], [0.33, 0.4, 0.33]);
        self.ball(g, 0x5f685d, [-0.3, 3.9, -1.2], [0.36, 0.2, 0.35]);
        self.beam(g, 0x885540, [-0.3, 3.3, -1.5], [0.5, 3.0, -1.4], 0.15);
        self.block(g, 0x903b31, [-2.85, 4.2, -1.6], [0.12, 1.2, 0.9]);
        // Broad armoured cheeks break the rectangular back-board silhouette.
        for z in [-1.4_f32, 1.4] {
            let cheek = self.block(g, 0x777d68, [-2.35, 3.6, z], [0.6, 2.7, 0.28]);
            self.commands.entity(cheek).insert(
                Transform::from_xyz(-2.35, 3.6, z)
                    .with_scale(Vec3::new(0.6, 2.7, 0.28))
                    .with_rotation(Quat::from_rotation_x(z.signum() * 0.18)),
            );
            for y in [2.6, 3.3, 4.4] {
                self.ball(g, 0xddddba, [-2.4, y, z * 1.12], [0.09; 3]);
            }
            self.piece(
                g,
                self.cylinder.clone(),
                0x87693c,
                Vec3::new(-0.6, 2.7, z * 1.22),
                Vec3::new(0.42, 0.65, 0.42),
                Quat::from_rotation_x(FRAC_PI_2),
            );
        }
        root
    }
    fn siege(&mut self, parent: Option<Entity>) -> Entity {
        let root = self.group(parent, Transform::default());
        let g = self.group(
            Some(root),
            Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        );
        for z in [-1.9, 1.9] {
            self.wheel(g, 1.6, z, 1.6);
            self.wheel(g, -2.0, z, 1.0);
        }
        self.block(g, 0x66705d, [-0.1, 2.0, 0.], [5.5, 1.8, 3.1]);
        self.block(g, 0x777d68, [0.1, 3.0, 0.], [3.5, 0.5, 3.2]);
        for z in [-1.6, 1.6] {
            self.block(g, 0x78572f, [-0.7, 2.0, z], [2.8, 0.9, 0.15]);
            self.beam(g, 0xa9ad95, [-2., 1., z * 1.3], [1.6, 1.6, z * 1.3], 0.15);
        }
        self.piece(
            g,
            self.cylinder.clone(),
            0x7e8375,
            Vec3::new(-0.8, 3.4, 0.),
            Vec3::new(0.8, 2.5, 0.8),
            Quat::from_rotation_z(FRAC_PI_2),
        );
        self.piece(
            g,
            self.cylinder.clone(),
            0x707969,
            Vec3::new(-1.8, 4.1, -0.8),
            Vec3::new(0.25, 2.5, 0.25),
            Quat::IDENTITY,
        );
        self.block(g, 0x66705d, [1., 3.7, 0.], [1.5, 0.9, 1.5]);
        self.piece(
            g,
            self.cylinder.clone(),
            0x7e8375,
            Vec3::new(2.2, 3.8, 0.),
            Vec3::new(0.3, 2.4, 0.3),
            Quat::from_rotation_z(FRAC_PI_2),
        );
        self.block(g, 0xa9ad95, [2.9, 1.9, 0.], [0.6, 2.1, 3.5]);
        for z in [-1., 1.] {
            self.piece(
                g,
                self.cylinder.clone(),
                0x7e8375,
                Vec3::new(3.4, 2.1, z),
                Vec3::new(0.55, 1., 0.55),
                Quat::from_rotation_z(FRAC_PI_2),
            );
        }
        root
    }
    fn moto(&mut self, parent: Option<Entity>) -> Entity {
        let root = self.group(parent, Transform::default());
        let g = self.group(
            Some(root),
            Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        );
        self.wheel(g, -1.4, 0., 0.75);
        self.wheel(g, 1.9, 0., 0.8);
        self.wheel(g, -0.7, 1.8, 0.6);
        self.block(g, 0x777d68, [0., 1.2, 0.], [2.8, 0.7, 0.8]);
        self.ball(g, 0x66705d, [0.1, 1.8, 0.], [0.8, 0.4, 0.5]);
        self.block(g, 0x78572f, [-0.8, 1.7, 0.], [1.2, 0.25, 0.8]);
        self.block(g, 0x777d68, [-0.6, 1., 1.5], [2.3, 0.7, 1.1]);
        self.block(g, 0x343934, [-0.7, 1.45, 1.5], [0.9, 0.25, 0.7]);
        for z in [-0.25, 0.25] {
            self.beam(g, 0xa9ad95, [0.8, 2.1, z], [1.9, 0.8, z], 0.13);
        }
        self.block(g, 0xa9ad95, [0.8, 2.2, 0.], [0.15, 0.15, 1.5]);
        self.ball(g, 0xddddba, [1.6, 1.7, 0.], [0.2; 3]);
        self.block(g, 0x885540, [-0.6, 2.1, 0.], [0.6, 0.8, 0.65]);
        self.ball(g, 0xceb08b, [-0.4, 2.8, 0.], [0.3; 3]);
        root
    }
    fn boss(&mut self) -> Entity {
        let root = self.group(None, Transform::from_xyz(0., 0., -17.));
        let g = self.group(
            Some(root),
            Transform::from_rotation(Quat::from_rotation_y(FRAC_PI_2)),
        );
        for z in [-4., 4.] {
            self.block(g, 0x343934, [0., 1.4, z], [11., 2.5, 2.]);
            for i in 0..6 {
                self.wheel(g, -4.5 + i as f32 * 1.8, z, 1.25);
            }
            for i in 0..12 {
                self.block(
                    g,
                    0x777d68,
                    [-5. + i as f32 * 0.9, 2.8, z],
                    [0.55, 0.3, 2.2],
                );
            }
            self.piece(
                g,
                self.cylinder.clone(),
                0xbea571,
                Vec3::new(3., 3.5, z),
                Vec3::new(1.7, 2.3, 1.7),
                Quat::from_rotation_x(FRAC_PI_2),
            );
            self.ball(g, 0xe7a340, [3., 3.5, z * 1.3], [1.3, 1.3, 0.15]);
        }
        self.block(g, 0x66705d, [0., 3.4, 0.], [9.5, 3., 6.5]);
        self.block(g, 0xa9ad95, [0., 5., 0.], [9.7, 0.4, 6.7]);
        self.block(g, 0x777d68, [-1., 5.7, 0.], [5.5, 1.5, 5.]);
        for x in [-3.5, 0.] {
            for z in [-2.3, 2.3] {
                self.piece(
                    g,
                    self.cylinder.clone(),
                    0x707969,
                    Vec3::new(x, 6.4, z),
                    Vec3::new(0.4, 3.7, 0.4),
                    Quat::IDENTITY,
                );
            }
        }
        self.block(g, 0xbea571, [5.1, 2.9, 0.], [0.7, 3., 6.5]);
        for z in [-2.5, -1.25, 0., 1.25, 2.5] {
            self.piece(
                g,
                self.cone.clone(),
                0xd3c59e,
                Vec3::new(5.8, 2.5, z),
                Vec3::new(0.35, 2., 0.35),
                Quat::from_rotation_z(-FRAC_PI_2),
            );
        }
        for z in [-1.5, 1.5] {
            self.block(g, 0xa9ad95, [0.4, 6.7, z], [1.8, 1., 1.5]);
            self.piece(
                g,
                self.cylinder.clone(),
                0x7e8375,
                Vec3::new(1.8, 6.8, z),
                Vec3::new(0.3, 3., 0.3),
                Quat::from_rotation_z(FRAC_PI_2),
            );
        }
        root
    }
    fn arch(&mut self, g: Entity, x: f32, bottom: f32, z: f32, width: f32, height: f32) {
        let radius = width / 2.0;
        let stem = height - radius;
        for side in [-1.0, 1.0] {
            self.block(
                g,
                0x9690a4,
                [x + side * radius, bottom + stem / 2.0, z],
                [0.4, stem, 0.5],
            );
        }
        for i in 0..18 {
            let a = i as f32 / 18.0 * PI;
            let b = (i + 1) as f32 / 18.0 * PI;
            self.beam(
                g,
                0x9690a4,
                [x + a.cos() * radius, bottom + stem + a.sin() * radius, z],
                [x + b.cos() * radius, bottom + stem + b.sin() * radius, z],
                0.45,
            );
        }
    }
    fn bay(&mut self, wall: Entity, x: f32) {
        self.block(wall, 0x5e596e, [x, 9.0, 0.0], [15.0, 18.0, 2.8]);
        self.block(wall, 0x292736, [x, 8.4, 1.46], [5.4, 11.2, 0.12]);
        self.ball(wall, 0x292736, [x, 13.0, 1.46], [2.7, 2.7, 0.12]);
        self.arch(wall, x, 2.5, 1.7, 5.5, 13.1);
        self.ball(wall, 0x4a465a, [x, 11.4, 1.85], [1.3, 1.5, 0.7]);
        self.block(wall, 0x655d71, [x, 12.0, 2.0], [3.0, 0.4, 0.7]);
        self.block(wall, 0x4c475a, [x, 7.0, 1.8], [3.8, 4.5, 0.7]);
        for dx in [-0.8, 0.0, 0.8] {
            self.block(wall, 0x6b6277, [x + dx, 8.6, 2.3], [0.45, 2.6, 0.5]);
        }
        for dx in [-7.0, 7.0] {
            self.block(wall, 0x736c80, [x + dx, 10.6, 0.9], [2.2, 21.2, 4.0]);
            for y in [1.0, 3.0, 18.0, 20.5] {
                self.block(wall, 0x93879d, [x + dx, y, 0.9], [3.1, 0.7, 4.7]);
            }
            self.ball(wall, 0xa9bfdc, [x + dx, 19.0, 3.3], [0.55, 0.8, 0.35]);
            self.block(wall, 0x797c78, [x + dx, 13.0, 3.3], [0.25, 0.25, 1.7]);
            self.block(wall, 0x878774, [x + dx, 12.1, 4.1], [0.6, 1.3, 0.6]);
            self.ball(wall, 0x6197c9, [x + dx, 12.2, 4.15], [0.17, 0.45, 0.17]);
        }
        self.block(wall, 0x9c93a6, [x, 18.2, 0.5], [15.0, 0.6, 3.5]);
        // Pale frost sits on ledges, leaving the actual drivable floor clear.
        self.block(wall, 0xa9bfdc, [x, 18.56, 0.6], [14.7, 0.1, 3.3]);
    }
    fn courtyard(&mut self) {
        let root = self.group(None, Transform::default());
        for (x, z, angle) in [
            (-60.0, 0.0, FRAC_PI_2),
            (60.0, 0.0, -FRAC_PI_2),
            (0.0, 60.0, PI),
        ] {
            let wall = self.group(
                Some(root),
                Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(angle)),
            );
            for i in -3..=3 {
                self.bay(wall, i as f32 * 16.0);
            }
        }
        let north = self.group(Some(root), Transform::from_xyz(0.0, 0.0, -60.0));
        for x in [-48.0, -32.0, 32.0, 48.0] {
            self.bay(north, x);
        }
        for x in [-15.0, 15.0] {
            self.block(north, 0x716b81, [x, 13.5, 0.5], [5.0, 27.0, 6.5]);
            for y in [1.0, 3.5, 24.5, 26.5] {
                self.block(north, 0xa69aae, [x, y, 0.8], [6.7, 0.7, 7.4]);
            }
        }
        self.block(north, 0x746b81, [0.0, 25.0, 0.0], [29.0, 7.0, 5.5]);
        self.block(north, 0x1d1d2b, [0.0, 10.5, -0.5], [25.0, 21.0, 0.2]);
        self.arch(north, 0.0, 0.0, 2.5, 24.0, 23.5);
        for i in -5..=5 {
            self.block(
                north,
                0x797c78,
                [i as f32 * 2.0, 9.0, 1.1],
                [0.18, 18.0, 0.4],
            );
        }
        for y in [3.0, 8.0, 13.0] {
            self.block(north, 0x878774, [0.0, y, 1.2], [23.0, 0.25, 0.5]);
        }
        // Thin inlays identify the collision boundary without physical obstacles inside it.
        for x in [-56.0, 56.0] {
            self.block(root, 0x93896c, [x, 0.01, 0.0], [0.12, 0.03, 112.0]);
        }
        for z in [-56.0, 56.0] {
            self.block(root, 0x93896c, [0.0, 0.01, z], [112.0, 0.03, 0.12]);
        }
        for i in 0..9 {
            let height = 40.0 + (i % 3) as f32 * 13.0;
            self.piece(
                root,
                self.cone.clone(),
                0x3c3d58,
                Vec3::new(-140.0 + i as f32 * 36.0, height / 2.0 - 4.0, -110.0),
                Vec3::new(36.0, height, 30.0),
                Quat::IDENTITY,
            );
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    config: Res<Configuration>,
    camera: Res<CameraControl>,
) {
    let texture = images.add(textures::image(Surface::Floor));
    let floor = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.98,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(122.0, 122.0))),
        MeshMaterial3d(floor),
        Transform::from_xyz(0.0, -0.03, 0.0),
        Name::new("Pavage de la cour"),
    ));
    {
        let mut art = Art::new(&mut commands, &mut meshes, &mut materials, &mut images);
        art.courtyard();
        let player = art.demolisher(true);
        if !std::env::args().any(|a| a == "--bake-assets") {
            let siege = art.siege(Some(player));
            art.commands.entity(siege).insert(crate::raid::PlayerVisual(
                leviathan_combat::Kind::Siege,
                true,
            ));
            let moto = art.moto(Some(player));
            art.commands.entity(moto).insert(crate::raid::PlayerVisual(
                leviathan_combat::Kind::Moto,
                true,
            ));
            for i in 0..6 {
                let entity = match i {
                    0 | 1 => art.demolisher(false),
                    2 | 3 => art.siege(None),
                    _ => art.moto(None),
                };
                art.commands.entity(entity).insert(crate::raid::RaidBody(i));
            }
            let boss = art.boss();
            art.commands.entity(boss).insert(crate::raid::BossBody);
        }
    }
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.75, 0.8, 1.0),
        brightness: 300.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.91, 0.9, 1.0),
            illuminance: 7500.0,
            shadow_maps_enabled: config.settings.video.shadows,
            ..default()
        },
        Transform::from_xyz(-30.0, 60.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 12.0,
            maximum_distance: 140.0,
            ..default()
        }
        .build(),
    ));
    commands.spawn((
        FollowCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: config.settings.camera.fov_degrees.to_radians(),
            far: 350.0,
            ..default()
        }),
        Transform::from_translation(camera.center + Vec3::from_array(camera.orbit.offset()))
            .looking_at(camera.center, Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.12, 0.13, 0.21),
            falloff: FogFalloff::Linear {
                start: 105.0,
                end: 220.0,
            },
            ..default()
        },
    ));
}
