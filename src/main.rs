mod actions;
mod input;
mod sprites;
mod updates;

use actions::*;
use sprites::*;
use updates::*;

use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(InputState::default())
        .insert_resource(GameWorld::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (input::move_player, input::update_camera).chain())
        .add_systems(
            Update,
            (
                input::mouse_button_events,
                handle_player_actions,
                update_preview_tile,
                update_rotating_tiles,
            ),
        )
        .add_systems(FixedUpdate, (update_tiles, update_miners, update_movers))
        .run();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ResourceType {
    texture_name: &'static str,
}

impl ResourceType {
    const fn new(texture_name: &'static str) -> Self {
        Self { texture_name }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ItemType {
    texture_name: &'static str,
}

impl ItemType {
    const fn new(texture_name: &'static str) -> Self {
        Self { texture_name }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct TileType {
    texture_name: &'static str,
    rotating_texture_name: Option<&'static str>,
}

impl TileType {
    const fn new(texture_name: &'static str) -> Self {
        Self {
            texture_name,
            rotating_texture_name: None,
        }
    }

    const fn with_rotating_part(self, texture_name: &'static str) -> Self {
        Self {
            rotating_texture_name: Some(texture_name),
            ..self
        }
    }
}

#[derive(Resource, Default)]
struct GameWorld {
    tiles: HashMap<(i32, i32), PlacedTile>,
}

mod resources {
    use super::ResourceType;

    pub static IRON_ORE: ResourceType = ResourceType::new("iron_ore");
}

mod items {
    use super::ItemType;

    pub static IRON_ORE: ItemType = ItemType::new("iron_ore");
    pub static IRON_SHEET: ItemType = ItemType::new("iron_sheet");
    pub static BELT: ItemType = ItemType::new("belt");
    pub static MINER: ItemType = ItemType::new("miner");
    pub static INSERTER: ItemType = ItemType::new("inserter");
    pub static FURNACE: ItemType = ItemType::new("furnace");
}

mod tiles {
    use super::TileType;

    pub static BELT: TileType = TileType::new("belt");
    pub static MINER: TileType = TileType::new("miner");
    pub static INSERTER: TileType =
        TileType::new("inserter_base").with_rotating_part("inserter_hand");
    pub static FURNACE: TileType = TileType::new("furnace");
}

const PLAYER_SPEED: f32 = 200.;

#[derive(Resource, Default)]
struct InputState {
    drag_start: Option<Vec2>,
    dropping_items: bool,
    deleting_tile: bool,
    rotation: u8,
    item_in_hand: Option<ItemType>,
}

#[derive(Component)]
struct TileRotation {
    anchor: Vec2,
    speed: f32,
    from: f32,
    to: f32,
    time: f32,
}

enum Layer {
    Background,
    Resource,
    Tile,
    Item,
    Player,
}

impl Layer {
    fn depth(&self) -> f32 {
        match self {
            Layer::Background => 0.0,
            Layer::Resource => 0.1,
            Layer::Tile => 0.2,
            Layer::Item => 0.3,
            Layer::Player => 0.4,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DroppedItem {
    item_type: ItemType,
}

#[derive(Component, Clone)]
struct PlacedTile {
    tile_type: TileType,
    rotation: u8,
    x: i32,
    y: i32,
}

#[derive(Component)]
struct ResourceProducer {
    timer: Timer,
    resource: ResourceType,
}

#[derive(Component)]
struct ItemMover {
    item: Option<ItemType>,
}

#[derive(Component)]
struct ResourceTile {
    resource_type: ResourceType,
    x: i32,
    y: i32,
}

#[derive(Component)]
struct PreviewTile;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let dirt_texture = asset_server.load("textures/bg/dirt.png");

    commands.spawn(Camera2dBundle::default());

    for y in -100..=100 {
        for x in -100..=100 {
            commands.spawn((SpriteBundle {
                transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(vec3(
                    x as f32 * 32.0,
                    y as f32 * 32.0,
                    Layer::Background.depth(),
                )),
                texture: dirt_texture.clone(),
                ..default()
            },));
        }
    }

    for (x, y) in [(5, 3), (5, 4), (5, 5), (6, 4), (6, 5)] {
        let tile = ResourceTile {
            resource_type: resources::IRON_ORE,
            x,
            y,
        };
        commands.spawn((create_resource_sprite(&asset_server, &tile), tile));
    }

    // Player
    commands.spawn((
        Player,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(5.)).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            transform: Transform {
                translation: vec3(0., 0., Layer::Player.depth()),
                ..default()
            },
            ..default()
        },
    ));
}

fn calc_rotating_tile_transform(tile: &PlacedTile, anchor: Vec2, angle: f32) -> Transform {
    let mut transform = Transform::from_scale(Vec3::splat(1.0))
        .with_rotation(Quat::from_rotation_z(PI / 2.0 * tile.rotation as f32))
        .with_translation(vec3(
            (tile.x as f32) * 32.0,
            (tile.y as f32) * 32.0,
            Layer::Tile.depth(),
        ));
    transform = transform.mul_transform(Transform::from_rotation(Quat::from_rotation_z(angle)));
    transform = transform.mul_transform(Transform::from_translation(-anchor.extend(0.0) * 32.0));
    transform
}
