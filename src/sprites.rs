use std::f32::consts::PI;

use bevy::{
    math::vec3,
    prelude::*,
};

use crate::{calc_rotating_tile_transform, DroppedItem, Layer, PlacedTile, ResourceTile, TileType};

pub fn create_dropped_item(
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

pub fn create_preview_sprite(
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

pub fn create_rotating_preview_sprite(
    asset_server: &Res<AssetServer>,
    tile_type: TileType,
    x: i32,
    y: i32,
    rotation: u8,
    anchor: Vec2,
    start_angle: f32,
) -> impl Bundle {
    let item_texture = asset_server.load(format!("textures/tiles/{}.png", tile_type.rotating_texture_name.unwrap()));

    SpriteBundle {
        transform: calc_rotating_tile_transform(&PlacedTile { tile_type, rotation, x, y }, anchor, start_angle),
        texture: item_texture.clone(),
        sprite: Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.7),
            ..Default::default()
        },
        ..default()
    }
}

pub fn create_tile_sprite(asset_server: &Res<AssetServer>, tile: &PlacedTile) -> impl Bundle {
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

pub fn create_rotating_tile_sprite(
    asset_server: &Res<AssetServer>,
    tile: &PlacedTile,
    anchor: Vec2,
    start_angle: f32,
) -> impl Bundle {
    let item_texture = asset_server.load(format!(
        "textures/tiles/{}.png",
        tile.tile_type.rotating_texture_name.unwrap()
    ));

    SpriteBundle {
        transform: calc_rotating_tile_transform(tile, anchor, start_angle),
        texture: item_texture.clone(),
        ..default()
    }
}

pub fn create_resource_sprite(asset_server: &Res<AssetServer>, tile: &ResourceTile) -> impl Bundle {
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
