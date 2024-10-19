use std::f32::consts::PI;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
    window::PrimaryWindow,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(InputState::default())
        .insert_resource(GameWorld::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (move_player, update_camera).chain())
        .add_systems(
            Update,
            (
                mouse_button_events,
                handle_player_actions,
                update_preview_tile,
            ),
        )
        .add_systems(FixedUpdate, update_tiles)
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
}

impl TileType {
    const fn new(texture_name: &'static str) -> Self {
        Self { texture_name }
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
}

mod tiles {
    use super::TileType;

    pub static BELT: TileType = TileType::new("belt");
    pub static MINER: TileType = TileType::new("miner");
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

fn update_preview_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    input_state: Res<InputState>,
    q_preview_tile: Query<Entity, With<PreviewTile>>,
    game_world: Res<GameWorld>,
) {
    if let Ok(e) = q_preview_tile.get_single() {
        commands.entity(e).despawn();
    }

    if let Some(item) = input_state.item_in_hand {
        let tile = if item == items::BELT {
            Some(tiles::BELT)
        } else if item == items::MINER {
            Some(tiles::MINER)
        } else {
            None
        };

        if let Some(tile_type) = tile {
            let window = q_windows.single();
            let (camera, camera_transform) = q_camera.single();

            let mouse_pos = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                .map(|ray| ray.origin.truncate());

            if let Some(pos) = mouse_pos {
                let x = (pos.x / 32.0 + 0.5).floor() as i32;
                let y = (pos.y / 32.0 + 0.5).floor() as i32;

                if !game_world.tiles.contains_key(&(x, y)) {
                    commands.spawn((
                        create_preview_sprite(&asset_server, tile_type, x, y, input_state.rotation),
                        PreviewTile,
                    ));
                }
            }
        }
    }
}

fn handle_player_actions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    input_state: Res<InputState>,
    mut game_world: ResMut<GameWorld>,
    q_tiles: Query<(Entity, &PlacedTile)>,
    q_resource_tiles: Query<(Entity, &ResourceTile)>,
) {
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    let mouse_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    if let Some(pos) = mouse_pos {
        if input_state.dropping_items {
            let xx = pos.x / 32.0;
            let yy = pos.y / 32.0;

            let item = DroppedItem {
                item_type: items::IRON_SHEET,
            };
            commands.spawn((create_dropped_item(&asset_server, &item, xx, yy), item));
        }

        if input_state.deleting_tile {
            let xx = (pos.x / 32.0 + 0.5).floor() as i32;
            let yy = (pos.y / 32.0 + 0.5).floor() as i32;
            if let Some((entity, _)) = q_tiles
                .iter()
                .find(|(_, tile)| tile.x == xx && tile.y == yy)
            {
                game_world.tiles.remove(&(xx, yy));
                commands.entity(entity).despawn();
            } else if let Some((_, _)) = q_resource_tiles
                .iter()
                .find(|(_, t)| t.x == xx && t.y == yy)
            {
                let item = DroppedItem {
                    item_type: items::IRON_ORE,
                };
                commands.spawn((
                    create_dropped_item(
                        &asset_server,
                        &item,
                        (pos.x + 8.0) / 32.0,
                        (pos.y - 8.0) / 32.0,
                    ),
                    item,
                ));
            }
        }

        if let Some(drag_start) = input_state.drag_start {
            let prev_x = drag_start.x;
            let prev_y = drag_start.y;
            let curr_x = pos.x;
            let curr_y = pos.y;

            let dist_x = (curr_x - prev_x).abs();
            let dist_y = (curr_y - prev_y).abs();
            let steps = (dist_x.max(dist_y) / 32.0 * 10.0) as i32 + 1;

            let dx = (curr_x - prev_x) / steps as f32;
            let dy = (curr_y - prev_y) / steps as f32;

            for i in 0..=steps {
                let x = prev_x + i as f32 * dx;
                let y = prev_y + i as f32 * dy;
                let xx = (x / 32.0 + 0.5).floor() as i32;
                let yy = (y / 32.0 + 0.5).floor() as i32;

                if let Some(item_in_hand) = input_state.item_in_hand {
                    if !game_world.tiles.contains_key(&(xx, yy)) {
                        match item_in_hand {
                            i if i == items::BELT => {
                                let tile = PlacedTile {
                                    tile_type: tiles::BELT,
                                    rotation: input_state.rotation,
                                    x: xx,
                                    y: yy,
                                };
                                game_world.tiles.insert((xx, yy), tile.clone());

                                commands.spawn((create_tile_sprite(&asset_server, &tile), tile));
                            }
                            i if i == items::MINER => {
                                let tile = PlacedTile {
                                    tile_type: tiles::MINER,
                                    rotation: input_state.rotation,
                                    x: xx,
                                    y: yy,
                                };
                                game_world.tiles.insert((xx, yy), tile.clone());

                                commands.spawn((
                                    create_tile_sprite(&asset_server, &tile),
                                    tile,
                                    ResourceProducer {
                                        timer: Timer::from_seconds(1.0, TimerMode::Once),
                                        resource: resources::IRON_ORE,
                                    },
                                ));
                            }
                            _ => {}
                        };
                    }
                }
            }
        }
    }
}

fn update_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_tiles: Query<&PlacedTile>,
    mut q_items: Query<&mut Transform, With<DroppedItem>>,
    q_resource_tiles: Query<&ResourceTile>,
) {
    for tile in q_tiles.iter() {
        let tx = tile.x as f32 * 32.0;
        let ty = tile.y as f32 * 32.0 - 16.0;

        if tile.tile_type == tiles::BELT {
            for mut transform in q_items.iter_mut() {
                let item_pos = &mut transform.translation;

                if item_pos.x + 8.0 >= tx
                    && item_pos.x + 8.0 < tx + 32.0
                    && item_pos.y + 8.0 >= ty
                    && item_pos.y + 8.0 < ty + 32.0
                {
                    let dir = match tile.rotation {
                        0 => vec2(1.0, 0.0),
                        1 => vec2(0.0, 1.0),
                        2 => vec2(-1.0, 0.0),
                        3 => vec2(0.0, -1.0),
                        _ => unreachable!(),
                    };
                    transform.translation += dir.extend(0.0);
                }
            }
        } else if tile.tile_type == tiles::MINER {
            if let Some(res_tile) = q_resource_tiles
                .iter()
                .find(|t| t.x == tile.x && t.y == tile.y)
            {
                let dir = match tile.rotation {
                    0 => vec2(1.0, 0.0),
                    1 => vec2(0.0, 1.0),
                    2 => vec2(-1.0, 0.0),
                    3 => vec2(0.0, -1.0),
                    _ => unreachable!(),
                };

                let item_type = match res_tile.resource_type {
                    t if t == resources::IRON_ORE => items::IRON_ORE,
                    _ => unimplemented!(),
                };

                let item = DroppedItem { item_type };
                commands.spawn((
                    create_dropped_item(
                        &asset_server,
                        &item,
                        tile.x as f32 + 0.25 + dir.x * 0.75,
                        tile.y as f32 - 0.25 + dir.y * 0.75,
                    ),
                    item,
                ));
            }
        }
    }
}

fn mouse_button_events(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut input_state: ResMut<InputState>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    let mouse_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    if let Some(pos) = mouse_pos {
        if buttons.pressed(MouseButton::Left) {
            input_state.drag_start = Some(pos);
        } else {
            input_state.drag_start = None;
        }
        input_state.deleting_tile = buttons.pressed(MouseButton::Right);
    } else {
        // mouse is outside the window
        input_state.drag_start = None;
    }
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

fn create_dropped_item(
    asset_server: &Res<AssetServer>,
    item: &DroppedItem,
    x: f32,
    y: f32,
) -> impl Bundle {
    let item_texture = asset_server.load(format!(
        "textures/items/{}.png",
        item.item_type.texture_name
    ));
    SpriteBundle {
        transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(vec3(
            x * 32.0,
            y * 32.0,
            Layer::Item.depth(),
        )),
        texture: item_texture.clone(),
        ..default()
    }
}

fn create_preview_sprite(
    asset_server: &Res<AssetServer>,
    tile_type: TileType,
    x: i32,
    y: i32,
    rotation: u8,
) -> impl Bundle {
    let item_texture = asset_server.load(format!("textures/tiles/{}.png", tile_type.texture_name));

    SpriteBundle {
        transform: Transform::from_scale(Vec3::splat(1.0))
            .with_rotation(Quat::from_rotation_z(PI / 2.0 * rotation as f32))
            .with_translation(vec3(x as f32 * 32.0, y as f32 * 32.0, Layer::Tile.depth())),
        texture: item_texture.clone(),
        sprite: Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.7),
            ..Default::default()
        },
        ..default()
    }
}

fn create_tile_sprite(asset_server: &Res<AssetServer>, tile: &PlacedTile) -> impl Bundle {
    let item_texture = asset_server.load(format!(
        "textures/tiles/{}.png",
        tile.tile_type.texture_name
    ));
    SpriteBundle {
        transform: Transform::from_scale(Vec3::splat(1.0))
            .with_rotation(Quat::from_rotation_z(PI / 2.0 * tile.rotation as f32))
            .with_translation(vec3(
                tile.x as f32 * 32.0,
                tile.y as f32 * 32.0,
                Layer::Tile.depth(),
            )),
        texture: item_texture.clone(),
        ..default()
    }
}

fn create_resource_sprite(asset_server: &Res<AssetServer>, tile: &ResourceTile) -> impl Bundle {
    let item_texture = asset_server.load(format!(
        "textures/resources/{}.png",
        tile.resource_type.texture_name
    ));
    SpriteBundle {
        transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(vec3(
            tile.x as f32 * 32.0,
            tile.y as f32 * 32.0,
            Layer::Resource.depth(),
        )),
        texture: item_texture.clone(),
        ..default()
    }
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera.translation = camera
        .translation
        .lerp(direction, time.delta_seconds() * 4.0);
}

fn move_player(
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut input_state: ResMut<InputState>,
) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    if kb_input.pressed(KeyCode::Escape) {
        app_exit_events.send(bevy::app::AppExit::Success);
    }

    input_state.dropping_items = kb_input.pressed(KeyCode::KeyZ);

    if kb_input.just_pressed(KeyCode::KeyR) {
        input_state.rotation += 1;
        input_state.rotation %= 4;
    }

    if kb_input.just_pressed(KeyCode::KeyB) {
        input_state.item_in_hand = Some(items::BELT);
    }
    if kb_input.just_pressed(KeyCode::KeyM) {
        input_state.item_in_hand = Some(items::MINER);
    }
    if kb_input.just_pressed(KeyCode::KeyQ) {
        input_state.item_in_hand = None;
    }

    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    player.translation += move_delta.extend(0.);
}
