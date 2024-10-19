use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(InputState { drag_start: None, dropping_items: false })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (move_player, update_camera).chain())
        .add_systems(Update, (mouse_button_events, handle_player_actions))
        .run();
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

mod items {
    use super::ItemType;

    pub static IRON_SHEET: ItemType = ItemType::new("iron_sheet");
}

mod tiles {
    use super::TileType;

    pub static BELT: TileType = TileType::new("belt");
}

const PLAYER_SPEED: f32 = 200.;

#[derive(Resource)]
struct InputState {
    drag_start: Option<Vec2>,
    dropping_items: bool,
}

fn handle_player_actions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    input_state: ResMut<InputState>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

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
            commands.spawn(create_dropped_item(
                &asset_server,
                texture_atlas_layout.clone(),
                items::IRON_SHEET,
                xx,
                yy,
            ));
        }

        if let Some(drag_start) = input_state.drag_start {
            let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);

            let prev_x = drag_start.x;
            let prev_y = drag_start.y;
            let curr_x = pos.x;
            let curr_y = pos.y;

            let steps =
                ((curr_x - prev_x).abs().max((curr_y - prev_y).abs()) / 32.0 * 10.0) as i32 + 1;
            let dx = (curr_x - prev_x) / steps as f32;
            let dy = (curr_y - prev_y) / steps as f32;

            for i in 0..=steps {
                let x = prev_x + i as f32 * dx;
                let y = prev_y + i as f32 * dy;
                let xx = (x / 32.0 + 0.5).floor() as i32;
                let yy = (y / 32.0 + 0.5).floor() as i32;

                commands.spawn(create_belt(
                    &asset_server,
                    texture_atlas_layout.clone(),
                    xx,
                    yy,
                    0,
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
    } else {
        // mouse is outside the window
        input_state.drag_start = None;
    }
}

enum Layer {
    Background,
    Tile,
    Item,
    Player,
}

impl Layer {
    fn depth(&self) -> f32 {
        match self {
            Layer::Background => 0.0,
            Layer::Tile => 0.1,
            Layer::Item => 0.2,
            Layer::Player => 0.3,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DroppedItem {
    item_type: ItemType,
}

#[derive(Component)]
struct PlacedTile {
    tile_type: TileType,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let dirt_texture = asset_server.load("textures/bg/dirt.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn(Camera2dBundle::default());

    for y in -100..=100 {
        for x in -100..=100 {
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(vec3(
                        x as f32 * 32.0,
                        y as f32 * 32.0,
                        Layer::Background.depth(),
                    )),
                    texture: dirt_texture.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                },
            ));
        }
    }

    commands.spawn(create_iron_sheet(
        &asset_server,
        texture_atlas_layout.clone(),
        5.0,
        3.0,
    ));

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

fn create_iron_sheet(
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    x: f32,
    y: f32,
) -> impl Bundle {
    create_dropped_item(asset_server, texture_atlas_layout, items::IRON_SHEET, x, y)
}

fn create_belt(
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    x: i32,
    y: i32,
    rotation: u8,
) -> impl Bundle {
    create_placed_tile(
        asset_server,
        texture_atlas_layout,
        tiles::BELT,
        x as f32,
        y as f32,
        rotation,
    )
}

fn create_dropped_item(
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    item_type: ItemType,
    x: f32,
    y: f32,
) -> impl Bundle {
    let item_texture = asset_server.load(format!("textures/items/{}.png", item_type.texture_name));
    (
        DroppedItem { item_type },
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(1.0)).with_translation(vec3(
                x * 32.0,
                y * 32.0,
                Layer::Item.depth(),
            )),
            texture: item_texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
    )
}

fn create_placed_tile(
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    tile_type: TileType,
    x: f32,
    y: f32,
    rotation: u8,
) -> impl Bundle {
    let item_texture = asset_server.load(format!("textures/items/{}.png", tile_type.texture_name));
    (
        PlacedTile { tile_type },
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(1.0))
                .with_rotation(Quat::from_rotation_z(PI / 2.0 * rotation as f32))
                .with_translation(vec3(x * 32.0, y * 32.0, Layer::Tile.depth())),
            texture: item_texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
    )
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

    // Applies a smooth effect to camera movement using interpolation between
    // the camera position and the player position on the x and y axes.
    // Here we use the in-game time, to get the elapsed time (in seconds)
    // since the previous update. This avoids jittery movement when tracking
    // the player.
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

    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    player.translation += move_delta.extend(0.);
}
