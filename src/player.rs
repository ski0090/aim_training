use crate::GameState;
use crate::actions::Actions;
use crate::audio::FireRequest;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)))
            .add_systems(Update, handle_fire.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                despawn_hit_marker.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands) {
    // Spawn player with a 3D camera
    commands
        .spawn((
            Player,
            Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::default(),
                Projection::from(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }),
            ));
        });
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mouse_sensitivity = 0.002;
    let speed = 6.0;

    for mut player_transform in &mut player_query {
        // Rotation (Yaw and Pitch)
        // Note: For a proper FPS controller, we usually separate Yaw (Player Body) and Pitch (Camera).
        // For simplicity, we'll just rotate the player entity for now, or we can handle pitch if we had the camera query separately.
        // Let's implement basic free-cam style rotation for the Player entity for now, or planar movement + yaw.

        if let Some(rotation) = actions.player_rotation {
            // Yaw (around Y axis)
            player_transform.rotate_axis(Dir3::Y, -rotation.x * mouse_sensitivity);

            // Pitch (around local X axis) - Limit this in a real implementation to avoid flipping
            let right = player_transform.right();
            if let Ok(right_dir) = Dir3::new(*right) {
                player_transform.rotate_axis(right_dir, -rotation.y * mouse_sensitivity);
            }
        }

        if let Some(movement) = actions.player_movement {
            // Calculate movement direction relative to player's rotation
            let forward = player_transform.forward();
            let right = player_transform.right();

            // Allow movement only on the XZ plane for "walking" feel, or full free cam?
            // "FPS" usually means walking on ground. Let's zero out Y component of forward/right for movement direction.
            let mut forward_plane = *forward;
            forward_plane.y = 0.0;
            forward_plane = forward_plane.normalize_or_zero();

            let mut right_plane = *right;
            right_plane.y = 0.0;
            right_plane = right_plane.normalize_or_zero();

            let move_dir =
                (forward_plane * movement.y + right_plane * movement.x).normalize_or_zero();

            player_transform.translation += move_dir * speed * time.delta_secs();
        }
    }
}

#[derive(Component)]
struct HitMarker;

#[derive(Component)]
struct Lifetime {
    timer: Timer,
}

fn handle_fire(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fire_request: ResMut<FireRequest>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok((_camera, camera_transform)) = q_camera.single() {
            if let Ok(_window) = q_window.single() {
                // In 3D FPS, cursor is usually locked. We should use center of screen or camera forward.
                // For now, assuming cursor-based or center if locked.
                // If cursor is locked, cursor_position might be None or center.
                // Let's assume we want to shoot forward from camera.

                let ray_origin = camera_transform.translation();
                let ray_dir = camera_transform.forward();
                let distance = 10.0;
                let hit_point = ray_origin + ray_dir * distance;

                // Spawn a visual indicator (HitMarker)
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.1))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.0, 0.0),
                        emissive: LinearRgba::new(1.0, 0.0, 0.0, 1.0),
                        ..default()
                    })),
                    Transform::from_translation(hit_point),
                    HitMarker,
                    Lifetime {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                ));

                // Play fire sound via resource flag
                fire_request.0 = true;
            }
        }
    }
}

fn despawn_hit_marker(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), With<HitMarker>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
