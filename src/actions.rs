use std::f32::consts::PI;

use bevy::{math::vec2, prelude::*, window::PrimaryWindow};

use crate::{
    create_dropped_item, create_rotating_tile_sprite, create_tile_sprite, items, resources, tiles,
    DroppedItem, GameWorld, InputState, ItemMover, PlacedTile, ResourceProducer, ResourceTile,
    TileRotation,
};

pub fn handle_player_actions(
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
                            i if i == items::INSERTER => {
                                let tile = PlacedTile {
                                    tile_type: tiles::INSERTER,
                                    rotation: input_state.rotation,
                                    x: xx,
                                    y: yy,
                                };
                                game_world.tiles.insert((xx, yy), tile.clone());

                                let anchor = vec2(0.0, -0.5 + 3.0 / 32.0);

                                let main_sprite = create_tile_sprite(&asset_server, &tile);
                                let rotating_sprite = create_rotating_tile_sprite(
                                    &asset_server,
                                    &tile,
                                    anchor,
                                    PI * 0.5,
                                );

                                commands.spawn((tile.clone(), main_sprite));
                                commands.spawn((
                                    tile,
                                    rotating_sprite,
                                    TileRotation {
                                        anchor,
                                        speed: 2.0,
                                        from: -PI * 0.5,
                                        to: PI * 0.5,
                                        time: 1.0,
                                    },
                                    ItemMover { item: None },
                                ));
                            }
                            i if i == items::FURNACE => {
                                let tile = PlacedTile {
                                    tile_type: tiles::FURNACE,
                                    rotation: input_state.rotation,
                                    x: xx,
                                    y: yy,
                                };
                                game_world.tiles.insert((xx, yy), tile.clone());

                                commands.spawn((
                                    tile.clone(),
                                    create_tile_sprite(&asset_server, &tile),
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
