//! Baked GLB pipeline: export our procedural art, then load it as a normal glTF scene.
use crate::{Player, Wheel};
use bevy::{
    asset::LoadState,
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
};

#[derive(Component)]
pub struct ProceduralVisual;
#[derive(Component)]
struct ImportedVisual;
#[derive(Resource, Default)]
pub struct ModelStatus {
    pub loaded: bool,
    pub animated_wheels: usize,
    pub fallback: bool,
    pub requested: bool,
    pub scene: Option<Handle<WorldAsset>>,
    pub error: String,
    pub converted: bool,
    pub boss_loaded: bool,
    pub player_loaded: [bool; 3],
    pub scenes_loaded: usize,
    pub animation_players: usize,
}
impl ModelStatus {
    pub fn has_player(&self, kind: leviathan_combat::Kind) -> bool {
        if self.converted {
            self.player_loaded[crate::wow::kind_index(kind)]
        } else {
            kind == leviathan_combat::Kind::Demo && self.loaded
        }
    }
}

pub fn asset_directory() -> PathBuf {
    let packaged = crate::output_directory().join("assets");
    if packaged.is_dir() {
        packaged
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }
}
pub fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    player: Single<Entity, With<Player>>,
    mut status: ResMut<ModelStatus>,
) {
    if status.converted {
        return;
    }
    if std::env::args().any(|a| a == "--bake-assets" || a == "--smoke-fallback") {
        status.fallback = true;
        status.error = "Modele procedural texture (mode de validation).".into();
        return;
    }
    if !asset_directory().join("models/demolisher.glb").is_file() {
        status.fallback = true;
        status.error = "GLB absent : modele procedural de secours.".into();
        return;
    }
    if let Err(error) = validate(&asset_directory().join("models/demolisher.glb")) {
        status.fallback = true;
        status.error = error;
        return;
    }
    let handle: Handle<WorldAsset> =
        assets.load(GltfAssetLabel::Scene(0).from_asset("models/demolisher.glb"));
    status.requested = true;
    status.scene = Some(handle.clone());
    commands
        .spawn((
            WorldAssetRoot(handle),
            ImportedVisual,
            crate::raid::PlayerVisual(leviathan_combat::Kind::Demo, false),
            ChildOf(*player),
            Visibility::Hidden,
        ))
        .observe(
            |event: On<WorldInstanceReady>,
             children: Query<&Children>,
             names: Query<&Name>,
             mut commands: Commands,
             fallback: Query<Entity, With<ProceduralVisual>>,
             mut status: ResMut<ModelStatus>| {
                for entity in children.iter_descendants(event.entity) {
                    if let Ok(name) = names.get(entity)
                        && let Some(radius) = name
                            .as_str()
                            .strip_prefix("Wheel:")
                            .and_then(|s| s.parse::<f32>().ok())
                    {
                        commands.entity(entity).insert(Wheel(radius));
                        status.animated_wheels += 1;
                    }
                }
                for entity in &fallback {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
                commands.entity(event.entity).insert(Visibility::Visible);
                status.loaded = true;
            },
        );
}
pub fn check_load(assets: Res<AssetServer>, mut status: ResMut<ModelStatus>) {
    if !status.loaded
        && !status.fallback
        && let Some(handle) = &status.scene
        && let Some(LoadState::Failed(error)) = assets.get_load_state(handle.id())
    {
        status.error = format!("GLB non charge : {error}. Modele procedural conserve.");
        status.fallback = true;
    }
}

fn append(
    binary: &mut Vec<u8>,
    bytes: &[u8],
    views: &mut Vec<Value>,
    target: Option<u32>,
) -> usize {
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    let index = views.len();
    let mut view = json!({"buffer":0,"byteOffset":binary.len(),"byteLength":bytes.len()});
    if let Some(target) = target {
        view["target"] = json!(target);
    }
    views.push(view);
    binary.extend_from_slice(bytes);
    index
}
fn floats<const N: usize>(data: &[[f32; N]]) -> Vec<u8> {
    data.iter()
        .flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes()))
        .collect()
}

fn build_document(world: &mut World) -> Result<Vec<u8>, String> {
    let root = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .map_err(|e| e.to_string())?;
    let mut entities = vec![];
    fn visit(world: &World, entity: Entity, out: &mut Vec<Entity>) {
        out.push(entity);
        if let Some(children) = world.get::<Children>(entity) {
            for child in children {
                visit(world, *child, out);
            }
        }
    }
    if let Some(children) = world.get::<Children>(root) {
        for child in children {
            visit(world, *child, &mut entities);
        }
    }
    let ids: HashMap<_, _> = entities.iter().enumerate().map(|(i, e)| (*e, i)).collect();
    let mut binary = vec![];
    let mut views = vec![];
    let mut accessors = vec![];
    let mut meshes = vec![];
    let mut materials = vec![];
    let mut images = vec![];
    let mut textures = vec![];
    let mut mesh_ids = HashMap::new();
    let mut material_ids = HashMap::new();
    let mut image_ids = HashMap::new();
    let mut nodes = vec![];
    let mesh_assets = world.resource::<Assets<Mesh>>();
    let material_assets = world.resource::<Assets<StandardMaterial>>();
    let image_assets = world.resource::<Assets<Image>>();
    for entity in &entities {
        let t = world.get::<Transform>(*entity).copied().unwrap_or_default();
        let mut node = json!({"translation":t.translation.to_array(),"rotation":t.rotation.to_array(),"scale":t.scale.to_array()});
        if let Some(name) = world.get::<Name>(*entity) {
            node["name"] = json!(name.as_str());
        }
        if let Some(children) = world.get::<Children>(*entity) {
            node["children"] = json!(
                children
                    .iter()
                    .filter_map(|e| ids.get(&e).copied())
                    .collect::<Vec<_>>()
            );
        }
        if let (Some(handle), Some(mat_handle)) = (
            world.get::<Mesh3d>(*entity),
            world.get::<MeshMaterial3d<StandardMaterial>>(*entity),
        ) {
            let material_id = if let Some(i) = material_ids.get(&mat_handle.0.id()) {
                *i
            } else {
                let mat = material_assets
                    .get(&mat_handle.0)
                    .ok_or("Material missing")?;
                let color = mat.base_color.to_linear();
                let mut pbr = json!({"baseColorFactor":[color.red,color.green,color.blue,color.alpha],"metallicFactor":mat.metallic,"roughnessFactor":mat.perceptual_roughness});
                if let Some(image_handle) = &mat.base_color_texture {
                    let texture_id = if let Some(index) = image_ids.get(&image_handle.id()) {
                        *index
                    } else {
                        let image = image_assets.get(image_handle).ok_or("Texture missing")?;
                        let bytes = image.data.as_ref().ok_or("No CPU pixels")?;
                        let rgba = image::RgbaImage::from_raw(
                            image.width(),
                            image.height(),
                            bytes.clone(),
                        )
                        .ok_or("Invalid pixels")?;
                        let mut png = Cursor::new(Vec::new());
                        image::DynamicImage::ImageRgba8(rgba)
                            .write_to(&mut png, image::ImageFormat::Png)
                            .map_err(|e| e.to_string())?;
                        let view = append(&mut binary, png.get_ref(), &mut views, None);
                        let index = images.len();
                        images.push(json!({"bufferView":view,"mimeType":"image/png"}));
                        textures.push(json!({"source":index,"sampler":0}));
                        image_ids.insert(image_handle.id(), index);
                        index
                    };
                    pbr["baseColorTexture"] = json!({"index":texture_id});
                }
                let index = materials.len();
                materials.push(json!({"pbrMetallicRoughness":pbr,"doubleSided":mat.double_sided}));
                material_ids.insert(mat_handle.0.id(), index);
                index
            };
            let key = (handle.0.id(), material_id);
            let mesh_id = if let Some(i) = mesh_ids.get(&key) {
                *i
            } else {
                let mesh = mesh_assets.get(&handle.0).ok_or("Mesh missing")?;
                let mut attributes = json!({});
                for (name, attribute) in [
                    ("POSITION", Mesh::ATTRIBUTE_POSITION),
                    ("NORMAL", Mesh::ATTRIBUTE_NORMAL),
                    ("TEXCOORD_0", Mesh::ATTRIBUTE_UV_0),
                ] {
                    let Some(values) = mesh.attribute(attribute) else {
                        continue;
                    };
                    let (bytes, count, kind) = match values {
                        VertexAttributeValues::Float32x3(v) => (floats(v), v.len(), "VEC3"),
                        VertexAttributeValues::Float32x2(v) => (floats(v), v.len(), "VEC2"),
                        _ => return Err("Unexpected vertex format".into()),
                    };
                    let view = append(&mut binary, &bytes, &mut views, Some(34962));
                    let mut accessor =
                        json!({"bufferView":view,"componentType":5126,"count":count,"type":kind});
                    if name == "POSITION"
                        && let VertexAttributeValues::Float32x3(values) = values
                    {
                        let min = values.iter().fold(Vec3::splat(f32::INFINITY), |a, b| {
                            a.min(Vec3::from_array(*b))
                        });
                        let max = values.iter().fold(Vec3::splat(f32::NEG_INFINITY), |a, b| {
                            a.max(Vec3::from_array(*b))
                        });
                        accessor["min"] = json!(min.to_array());
                        accessor["max"] = json!(max.to_array());
                    }
                    attributes[name] = json!(accessors.len());
                    accessors.push(accessor);
                }
                let mut primitive =
                    json!({"attributes":attributes,"material":material_id,"mode":4});
                if let Some(indices) = mesh.indices() {
                    let (bytes, component, count): (Vec<u8>, _, _) = match indices {
                        Indices::U16(i) => (
                            i.iter().flat_map(|n| n.to_le_bytes()).collect(),
                            5123,
                            i.len(),
                        ),
                        Indices::U32(i) => (
                            i.iter().flat_map(|n| n.to_le_bytes()).collect(),
                            5125,
                            i.len(),
                        ),
                    };
                    let view = append(&mut binary, &bytes, &mut views, Some(34963));
                    primitive["indices"] = json!(accessors.len());
                    accessors.push(json!({"bufferView":view,"componentType":component,"count":count,"type":"SCALAR"}));
                }
                let index = meshes.len();
                meshes.push(json!({"primitives":[primitive]}));
                mesh_ids.insert(key, index);
                index
            };
            node["mesh"] = json!(mesh_id);
        }
        nodes.push(node);
    }
    while binary.len() % 4 != 0 {
        binary.push(0);
    }
    let roots: Vec<_> = world
        .get::<Children>(root)
        .unwrap()
        .iter()
        .filter_map(|e| ids.get(&e).copied())
        .collect();
    let document = json!({"asset":{"version":"2.0","generator":"Leviathan Rust original art baker 0.2"},"scene":0,"scenes":[{"nodes":roots}],"nodes":nodes,"meshes":meshes,"materials":materials,"textures":textures,"images":images,"samplers":[{"magFilter":9729,"minFilter":9987,"wrapS":10497,"wrapT":10497}],"buffers":[{"byteLength":binary.len()}],"bufferViews":views,"accessors":accessors});
    package_glb(document, binary)
}
fn package_glb(document: Value, binary: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut json = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    let mut output = Vec::new();
    for value in [
        0x46546c67u32,
        2,
        (12 + 8 + json.len() + 8 + binary.len()) as u32,
        json.len() as u32,
        0x4e4f534a,
    ] {
        output.extend(value.to_le_bytes());
    }
    output.extend(json);
    output.extend((binary.len() as u32).to_le_bytes());
    output.extend(0x004e4942u32.to_le_bytes());
    output.extend(binary);
    Ok(output)
}

pub fn bake(world: &mut World) {
    if !std::env::args().any(|a| a == "--bake-assets") || world.contains_resource::<BakeDone>() {
        return;
    }
    // Run after deferred Startup commands and before runtime GLB loading.
    if world
        .query_filtered::<Entity, With<Player>>()
        .iter(world)
        .next()
        .is_none()
    {
        return;
    }
    world.insert_resource(BakeDone);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/models/demolisher.glb");
    let result = build_document(world).and_then(|bytes| {
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, bytes).map_err(|e| e.to_string())
    });
    let report = match &result {
        Ok(_) => format!("art_bake=PASS\nmodel={}\n", path.display()),
        Err(e) => format!("art_bake=FAIL\n{e}\n"),
    };
    let _ = std::fs::write(crate::output_directory().join("art-bake.txt"), report);
    world.write_message(if result.is_ok() {
        bevy::app::AppExit::Success
    } else {
        bevy::app::AppExit::error()
    });
}
#[derive(Resource)]
struct BakeDone;

pub fn validate(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    gltf::Gltf::from_slice(&bytes).map_err(|e| format!("Invalid glTF document: {e}"))?;
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err("Invalid GLB header".into());
    }
    let length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if length != bytes.len() || 20 + json_length > length {
        return Err("Invalid GLB length".into());
    }
    let document: Value =
        serde_json::from_slice(&bytes[20..20 + json_length]).map_err(|e| e.to_string())?;
    if document["images"].as_array().map_or(0, Vec::len) < 1 {
        return Err("Embedded material textures missing".into());
    }
    let wheels = document["nodes"]
        .as_array()
        .ok_or("GLB contains no nodes")?
        .iter()
        .filter(|node| {
            node["name"]
                .as_str()
                .is_some_and(|n| n.starts_with("Wheel:"))
        })
        .count();
    if wheels != 4 {
        return Err(format!("Expected four wheel pivots, found {wheels}"));
    }
    Ok(())
}

pub fn validate_wow(path: &Path) -> Result<(), String> {
    validate_wow_clips(path, &["Idle", "Drive"])
}
pub fn validate_wow_clips(path: &Path, required: &[&str]) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let gltf = gltf::Gltf::from_slice(&bytes).map_err(|e| e.to_string())?;
    if gltf.blob.is_none()
        || gltf
            .buffers()
            .any(|b| matches!(b.source(), gltf::buffer::Source::Uri(_)))
        || gltf.images().count() == 0
        || gltf
            .images()
            .any(|i| matches!(i.source(), gltf::image::Source::Uri { .. }))
    {
        return Err("Le GLB doit contenir toutes ses textures et donnees.".into());
    }
    let animations: Vec<_> = gltf.animations().filter_map(|a| a.name()).collect();
    if gltf.skins().count() == 0 || !animations.starts_with(required) {
        return Err(format!(
            "Squelette ou animations {required:?} absents ou mal ordonnés."
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copied_wow_models_are_self_contained_and_animated() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        for name in ["demolisher", "siege", "moto", "leviathan"] {
            validate_wow(&root.join(format!("wow/{name}.glb"))).unwrap();
        }
        for name in ["lasher", "ward"] {
            validate_wow_clips(
                &root.join(format!("wow/{name}.glb")),
                &["Idle", "Drive", "Attack"],
            )
            .unwrap();
        }
        validate_wow_clips(&root.join("wow/pyrite-air.glb"), &["Idle"]).unwrap();
        validate_wow_clips(&root.join("wow/pyrite-ground.glb"), &[]).unwrap();
    }

    #[test]
    fn player_fallback_stays_until_its_own_role_model_is_loaded() {
        use leviathan_combat::Kind;
        let mut status = ModelStatus::default();
        status.converted = true;
        status.player_loaded[0] = true;
        assert!(status.has_player(Kind::Demo));
        assert!(!status.has_player(Kind::Siege));
        assert!(!status.has_player(Kind::Moto));
        status.converted = false;
        status.loaded = true;
        assert!(status.has_player(Kind::Demo));
        assert!(!status.has_player(Kind::Siege));
    }
    #[test]
    fn packaged_glb_has_embedded_textures_and_four_wheel_pivots() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/models/demolisher.glb");
        validate(&path).unwrap();
    }
    #[test]
    fn glb_binary_layout_is_aligned() {
        let bytes = package_glb(json!({"asset":{"version":"2.0"}}), vec![0; 4]).unwrap();
        assert_eq!(bytes.len() % 4, 0);
        assert_eq!(
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
            bytes.len()
        );
    }
}
