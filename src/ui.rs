use bevy::prelude::*;

use crate::{InputState, ItemType, Player};

const COLOR_ITEM_BORDER: Color = Color::hsv(0.0, 0.0, 0.2);
const COLOR_ITEM_BG_NORMAL: Color = Color::hsv(0.0, 0.0, 0.25);
const COLOR_ITEM_BG_HOVER: Color = Color::hsv(0.0, 0.0, 0.35);

#[derive(Component)]
pub struct InventoryItem {
    idx: usize,
}

#[derive(Component)]
pub struct CraftableItem {
    idx: usize,
}

pub fn hanle_player_inventory_ui_events(
    mut input_state: ResMut<InputState>,
    q_player: Query<&Player>,
    mut q_inventory_item_int: Query<
        (&InventoryItem, &mut BackgroundColor, &Interaction),
        Changed<Interaction>,
    >,
    mut q_craftable_item_int: Query<
        (&CraftableItem, &mut BackgroundColor, &Interaction),
        (Changed<Interaction>, Without<InventoryItem>),
    >,
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
                bg.0 = COLOR_ITEM_BG_HOVER;
            }
            Interaction::None => {
                bg.0 = COLOR_ITEM_BG_NORMAL;
            }
        }
    }

    for (item, mut bg, interaction) in q_craftable_item_int.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                println!("Crafting slot {} clicked", item.idx)
            }
            Interaction::Hovered => {
                bg.0 = COLOR_ITEM_BG_HOVER;
            }
            Interaction::None => {
                bg.0 = COLOR_ITEM_BG_NORMAL;
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
                                background_color: BackgroundColor(COLOR_ITEM_BORDER),
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
                                                background_color: BackgroundColor(
                                                    COLOR_ITEM_BG_NORMAL,
                                                ),
                                                ..default()
                                            },
                                            InventoryItem { idx },
                                        ))
                                        .with_children(|parent| {
                                            if let Some((item_type, count)) = inv {
                                                parent
                                                    .spawn((
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
                                                    ))
                                                    .with_children(|parent| {
                                                        parent
                                                            .spawn(NodeBundle {
                                                                style: Style {
                                                                    width: Val::Percent(100.0),
                                                                    height: Val::Percent(100.0),
                                                                    justify_content:
                                                                        JustifyContent::End,
                                                                    align_items: AlignItems::End,
                                                                    ..default()
                                                                },
                                                                ..default()
                                                            })
                                                            .with_children(|parent| {
                                                                create_outlined_text(
                                                                    parent,
                                                                    format!("{}", count),
                                                                );
                                                            });
                                                    });
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

            parent.spawn(NodeBundle {
                style: Style {
                    margin: UiRect::horizontal(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::Stretch,
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            background_color: BackgroundColor(COLOR_ITEM_BORDER),
                            ..default()
                        })
                        .with_children(|parent| {
                            for x in 0..4 {
                                let category_label = match x {
                                    0 => "Logistics",
                                    1 => "Production",
                                    2 => "Intermediate",
                                    3 => "Combat",
                                    _ => unreachable!(),
                                };
                                let is_selected = x == 0; // TODO: use state for this
                                parent
                                    .spawn((ButtonBundle {
                                        style: Style {
                                            height: Val::Px(32.0 + 4.0),
                                            padding: UiRect::all(Val::Px(2.)),
                                            margin: UiRect::all(Val::Px(1.)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            flex_grow: 1.0,
                                            ..default()
                                        },
                                        background_color: BackgroundColor(if is_selected {
                                            COLOR_ITEM_BG_HOVER
                                        } else {
                                            COLOR_ITEM_BG_NORMAL
                                        }),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent
                                            .spawn((NodeBundle {
                                                style: Style { ..default() },
                                                ..default()
                                            },))
                                            .with_children(|parent| {
                                                parent
                                                    .spawn(NodeBundle {
                                                        style: Style {
                                                            width: Val::Percent(100.0),
                                                            height: Val::Percent(100.0),
                                                            justify_content: JustifyContent::End,
                                                            align_items: AlignItems::End,
                                                            ..default()
                                                        },
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        create_outlined_text(
                                                            parent,
                                                            category_label.to_string(),
                                                        );
                                                    });
                                            });
                                    });
                            }
                        });

                    for y in 0..8 {
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    align_items: AlignItems::FlexStart,
                                    ..default()
                                },
                                background_color: BackgroundColor(COLOR_ITEM_BG_NORMAL),
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
                                                background_color: BackgroundColor(
                                                    COLOR_ITEM_BG_NORMAL,
                                                ),
                                                ..default()
                                            },
                                            CraftableItem { idx },
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

fn create_outlined_text(parent: &mut ChildBuilder<'_>, text_str: String) {
    parent
        .spawn(NodeBundle {
            style: Style { ..default() },
            ..default()
        })
        .with_children(|parent| {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    parent.spawn(
                        TextBundle::from_section(
                            text_str.clone(),
                            TextStyle {
                                color: Color::hsv(0.0, 0.0, 0.0),
                                font_size: 14.0,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px((dx + 1) as f32),
                            bottom: Val::Px((dy + 1) as f32),
                            ..default()
                        }),
                    );
                }
            }
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::right(Val::Px(1.0)).with_bottom(Val::Px(1.0)),
                    ..default()
                },
                text: Text::from_section(
                    text_str,
                    TextStyle {
                        color: Color::hsv(0.0, 0.0, 0.8),
                        font_size: 14.0,
                        ..default()
                    },
                ),
                ..default()
            });
        });
}
