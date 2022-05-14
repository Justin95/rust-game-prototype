
use bevy::prelude::*;
use bevy::render::camera::DepthCalculation;
use bevy::render::primitives::Frustum;
use bevy_flycam::{FlyCam, MovementSettings};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MovementSettings {
            speed: 40.0,
            ..default()
        })
            .add_startup_system(initialize);
            //.add_system(manage_player);
    }
}

const FAR_VIEW: f32 = 2000.0;
const NEAR_VIEW: f32 = 0.2;
const FOV: f32 = 90.0;
const ASPECT_RATIO: f32 = 16.0 / 9.0;

#[derive(Component, Default)]
pub struct Player {
    
}

fn initialize(mut commands: Commands) {
    let perspective_projection = PerspectiveProjection {
        fov: FOV.to_radians(),
        near: NEAR_VIEW,
        far: FAR_VIEW,
        aspect_ratio: ASPECT_RATIO,
    };
    let view_projection = Mat4::perspective_infinite_reverse_rh(FOV.to_radians(), ASPECT_RATIO, NEAR_VIEW);
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &Vec3::ZERO,
        &Vec3::Z,
        FAR_VIEW,
    );
    
    commands.spawn()
        .insert(Player {
            
        })
        .insert_bundle(TransformBundle{
            local: Transform{
                translation: Vec3::new(0.0, 100.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(FlyCam)
        .with_children(|parent| {
            parent.spawn_bundle(PerspectiveCameraBundle {
                camera: Camera {
                    near: perspective_projection.near,
                    far: perspective_projection.far,
                    depth_calculation: DepthCalculation::Distance,
                    ..Default::default()
                },
                perspective_projection,
                frustum,
                transform: Transform {
                    translation: Vec3::new(0.0, 1.8, 0.0),
                    ..default()
                },
                ..default()
            });
        });
}
/*
fn manage_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>
) {
    
    for mut transform in player_query.iter_mut() {
        transform.rotate(Quat::from_rotation_y(0.01));
    }
    
}*/