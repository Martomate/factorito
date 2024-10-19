use std::f32::consts::PI;

use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(InputState { drag_start: None })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (move_player, update_camera).chain())
        .add_systems(Update, mouse_button_events)
        .run();
}

const PLAYER_SPEED: f32 = 200.;

#[derive(Resource)]
struct InputState {
    drag_start: Option<Vec2>,
}

fn mouse_button_events(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut input_state: ResMut<InputState>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();

    let Some(pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    else {
        // mouse is outside the window
        input_state.drag_start = None;
        return;
    };

    if let Some(drag_start) = input_state.drag_start {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let prev_x = drag_start.x;
        let prev_y = drag_start.y;
        let curr_x = pos.x;
        let curr_y = pos.y;

        let steps = ((curr_x - prev_x).abs().max((curr_y - prev_y).abs()) / 32.0 * 10.0) as i32 + 1;
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

    if buttons.pressed(MouseButton::Left) {
        input_state.drag_start = Some(pos);
    } else {
        input_state.drag_start = None;
    }
}

#[derive(Component)]
struct Player;

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
                        -1.0,
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

    commands.spawn(create_belt(
        &asset_server,
        texture_atlas_layout.clone(),
        5,
        3,
        0,
    ));
    commands.spawn(create_belt(
        &asset_server,
        texture_atlas_layout.clone(),
        4,
        -2,
        1,
    ));
    commands.spawn(create_belt(
        &asset_server,
        texture_atlas_layout.clone(),
        4,
        -3,
        2,
    ));
    commands.spawn(create_belt(
        &asset_server,
        texture_atlas_layout.clone(),
        4,
        -4,
        3,
    ));

    // Player
    commands.spawn((
        Player,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(5.)).into(),
            material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
            transform: Transform {
                translation: vec3(0., 0., 2.),
                ..default()
            },
            ..default()
        },
    ));
}

fn create_belt(
    asset_server: &Res<AssetServer>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    x: i32,
    y: i32,
    rotation: u8,
) -> impl Bundle {
    let belt_texture = asset_server.load("textures/items/belt.png");
    (
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(1.0))
                .with_rotation(Quat::from_rotation_z(PI / 2.0 * rotation as f32))
                .with_translation(vec3(x as f32 * 32.0, y as f32 * 32.0, 0.0)),
            texture: belt_texture.clone(),
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

    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    player.translation += move_delta.extend(0.);
}
