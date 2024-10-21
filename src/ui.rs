use bevy::prelude::*;

use crate::{InputState, ItemType, Player};

#[derive(Component)]
pub struct InventoryItem {
    idx: usize,
}

pub fn hanle_player_inventory_ui_events(
    mut input_state: ResMut<InputState>,
    q_player: Query<&Player>,
    mut q_inventory_item_int: Query<(&InventoryItem, &mut BackgroundColor, &Interaction), Changed<Interaction>>,
) {
    for (item, mut bg, interaction) in q_inventory_item_int.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                let inv = &q_player.single().inventory;
                if item.idx < inv.len() {
                    // TODO: remove from inventory
                    input_state.item_in_hand = Some(inv[item.idx].0);
                }
            }
            Interaction::Hovered => {
                bg.0 = Color::hsv(
                    0.0, 0.0, 0.4,
                );
            }
            Interaction::None => {
                bg.0 = Color::hsv(
                    0.0, 0.0, 0.3,
                );
            }
        }
    }
}

pub fn create_player_inventory_ui(
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    inventory: &[(ItemType, usize)],
) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for y in 0..9 {
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    align_items: AlignItems::FlexStart,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::hsv(0.0, 0.0, 0.2)), // dark gray
                                ..default()
                            })
                            .with_children(|parent| {
                                // A `NodeBundle` is used to display the logo the image as an `ImageBundle` can't automatically
                                // size itself with a child node present.
                                for x in 0..10 {
                                    let idx = y * 10 + x;
                                    let inv = if idx < inventory.len() {
                                        Some(inventory[idx])
                                    } else {
                                        None
                                    };
                                    parent
                                        .spawn((
                                            ButtonBundle {
                                                style: Style {
                                                    padding: UiRect::all(Val::Px(2.)),
                                                    margin: UiRect::all(Val::Px(1.)),
                                                    ..default()
                                                },
                                                background_color: BackgroundColor(Color::hsv(
                                                    0.0, 0.0, 0.3,
                                                )), // quite dark gray
                                                ..default()
                                            },
                                            InventoryItem { idx },
                                        ))
                                        .with_children(|parent| {
                                            if let Some((item_type, _count)) = inv {
                                                parent.spawn((
                                                    NodeBundle {
                                                        style: Style {
                                                            width: Val::Px(32.0),
                                                            height: Val::Px(32.0),
                                                            ..default()
                                                        },
                                                        ..default()
                                                    },
                                                    UiImage::new(asset_server.load(format!(
                                                        "textures/items/{}.png",
                                                        item_type.texture_name
                                                    ))),
                                                ));
                                            } else {
                                                parent
                                                    .spawn((NodeBundle {
                                                        style: Style {
                                                            width: Val::Px(32.0),
                                                            height: Val::Px(32.0),
                                                            ..default()
                                                        },
                                                        ..default()
                                                    },))
                                                    .with_children(|parent| {
                                                        parent.spawn((NodeBundle { ..default() },));
                                                    });
                                            }
                                        });
                                }
                            });
                    }
                });
        })
        .id()
}
