use std::f32::consts::PI;

use bevy::{math::vec2, prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::{
    calc_rotating_tile_transform, create_dropped_item_sprite, create_preview_sprite,
    create_rotating_preview_sprite, items, resources, tiles, DroppedItem, GameWorld, InputState,
    ItemMover, ItemProcessor, PlacedTile, PreviewTile, ResourceProducer, ResourceTile,
    TileRotation,
};

pub fn update_preview_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    input_state: Res<InputState>,
    q_preview_tiles: Query<Entity, With<PreviewTile>>,
    game_world: Res<GameWorld>,
) {
    for e in q_preview_tiles.iter() {
        commands.entity(e).despawn();
    }

    if let Some(item) = input_state.item_in_hand {
        let tile = if item == items::BELT {
            Some(tiles::BELT)
        } else if item == items::MINER {
            Some(tiles::MINER)
        } else if item == items::INSERTER {
            Some(tiles::INSERTER)
        } else if item == items::FURNACE {
            Some(tiles::FURNACE)
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
                    if tile_type.rotating_texture_name.is_some() {
                        let anchor = vec2(0.0, -0.5 + 3.0 / 32.0);
                        commands.spawn((
                            create_rotating_preview_sprite(
                                &asset_server,
                                tile_type,
                                x,
                                y,
                                input_state.rotation,
                                anchor,
                                PI * 0.5,
                            ),
                            PreviewTile,
                        ));
                    }
                }
            }
        }
    }
}

pub fn update_rotating_tiles(
    mut q_tiles: Query<(&mut Transform, &PlacedTile, &mut TileRotation)>,
    time: Res<Time>,
) {
    for (mut tr, tile, mut rot) in q_tiles.iter_mut() {
        rot.time += time.delta_seconds() * rot.speed;
        if rot.time > 1.0 {
            rot.time = 1.0;
        }
        *tr = calc_rotating_tile_transform(tile, rot.anchor, rot.from.lerp(rot.to, rot.time))
    }
}

const MIN_ITEM_DIST: f32 = 14.0;

pub fn update_tiles(
    q_tiles: Query<&PlacedTile>,
    mut q_items: Query<(Entity, &mut Transform), With<DroppedItem>>,
) {
    for tile in q_tiles.iter() {
        let tx = tile.x as f32 * 32.0;
        let ty = tile.y as f32 * 32.0 - 16.0;

        if tile.tile_type == tiles::BELT {
            let mut item_movements = HashMap::new();
            for (entity, transform) in q_items.iter() {
                let item_pos = &transform.translation;

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
                    let mut new_pos = transform.translation + dir.extend(0.0);

                    match tile.rotation {
                        0 | 2 => {
                            let mut dy = (new_pos.y + 8.0) % 32.0;
                            if dy < 0.0 {
                                dy += 32.0;
                            }
                            let dy = if dy < 8.0 {
                                (8.0 - dy).min(1.0)
                            } else if dy < 16.0 {
                                -(dy - 8.0).min(1.0)
                            } else if dy < 24.0 {
                                (24.0 - dy).min(1.0)
                            } else {
                                -(dy - 24.0).min(1.0)
                            };
                            new_pos.y += dy;
                        }
                        1 | 3 => {
                            let mut dx = (new_pos.x + 8.0) % 32.0;
                            if dx < 0.0 {
                                dx += 32.0;
                            }
                            let dx = if dx < 8.0 {
                                (8.0 - dx).min(1.0)
                            } else if dx < 16.0 {
                                -(dx - 8.0).min(1.0)
                            } else if dx < 24.0 {
                                (24.0 - dx).min(1.0)
                            } else {
                                -(dx - 24.0).min(1.0)
                            };
                            new_pos.x += dx;
                        }
                        _ => unreachable!(),
                    };

                    let mut total_movement = vec2(0.0, 0.0);
                    for v in 0..2 {
                        let movement = if v == 0 {
                            vec2(new_pos.x - transform.translation.x, 0.0)
                        } else {
                            vec2(0.0, new_pos.y - transform.translation.y)
                        };
                        let new_pos = transform.translation + movement.extend(0.0);
                        if q_items
                            .iter()
                            .filter(|(e, _)| *e != entity)
                            .map(|(_, tr)| tr.translation)
                            .all(|other| {
                                let dist_x_before = (other.x - item_pos.x).abs();
                                let dist_y_before = (other.y - item_pos.y).abs();
                                let dist_before = dist_x_before.max(dist_y_before);

                                let dist_x_after = (other.x - new_pos.x).abs();
                                let dist_y_after = (other.y - new_pos.y).abs();
                                let dist_after = dist_x_after.max(dist_y_after);

                                dist_after > MIN_ITEM_DIST || dist_after > dist_before - 1e-6
                            })
                        {
                            total_movement += movement;
                        }
                    }
                    item_movements.insert(entity, total_movement.extend(0.0));
                }
            }
            for (entity, mut transform) in q_items.iter_mut() {
                if let Some(movement) = item_movements.get(&entity) {
                    transform.translation += *movement;
                }
            }
        }
    }
}

pub fn update_movers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_movers: Query<(&mut ItemMover, &mut TileRotation, &Transform)>,
    q_items: Query<(Entity, &Transform, &DroppedItem)>,
    mut q_processors: Query<(&Transform, &mut ItemProcessor)>,
) {
    for (mut mover, mut rot, tr) in q_movers.iter_mut() {
        match mover.item {
            Some(item) => {
                if rot.time == 1.0 {
                    let pos = tr.transform_point(vec2(0.0, 1.0 * 32.0).extend(0.0));
                    if let Some((_, mut pr)) = q_processors
                        .iter_mut()
                        .find(|(tr, _)| tr.translation.distance_squared(pos) < 16.0 * 16.0)
                    {
                        if pr.item.is_none() {
                            let from = rot.to;
                            let to = rot.from;
                            rot.from = from;
                            rot.to = to;
                            rot.time = 0.0;

                            mover.item = None;
                            pr.item = Some(item);
                        }
                    } else if !q_items
                        .iter()
                        .map(|(_, tr, _)| tr.translation)
                        .any(|other| {
                            let dist_after = (other.x - pos.x).abs().max((other.y - pos.y).abs());
                            dist_after < MIN_ITEM_DIST
                        })
                    {
                        let from = rot.to;
                        let to = rot.from;
                        rot.from = from;
                        rot.to = to;
                        rot.time = 0.0;

                        let item = DroppedItem { item_type: item };
                        mover.item = None;

                        commands.spawn((
                            create_dropped_item_sprite(
                                &asset_server,
                                &item,
                                (pos.x + 8.0) / 32.0,
                                (pos.y - 8.0) / 32.0,
                            ),
                            item,
                        ));
                    }
                }
            }
            None => {
                if rot.time == 1.0 {
                    let pos = tr.transform_point(vec2(0.0, 1.0 * 32.0).extend(0.0));
                    const MIN_DIST: f32 = 0.5 * 32.0;
                    if let Some((e, _, it)) = q_items.iter().find(|(_, tr, _)| {
                        tr.translation.distance_squared(pos) < MIN_DIST * MIN_DIST
                    }) {
                        commands.entity(e).despawn_recursive();

                        let from = rot.to;
                        let to = rot.from;
                        rot.from = from;
                        rot.to = to;
                        rot.time = 0.0;

                        mover.item = Some(it.item_type);
                    } else if let Some((_, mut pr)) = q_processors
                        .iter_mut()
                        .find(|(tr, _)| tr.translation.distance_squared(pos) < 16.0 * 16.0)
                    {
                        if let Some((item_type, count)) = pr.output {
                            if count > 0 {
                                pr.output = if count > 1 {
                                    Some((item_type, count - 1))
                                } else {
                                    None
                                };

                                let from = rot.to;
                                let to = rot.from;
                                rot.from = from;
                                rot.to = to;
                                rot.time = 0.0;

                                mover.item = Some(item_type);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn update_miners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_tiles: Query<(&PlacedTile, &mut ResourceProducer)>,
    q_items: Query<&Transform, With<DroppedItem>>,
    q_resource_tiles: Query<&ResourceTile>,
    time: Res<Time>,
) {
    for (tile, mut producer) in q_tiles.iter_mut() {
        producer.timer.tick(time.delta());

        if producer.timer.finished() {
            producer.timer.reset();

            if let Some(res_tile) = q_resource_tiles
                .iter()
                .find(|t| t.x == tile.x && t.y == tile.y)
            {
                if res_tile.resource_type == producer.resource {
                    let dir = match tile.rotation {
                        0 => vec2(1.0, 0.0),
                        1 => vec2(0.0, 1.0),
                        2 => vec2(-1.0, 0.0),
                        3 => vec2(0.0, -1.0),
                        _ => unreachable!(),
                    };

                    let item_type = match res_tile.resource_type {
                        t if t == resources::COAL => items::COAL,
                        t if t == resources::IRON_ORE => items::IRON_ORE,
                        t if t == resources::COPPER_ORE => items::COPPER_ORE,
                        _ => unimplemented!(),
                    };

                    let item = DroppedItem { item_type };
                    let pos = vec2(
                        tile.x as f32 + 0.25 + dir.x * 0.75,
                        tile.y as f32 - 0.25 + dir.y * 0.75,
                    );
                    if !q_items.iter().map(|tr| tr.translation).any(|other| {
                        let dist_after = (other.x - pos.x * 32.0)
                            .abs()
                            .max((other.y - pos.y * 32.0).abs());
                        dist_after < MIN_ITEM_DIST
                    }) {
                        commands.spawn((
                            create_dropped_item_sprite(&asset_server, &item, pos.x, pos.y),
                            item,
                        ));
                    }
                }
            }
        }
    }
}

pub fn update_item_processors(mut q_processors: Query<&mut ItemProcessor>, time: Res<Time>) {
    for mut processor in q_processors.iter_mut() {
        if processor.timer.tick(time.delta()).finished() {
            if let Some(item_type) = processor.item {
                if let Some(output_item_type) = match item_type {
                    t if t == items::IRON_ORE => Some(items::IRON_SHEET),
                    t if t == items::COPPER_ORE => Some(items::COPPER_SHEET),
                    _ => None,
                } {
                    let new_output = if let Some(output) = processor.output {
                        if output.0 == output_item_type {
                            Some((output_item_type, output.1 + 1))
                        } else {
                            None
                        }
                    } else {
                        Some((output_item_type, 1))
                    };
                    if let Some(new_output) = new_output {
                        processor.item = None;
                        processor.output = Some(new_output);
                        processor.timer.reset();
                    }
                }
            }
        }
    }
}
