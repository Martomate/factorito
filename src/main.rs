mod actions;
mod input;
mod sprites;
mod ui;
mod updates;

use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};
use bevy_prng::ChaCha8Rng;
use bevy_rand::{plugin::EntropyPlugin, prelude::GlobalEntropy};
use rand::Rng;
use std::f32::consts::PI;

use sprites::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Factorito".to_string(),
                        canvas: Some("#app".to_string()),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        ) // prevents blurry sprites
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        .insert_resource(InputState::default())
        .insert_resource(GameWorld::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (input::move_player, input::update_camera).chain())
        .add_systems(
            Update,
            (
                input::mouse_button_events,
                actions::handle_player_actions,
                updates::update_preview_tile,
                updates::update_rotating_tiles,
                ui::hanle_player_inventory_ui_events,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                updates::update_tiles,
                updates::update_miners,
                updates::update_movers,
                updates::update_item_processors,
            ),
        )
        .run();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ResourceType {
    texture_name: &'static str,
    item_to_produce: ItemType,
}

impl ResourceType {
    const fn new(texture_name: &'static str, item_to_produce: ItemType) -> Self {
        Self {
            texture_name,
            item_to_produce,
        }
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
    item_to_drop: ItemType,
}

impl TileType {
    const fn new(texture_name: &'static str, item_to_drop: ItemType) -> Self {
        Self {
            texture_name,
            rotating_texture_name: None,
            item_to_drop,
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

static RESOURCE_TYPES: [ResourceType; 3] = [
    ResourceType::new("coal", items::COAL),
    ResourceType::new("iron_ore", items::IRON_ORE),
    ResourceType::new("copper_ore", items::COPPER_ORE),
];

mod items {
    use super::ItemType;

    pub static COAL: ItemType = ItemType::new("coal");
    pub static IRON_ORE: ItemType = ItemType::new("iron_ore");
    pub static COPPER_ORE: ItemType = ItemType::new("copper_ore");
    pub static IRON_SHEET: ItemType = ItemType::new("iron_sheet");
    pub static COPPER_SHEET: ItemType = ItemType::new("copper_sheet");
    pub static BELT: ItemType = ItemType::new("belt");
    pub static MINER: ItemType = ItemType::new("miner");
    pub static INSERTER: ItemType = ItemType::new("inserter");
    pub static FURNACE: ItemType = ItemType::new("furnace");
}

mod tiles {
    use crate::items;

    use super::TileType;

    pub static BELT: TileType = TileType::new("belt", items::BELT);
    pub static MINER: TileType = TileType::new("miner", items::MINER);
    pub static INSERTER: TileType =
        TileType::new("inserter_base", items::INSERTER).with_rotating_part("inserter_hand");
    pub static FURNACE: TileType = TileType::new("furnace", items::FURNACE);
}

const PLAYER_SPEED: f32 = 200.;

#[derive(Resource, Default)]
struct InputState {
    drag_start: Option<Vec2>,
    dropping_items: bool,
    dropping_items_timer: Option<Timer>,
    picking_items: bool,
    picking_items_timer: Option<Timer>,
    deleting_tile: bool,
    deleting_tile_timer: Option<Timer>,
    rotation: u8,
    item_in_hand: Option<ItemType>,
    inventory_ui: Option<Entity>,
    toggling_inventory_visible: bool,
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
struct Player {
    inventory: Vec<(ItemType, usize)>,
}

impl Player {
    pub fn has_item_in_inventory(&mut self, item_type: ItemType) -> bool {
        self.inventory
            .iter()
            .any(|(t, c)| *t == item_type && *c > 0)
    }

    pub fn decrement_inventory(&mut self, item_type: ItemType) -> bool {
        let res = if let Some((_, c)) = self
            .inventory
            .iter_mut()
            .find(|(t, c)| *t == item_type && *c > 0)
        {
            *c -= 1;
            true
        } else {
            false
        };
        self.inventory.retain(|(_, c)| *c > 0);
        res
    }

    pub fn increment_inventory(&mut self, item_type: ItemType) -> bool {
        if let Some((_, c)) = self.inventory.iter_mut().find(|(t, _)| *t == item_type) {
            *c += 1;
            true
        } else {
            self.inventory.push((item_type, 1));
            true
        }
    }
}

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
struct ItemProcessor {
    timer: Timer,
    item: Option<ItemType>,
    output: Option<(ItemType, usize)>,
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
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
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

    for _ in 0..100 {
        let cx = rng.gen_range(-100..100);
        let cy = rng.gen_range(-100..100);

        let res_type = RESOURCE_TYPES[rng.gen_range(0..3)];
        let num_tiles = rng.gen_range(5..40);

        let mut taken: Vec<(i32, i32)> = Vec::new();

        let x = 0;
        let y = 0;
        taken.push((x, y));

        let tile = ResourceTile {
            resource_type: res_type,
            x: cx + x,
            y: cy + y,
        };
        commands.spawn((create_resource_sprite(&asset_server, &tile), tile));

        for _ in 1..num_tiles {
            let mut found = false;

            while !found {
                let idx = rng.gen_range(0..taken.len());
                let (sx, sy) = taken[idx];
                for d in 0..4 {
                    let d = d * 2 + 1;
                    let dx = (d % 3) - 1;
                    let dy = (d / 3) - 1;

                    let x = sx + dx;
                    let y = sy + dy;

                    if !taken.contains(&(x, y)) {
                        taken.push((x, y));

                        let tile = ResourceTile {
                            resource_type: res_type,
                            x: cx + x,
                            y: cy + y,
                        };
                        commands.spawn((create_resource_sprite(&asset_server, &tile), tile));
                        found = true;
                        break;
                    }
                }
            }
        }
    }

    commands.spawn((
        Player {
            inventory: vec![
                (items::BELT, 100),
                (items::INSERTER, 50),
                (items::FURNACE, 10),
                (items::MINER, 20),
            ],
        },
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
