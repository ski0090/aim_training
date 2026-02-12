use crate::GameState;
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BeatTimer(Timer::from_seconds(0.1565, TimerMode::Repeating)))
            .add_systems(OnEnter(GameState::Playing), setup_rhythm_ui)
            .add_systems(
                Update,
                (spawn_notes, move_notes, despawn_notes).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_rhythm_ui);
    }
}

#[derive(Resource)]
struct BeatTimer(Timer);

#[derive(Component)]
struct RhythmUiRoot;

#[derive(Component)]
struct Note;

#[derive(Component)]
struct TargetLine;

fn setup_rhythm_ui(mut commands: Commands) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            RhythmUiRoot,
        ))
        .id();

    // Target Line (Hit Marker) - Placed at 20% from left
    let target = commands
        .spawn((
            Node {
                width: Val::Px(4.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                bottom: Val::Px(100.0), // Place it near bottom
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.8)),
            TargetLine,
        ))
        .id();

    commands.entity(root).add_child(target);
}

fn spawn_notes(
    mut commands: Commands,
    mut timer: ResMut<BeatTimer>,
    time: Res<Time>,
    root_query: Query<Entity, With<RhythmUiRoot>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if let Ok(root) = root_query.single() {
            let note = commands
                .spawn((
                    Node {
                        width: Val::Px(10.0),
                        height: Val::Px(30.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(100.0), // Spawn at right edge
                        bottom: Val::Px(110.0),    // Slightly higher to differ from line
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 0.8)),
                    Note,
                ))
                .id();
            commands.entity(root).add_child(note);
        }
    }
}

fn move_notes(mut query: Query<(Entity, &mut Node), With<Note>>, time: Res<Time>) {
    let speed = 40.0; // Speed in Percent per second (travels 80% distance in ~2s)

    for (_entity, mut node) in &mut query {
        if let Val::Percent(current_left) = node.left {
            let new_left = current_left - speed * time.delta_secs();
            node.left = Val::Percent(new_left);
        }
    }
}

fn despawn_notes(mut commands: Commands, query: Query<(Entity, &Node), With<Note>>) {
    for (entity, node) in &query {
        if let Val::Percent(left) = node.left {
            if left < -10.0 {
                commands.entity(entity).despawn_children().despawn();
            }
        }
    }
}

fn cleanup_rhythm_ui(mut commands: Commands, query: Query<Entity, With<RhythmUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_children().despawn();
    }
}
