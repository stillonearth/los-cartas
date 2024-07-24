use std::time::Duration;

use bevy::prelude::*;
use bevy_la_mesa::{
    events::RenderDeck, Chip, ChipArea, DeckArea, HandArea, LaMesaPluginSettings, PlayArea,
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::game::cards::{ChipType, DropChip, GameState, Kard, MoveChip};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.observe(spawn_board);
    app.add_systems(
        Update,
        (
            handle_drop_chip,
            handle_move_chip_to_sales,
            render_hand_area,
        ),
    );
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    // commands.trigger(SpawnPlayer);
}

#[derive(Event, Debug)]
pub struct SpawnBoard;

#[derive(Component)]
pub struct ResourceArea {
    pub marker: usize,
    pub player: usize,
}

fn spawn_board(
    _trigger: Trigger<SpawnBoard>,
    mut commands: Commands,
    mut ew_render_deck: EventWriter<RenderDeck>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 4.0, 4.0),
        ..default()
    });

    // Deck Area
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: materials.add(Color::BLACK),
            transform: Transform::from_translation(Vec3::new(-7.6, 0.0, 0.0))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0)),
            ..default()
        },
        DeckArea,
        Name::new("Deck"),
    ));

    // Resources - Production
    let face_texture = asset_server.load("tarjetas/resources-sales.png");
    let face_material = materials.add(StandardMaterial {
        base_color_texture: Some(face_texture.clone()),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(2.5 * 1.2, 3.5 * 1.2)
                    .subdivisions(10),
            ),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.2, 0.0, 3.5 * 1.2 / 2.0 + 0.1)),
            ..default()
        },
        ResourceArea {
            marker: 1,
            player: 1,
        },
        Name::new("Resources - Production - Player 1"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(2.5 * 1.2, 3.5 * 1.2)
                    .subdivisions(10),
            ),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.2, 0.0, -(3.5 * 1.2 / 2.0 + 0.1)))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ..default()
        },
        ResourceArea {
            marker: 1,
            player: 2,
        },
        Name::new("Resources - Production - Player 2"),
    ));

    // // Resources - Sales
    let face_texture = asset_server.load("tarjetas/resources-production.png");
    let face_material = materials.add(StandardMaterial {
        base_color_texture: Some(face_texture.clone()),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(2.5 * 1.2, 3.5 * 1.2)
                    .subdivisions(10),
            ),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(4.5, 0.0, 3.5 * 1.2 / 2.0 + 0.1)),
            ..default()
        },
        ResourceArea {
            marker: 2,
            player: 1,
        },
        Name::new("Resources - Sales - Player 1"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(2.5 * 1.2, 3.5 * 1.2)
                    .subdivisions(10),
            ),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(4.5, 0.0, -(3.5 * 1.2 / 2.0 + 0.1))),
            ..default()
        },
        ResourceArea {
            marker: 2,
            player: 2,
        },
        Name::new("Resources - Sales - Player 2"),
    ));

    // Play Area 1
    let face_texture = asset_server.load("tarjetas/debug.png");
    let face_material = materials.add(StandardMaterial {
        base_color_texture: Some(face_texture.clone()),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(-7.6, 0.0, -0.4)),
            visibility: Visibility::Hidden,
            ..default()
        },
        PlayArea { marker: 1 },
        Name::new("Play Area 1"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(-7.6 + 3.05, 0.0, -0.4)),
            visibility: Visibility::Hidden,
            ..default()
        },
        PlayArea { marker: 2 },
        Name::new("Play Area 2"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(-7.6 + 3.05 * 2.0, 0.0, -0.4)),
            visibility: Visibility::Hidden,
            ..default()
        },
        PlayArea { marker: 3 },
        Name::new("Play Area 3"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(-7.6 + 3.05 * 3.0, 0.0, -0.4)),
            visibility: Visibility::Hidden,
            ..default()
        },
        PlayArea { marker: 4 },
        Name::new("Play Area 4"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(10)),
            material: face_material.clone(),
            transform: Transform::from_translation(Vec3::new(-7.6 + 3.05 * 4.0, 0.0, -0.4)),
            visibility: Visibility::Hidden,
            ..default()
        },
        PlayArea { marker: 5 },
        Name::new("Play Area 5"),
    ));

    ew_render_deck.send(RenderDeck);
}

pub fn handle_drop_chip(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut er_drop_chip: EventReader<DropChip>,
    query: Query<(Entity, &ChipArea, &Chip<ChipType>)>,
) {
    for drop_chip in er_drop_chip.read() {
        let num_chips_of_kind = query
            .iter()
            .filter(|(_, area, chip)| area.marker == 1 && chip.data == drop_chip.chip_type)
            .count();

        let model = match drop_chip.chip_type {
            ChipType::Cannabis => asset_server
                .load("models/chip-cannabis/chip_for_tabletop_gam_0723233549_preview.obj"),
            ChipType::Cocaine => {
                asset_server.load("models/chip-cocaine/chip_for_tabletop_gam_0723233917_refine.obj")
            }
        };

        let initial_translation = match drop_chip.chip_type {
            ChipType::Cannabis => Transform::from_xyz(0.6, 12.0, -5.2).with_scale(Vec3::ONE * 1.0),
            ChipType::Cocaine => Transform::from_xyz(1.8, 12.0, -3.6).with_scale(Vec3::ONE * 1.0),
        }
        .translation;
        let mut final_translation = initial_translation.clone();
        final_translation.y = num_chips_of_kind as f32 * 0.2;

        let tween: Tween<Transform> = Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(350),
            TransformPositionLens {
                start: initial_translation,
                end: final_translation,
            },
        );

        commands.spawn((
            SceneBundle {
                scene: model,
                transform: match drop_chip.chip_type {
                    ChipType::Cannabis => {
                        Transform::from_xyz(0.6, 12.0, -5.2).with_scale(Vec3::ONE * 1.0)
                    }
                    ChipType::Cocaine => {
                        Transform::from_xyz(1.8, 12.0, -3.6).with_scale(Vec3::ONE * 1.0)
                    }
                },
                ..default()
            },
            Name::new("Chip"),
            Chip::<ChipType> {
                data: drop_chip.chip_type,
            },
            ChipArea {
                marker: drop_chip.area,
            },
            Animator::new(tween),
        ));
    }
}

pub fn handle_move_chip_to_sales(
    mut commands: Commands,
    mut er_move_chip: EventReader<MoveChip>,
    query: Query<(Entity, &Transform, &ChipArea, &Chip<ChipType>)>,
) {
    for move_chip in er_move_chip.read() {
        let chip = query.get(move_chip.entity).unwrap();
        let chip_type = chip.3.data;
        let initial_translation = chip.1.translation;
        let num_chips_of_kind = query
            .iter()
            .filter(|(_, _, area, chip)| area.marker == move_chip.area && chip.data == chip_type)
            .count();

        let mut final_translation = initial_translation.clone();
        final_translation.x = match chip_type {
            ChipType::Cannabis => 3.9,
            ChipType::Cocaine => 5.1,
        };

        final_translation.z = match chip_type {
            ChipType::Cannabis => -5.2,
            ChipType::Cocaine => -3.6,
        };

        final_translation.y = num_chips_of_kind as f32 * 0.2;

        let tween: Tween<Transform> = Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(350),
            TransformPositionLens {
                start: initial_translation,
                end: final_translation,
            },
        );

        commands
            .entity(move_chip.entity)
            .insert(Animator::new(tween))
            .insert(ChipArea {
                marker: move_chip.area,
            });
    }
}

pub fn render_hand_area(mut commands: Commands) {
    commands.spawn((
        Name::new("HandArea - Player 1"),
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 0.0, 5.8))
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 4.0)),
            ..default()
        },
        HandArea { player: 1 },
    ));
}