use bevy::{prelude::*, window::PrimaryWindow};

use crate::{items, InputState, Player, PLAYER_SPEED};

pub fn update_camera(
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

pub fn mouse_button_events(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut input_state: ResMut<InputState>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if input_state.inventory_ui.is_some() {
        return;
    }

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

pub fn move_player(
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

    input_state.toggling_inventory_visible = kb_input.just_pressed(KeyCode::KeyE);

    if kb_input.just_pressed(KeyCode::KeyB) {
        input_state.item_in_hand = Some(items::BELT);
    }
    if kb_input.just_pressed(KeyCode::KeyM) {
        input_state.item_in_hand = Some(items::MINER);
    }
    if kb_input.just_pressed(KeyCode::KeyI) {
        input_state.item_in_hand = Some(items::INSERTER);
    }
    if kb_input.just_pressed(KeyCode::KeyF) {
        input_state.item_in_hand = Some(items::FURNACE);
    }

    if kb_input.just_pressed(KeyCode::KeyQ) {
        input_state.item_in_hand = None;
    }

    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    player.translation += move_delta.extend(0.);
}
