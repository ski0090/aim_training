use crate::GameState;
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BeatTimer(Timer::from_seconds(
            0.15625,
            TimerMode::Repeating,
        )))
        .init_resource::<Combo>()
        .add_systems(OnEnter(GameState::Playing), setup_rhythm_ui)
        .add_systems(
            Update,
            (
                spawn_notes,
                move_notes,
                despawn_notes,
                check_hit,
                update_combo_ui,
                fade_judgement,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_rhythm_ui);
    }
}

#[derive(Resource)]
struct BeatTimer(Timer);

#[derive(Resource, Default)]
struct Combo(u32);

#[derive(Component)]
struct RhythmUiRoot;

#[derive(Component)]
struct Note;

#[derive(Component)]
struct TargetLine;

#[derive(Component)]
struct JudgementText;

#[derive(Component)]
struct ComboText;

#[derive(Component)]
struct JudgementTimer(Timer);

#[derive(Component, PartialEq)]
enum HitStatus {
    None,
    Hit,
    Miss,
}

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

    // Target Line (Hit Marker) - Place near bottom
    let target = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(4.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Percent(80.0), // Target is near bottom
                margin: UiRect {
                    left: Val::Px(-100.0),
                    ..default()
                }, // Center horizontally (half of width)
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.8)),
            TargetLine,
        ))
        .id();

    // Judgement Text
    let judgement = commands
        .spawn((
            Text::new(""),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Percent(60.0), // Above target
                margin: UiRect {
                    left: Val::Px(-50.0),
                    ..default()
                }, // Approximate centering
                ..default()
            },
            JudgementText,
            JudgementTimer(Timer::from_seconds(1.0, TimerMode::Once)),
        ))
        .id();

    // Combo Text
    let combo = commands
        .spawn((
            Text::new("Combo: 0"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Percent(40.0), // Above judgement
                margin: UiRect {
                    left: Val::Px(-50.0),
                    ..default()
                },
                ..default()
            },
            ComboText,
        ))
        .id();

    commands.entity(root).add_child(target);
    commands.entity(root).add_child(judgement);
    commands.entity(root).add_child(combo);
}

fn spawn_notes(
    mut commands: Commands,
    mut timer: ResMut<BeatTimer>,
    time: Res<Time>,
    root_query: Query<Entity, With<RhythmUiRoot>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if let Some(root) = root_query.iter().next() {
            let note = commands
                .spawn((
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(10.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(20.0),
                        top: Val::Percent(0.0), // Spawn at top
                        margin: UiRect {
                            left: Val::Px(-15.0),
                            ..default()
                        }, // Center
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 0.8)),
                    Note,
                    HitStatus::None,
                ))
                .id();
            commands.entity(root).add_child(note);
        }
    }
}

fn move_notes(mut query: Query<(Entity, &mut Node), With<Note>>, time: Res<Time>) {
    let speed = 40.0; // Speed in Percent per second

    for (_entity, mut node) in &mut query {
        if let Val::Percent(current_top) = node.top {
            let new_top = current_top + speed * time.delta_secs();
            node.top = Val::Percent(new_top);
        }
    }
}

fn check_hit(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<(Entity, &Node, &mut BackgroundColor, &mut HitStatus), With<Note>>,
    mut judgement_query: Query<
        (&mut Text, &mut TextColor, &mut JudgementTimer),
        With<JudgementText>,
    >,
    mut combo: ResMut<Combo>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let target_pos = 80.0; // Target is at 80% Top
        let hit_window = 5.0; // +/- 5%

        // Find the closest note to the target
        let mut closest_note: Option<(Entity, f32, Mut<BackgroundColor>, Mut<HitStatus>)> = None;
        let mut min_diff = f32::MAX;

        for (entity, node, bg_color, status) in &mut query {
            if *status != HitStatus::None {
                continue;
            }

            if let Val::Percent(top) = node.top {
                let diff = (top - target_pos).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest_note = Some((entity, top - target_pos, bg_color, status));
                }
            }
        }

        if let Some((mut judgement_text, mut judgement_color, mut timer)) =
            judgement_query.iter_mut().next()
        {
            if let Some((_entity, diff_signed, mut bg_color, mut status)) = closest_note {
                // Check if within reasonable range to be considered an attempt (e.g., +/- 15%)
                if diff_signed.abs() <= 15.0 {
                    timer.0.reset();
                    if diff_signed.abs() <= hit_window {
                        // Hit!
                        *bg_color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 1.0)); // Red
                        *status = HitStatus::Hit;
                        combo.0 += 1;

                        // Precise judgement
                        if diff_signed.abs() <= 1.0 {
                            judgement_text.0 = "PERFECT!!".to_string();
                            judgement_color.0 = Color::srgb(1.0, 0.84, 0.0); // Gold
                        } else {
                            judgement_text.0 = "GOOD!".to_string();
                            judgement_color.0 = Color::srgb(0.0, 1.0, 0.0); // Green
                        }

                        info!("Hit! Diff: {:.2}", diff_signed);
                    } else if diff_signed > hit_window {
                        // Too Late (Top > Target)
                        *bg_color = BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)); // Gray
                        *status = HitStatus::Miss;
                        combo.0 = 0;
                        judgement_text.0 = "LATE".to_string();
                        judgement_color.0 = Color::srgb(1.0, 0.65, 0.0); // Orange
                        info!("Too Late! Diff: {:.2}", diff_signed);
                    } else {
                        // Too Early (Top < Target)
                        *bg_color = BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)); // Gray
                        *status = HitStatus::Miss;
                        combo.0 = 0;
                        judgement_text.0 = "EARLY".to_string();
                        judgement_color.0 = Color::srgb(1.0, 0.65, 0.0); // Orange
                        info!("Too Early! Diff: {:.2}", diff_signed);
                    }
                }
            }
        }
    }
}

fn update_combo_ui(combo: Res<Combo>, mut query: Query<&mut Text, With<ComboText>>) {
    if combo.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Combo: {}", combo.0);
        }
    }
}

fn fade_judgement(
    time: Res<Time>,
    mut query: Query<(&mut TextColor, &mut JudgementTimer), With<JudgementText>>,
) {
    for (mut color, mut timer) in &mut query {
        timer.0.tick(time.delta());
        let alpha = 1.0 - timer.0.fraction(); // Fade out
        color.0.set_alpha(alpha);
    }
}

fn despawn_notes(
    mut commands: Commands,
    query: Query<(Entity, &Node, &HitStatus), With<Note>>,
    mut combo: ResMut<Combo>,
    mut judgement_query: Query<
        (&mut Text, &mut TextColor, &mut JudgementTimer),
        With<JudgementText>,
    >,
) {
    for (entity, node, status) in &query {
        if let Val::Percent(top) = node.top {
            if top > 110.0 {
                // If the note goes off-screen without being hit, it's a miss
                if *status == HitStatus::None {
                    combo.0 = 0;
                    if let Some((mut text, mut color, mut timer)) =
                        judgement_query.iter_mut().next()
                    {
                        text.0 = "MISS".to_string();
                        color.0 = Color::srgb(1.0, 0.0, 0.0); // Red
                        timer.0.reset();
                    }
                }
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
