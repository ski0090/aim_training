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

#[derive(Component)]
pub struct WeaponState {
    pub fire_cooldown: f32,
    pub recoil_pitch: f32,
    pub recoil_yaw: f32,
    pub target_recoil_pitch: f32,
    pub target_recoil_yaw: f32,
    pub original_fov: f32,
    pub ads_fov: f32,
    pub is_ads: bool,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            fire_cooldown: 0.0,
            recoil_pitch: 0.0,
            recoil_yaw: 0.0,
            target_recoil_pitch: 0.0,
            target_recoil_yaw: 0.0,
            original_fov: 90.0_f32.to_radians(),
            ads_fov: 45.0_f32.to_radians(),
            is_ads: false,
        }
    }
}

fn spawn_player(mut commands: Commands) {
    // Spawn player with a 3D camera
    commands
        .spawn((
            Player,
            WeaponState::default(),
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
    mut player_query: Query<(&mut Transform, &mut WeaponState), With<Player>>,
    mut camera_query: Query<&mut Projection, With<Camera3d>>,
) {
    let dt = time.delta_secs();
    let base_mouse_sensitivity = 0.002;
    let speed = 6.0;

    for (mut player_transform, mut weapon_state) in &mut player_query {
        // Handle ADS FOV interpolation
        if let Ok(mut projection) = camera_query.single_mut() {
            if let Projection::Perspective(ref mut perspective) = *projection {
                let target_fov = if weapon_state.is_ads {
                    weapon_state.ads_fov
                } else {
                    weapon_state.original_fov
                };
                perspective.fov = perspective.fov.lerp(target_fov, dt * 10.0);
            }
        }

        // Adjust sensitivity based on ADS
        let mouse_sensitivity = if weapon_state.is_ads {
            base_mouse_sensitivity * 0.5
        } else {
            base_mouse_sensitivity
        };

        // Recoil Recovery
        weapon_state.recoil_pitch = weapon_state.recoil_pitch.lerp(0.0, dt * 5.0);
        weapon_state.recoil_yaw = weapon_state.recoil_yaw.lerp(0.0, dt * 5.0);
        weapon_state.target_recoil_pitch = weapon_state.target_recoil_pitch.lerp(0.0, dt * 5.0);
        weapon_state.target_recoil_yaw = weapon_state.target_recoil_yaw.lerp(0.0, dt * 5.0);

        // Rotation (Yaw and Pitch) with Recoil
        if let Some(rotation) = actions.player_rotation {
            // Yaw (around Y axis)
            player_transform.rotate_axis(Dir3::Y, -rotation.x * mouse_sensitivity);

            // Pitch (around local X axis)
            let right = player_transform.right();
            if let Ok(right_dir) = Dir3::new(*right) {
                player_transform.rotate_axis(right_dir, -rotation.y * mouse_sensitivity);
            }
        }

        // Apply visual recoil offset to rotation (this is a simple approximation)
        // Ideally we'd separate camera rotation from player body for pitch recoil
        let right = player_transform.right();
        if let Ok(right_dir) = Dir3::new(*right) {
            player_transform.rotate_axis(right_dir, weapon_state.recoil_pitch * dt);
            player_transform.rotate_axis(Dir3::Y, weapon_state.recoil_yaw * dt);
        }

        if let Some(movement) = actions.player_movement {
            // Calculate movement direction relative to player's rotation
            let forward = player_transform.forward();
            let right = player_transform.right();

            let mut forward_plane = *forward;
            forward_plane.y = 0.0;
            forward_plane = forward_plane.normalize_or_zero();

            let mut right_plane = *right;
            right_plane.y = 0.0;
            right_plane = right_plane.normalize_or_zero();

            let move_dir =
                (forward_plane * movement.y + right_plane * movement.x).normalize_or_zero();

            player_transform.translation += move_dir * speed * dt;
        }

        // Update fire cooldown
        if weapon_state.fire_cooldown > 0.0 {
            weapon_state.fire_cooldown -= dt;
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
    actions: Res<Actions>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fire_request: ResMut<FireRequest>,
    mut player_query: Query<&mut WeaponState, With<Player>>,
) {
    if let Ok(mut weapon_state) = player_query.single_mut() {
        // Handle ADS toggling
        weapon_state.is_ads = actions.player_aim;

        // Handle Firing
        if mouse_button_input.just_pressed(MouseButton::Left) {
            if weapon_state.fire_cooldown <= 0.0 {
                if let Ok((_camera, camera_transform)) = q_camera.single() {
                    if let Ok(_window) = q_window.single() {
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

                        // Add Recoil
                        weapon_state.target_recoil_pitch += 0.05; // Kick up
                        weapon_state.recoil_pitch = 0.5; // Instant visual kick

                        // Reset cooldown for 96 BPM 16th notes
                        weapon_state.fire_cooldown = 0.15625;
                    }
                }
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
