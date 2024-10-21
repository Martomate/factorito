use std::f32::consts::PI;

use bevy::{math::vec2, prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::{
    calc_rotating_tile_transform, create_dropped_item, create_preview_sprite, create_rotating_preview_sprite, items, resources, tiles, DroppedItem, GameWorld, InputState, ItemMover, PlacedTile, PreviewTile, ResourceProducer, ResourceTile, TileRotation
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
                            create_rotating_preview_sprite(&asset_server, tile_type, x, y, input_state.rotation, anchor, PI * 0.5),
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

const MIN_ITEM_DIST: f32 = 16.0;

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
                    let new_pos = transform.translation + dir.extend(0.0);
                    if !q_items
                        .iter()
                        .filter(|(e, _)| *e != entity)
                        .map(|(_, tr)| tr.translation)
                        .any(|other| {
                            let dist_before = (other.x - item_pos.x)
                                .abs()
                                .max((other.y - item_pos.y).abs());
                            let dist_after =
                                (other.x - new_pos.x).abs().max((other.y - new_pos.y).abs());
                            dist_after < MIN_ITEM_DIST && dist_after - dist_before < 1e-6
                        })
                    {
                        item_movements.insert(entity, dir.extend(0.0));
                    }
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
) {
    for (mut mover, mut rot, tr) in q_movers.iter_mut() {
        match mover.item {
            Some(item) => {
                if rot.time == 1.0 {
                    let pos = tr.transform_point(vec2(0.0, 1.0 * 32.0).extend(0.0));
                    if !q_items
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
                            create_dropped_item(
                                &asset_server,
                                &item,
                                (pos.x - 8.0) / 32.0,
                                (pos.y - 8.0) / 32.0,
                            ),
                            item,
                        ));
                    }
                }
            }
            None => {
                if rot.time == 1.0 {
                    let pos = tr.transform_point(vec2(0.0, 0.0 * 32.0).extend(0.0))
                        - vec2(8.0, 8.0).extend(0.0);
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
                        t if t == resources::IRON_ORE => items::IRON_ORE,
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
                            create_dropped_item(&asset_server, &item, pos.x, pos.y),
                            item,
                        ));
                    }
                }
            }
        }
    }
}
