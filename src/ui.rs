use crate::GameState;
use crate::audio::FireRequest;
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BeatTimer(Timer::from_seconds(
            0.15625,
            TimerMode::Repeating,
        )))
        .init_resource::<Combo>()
        .init_resource::<AutoMode>()
        .init_resource::<RhythmRunning>()
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
                toggle_auto_mode,
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

#[derive(Resource, Default)]
struct AutoMode(bool);

#[derive(Resource, Default)]
struct RhythmRunning(bool);

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
struct AutoModeText;

#[derive(Component)]
struct StartText;

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

    // Auto Mode Indicator
    let auto_mode_text = commands
        .spawn((
            Text::new("AUTO MODE: OFF"),
            TextFont {
                font_size: 25.0,
                ..default()
            },
            TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)), // Gray, semi-transparent
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(10.0),
                left: Val::Percent(50.0), // Center horizontally
                margin: UiRect {
                    left: Val::Px(-100.0), // Approximate half width centering
                    ..default()
                },
                ..default()
            },
            AutoModeText,
        ))
        .id();

    // Start Text
    let start_text = commands
        .spawn((
            Text::new("CLICK TO START"),
            TextFont {
                font_size: 50.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0), // Center
                margin: UiRect {
                    left: Val::Px(-150.0),
                    top: Val::Px(-25.0),
                    ..default()
                },
                ..default()
            },
            StartText,
        ))
        .id();

    commands.entity(root).add_child(target);
    commands.entity(root).add_child(judgement);
    commands.entity(root).add_child(combo);
    commands.entity(root).add_child(auto_mode_text);
    commands.entity(root).add_child(start_text);
}

fn spawn_notes(
    mut commands: Commands,
    mut timer: ResMut<BeatTimer>,
    time: Res<Time>,
    root_query: Query<Entity, With<RhythmUiRoot>>,
    rhythm_running: Res<RhythmRunning>,
) {
    if !rhythm_running.0 {
        return;
    }
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

fn move_notes(
    mut query: Query<(Entity, &mut Node), With<Note>>,
    time: Res<Time>,
    rhythm_running: Res<RhythmRunning>,
) {
    if !rhythm_running.0 {
        return;
    }
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
    auto_mode: Res<AutoMode>,
    mut fire_request: ResMut<FireRequest>,
    mut rhythm_running: ResMut<RhythmRunning>,
    mut start_text_query: Query<&mut Visibility, With<StartText>>,
    mut beat_timer: ResMut<BeatTimer>,
) {
    // Start Game Logic
    if !rhythm_running.0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            rhythm_running.0 = true;
            beat_timer.0.reset(); // Snyc start
            if let Some(mut vis) = start_text_query.iter_mut().next() {
                *vis = Visibility::Hidden;
            }
            fire_request.0 = true; // Play start sound
        }
        return;
    }

    let target_pos = 80.0; // Target is at 80% Top
    let hit_window = 5.0; // +/- 5%

    // Logic for Auto Mode or Manual Input
    let should_check = auto_mode.0 || mouse_button_input.just_pressed(MouseButton::Left);

    if should_check {
        // Find the closest note to the target
        let mut closest_note: Option<(Entity, f32, Mut<BackgroundColor>, Mut<HitStatus>)> = None;
        let mut min_diff = f32::MAX;

        for (entity, node, bg_color, status) in &mut query {
            if *status != HitStatus::None {
                continue;
            }

            if let Val::Percent(top) = node.top {
                let diff = top - target_pos as f32;
                // For AutoMode, we only care if it's EXACTLY inside the perfect window to trigger
                // For Manual, we find the closest one in range

                // If AutoMode, wait until it's very close (e.g., within 0.5)
                if auto_mode.0 {
                    if diff.abs() <= 1.0 {
                        // Found a note to hit perfectly
                        // We can break early or just pick this one
                        closest_note = Some((entity, diff, bg_color, status));
                        break;
                    }
                } else {
                    // Manual mode: find closest
                    let abs_diff = diff.abs();
                    if abs_diff < min_diff {
                        min_diff = abs_diff;
                        closest_note = Some((entity, diff, bg_color, status));
                    }
                }
            }
        }

        // If we are in AutoMode but didn't find a note in perfect range, return
        if auto_mode.0 && closest_note.is_none() {
            return;
        }

        let mut should_reset = false;

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
                        fire_request.0 = true; // Play sound

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
                        // Only process miss on manual input, auto mode shouldn't miss if logic is correct
                        // But if user clicks late, it's a miss.
                        if !auto_mode.0 {
                            // Too Late (Top > Target)
                            *bg_color = BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)); // Gray
                            *status = HitStatus::Miss;
                            combo.0 = 0;
                            judgement_text.0 = "LATE (MISS)".to_string();
                            judgement_color.0 = Color::srgb(1.0, 0.0, 0.0); // Red
                            info!("Too Late! Diff: {:.2}", diff_signed);

                            should_reset = true;
                        }
                    } else {
                        if !auto_mode.0 {
                            // Too Early (Top < Target)
                            *bg_color = BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 1.0)); // Gray
                            *status = HitStatus::Miss;
                            combo.0 = 0;
                            judgement_text.0 = "EARLY (MISS)".to_string();
                            judgement_color.0 = Color::srgb(1.0, 0.0, 0.0); // Red
                            info!("Too Early! Diff: {:.2}", diff_signed);

                            should_reset = true;
                        }
                    }
                } else {
                    // Out of reasonable range (Spamming far from any note)
                    if !auto_mode.0 {
                        combo.0 = 0;
                        judgement_text.0 = "MISS".to_string();
                        judgement_color.0 = Color::srgb(1.0, 0.0, 0.0); // Red
                        timer.0.reset();

                        should_reset = true;
                    }
                }
            } else {
                // No notes found (Spamming empty space)
                if !auto_mode.0 {
                    combo.0 = 0;
                    judgement_text.0 = "MISS".to_string();
                    judgement_color.0 = Color::srgb(1.0, 0.0, 0.0); // Red
                    timer.0.reset();

                    should_reset = true;
                }
            }
        }

        if should_reset {
            // Trigger Miss Routine - Stop movement only, don't despawn notes
            rhythm_running.0 = false;
            if let Some(mut vis) = start_text_query.iter_mut().next() {
                *vis = Visibility::Visible;
            }
        }
    }
}

fn toggle_auto_mode(
    input: Res<ButtonInput<KeyCode>>,
    mut auto_mode: ResMut<AutoMode>,
    mut query: Query<(&mut Text, &mut TextColor), With<AutoModeText>>,
) {
    if input.just_pressed(KeyCode::F1) {
        auto_mode.0 = !auto_mode.0;
        info!("Auto Mode: {}", auto_mode.0);

        if let Some((mut text, mut color)) = query.iter_mut().next() {
            if auto_mode.0 {
                text.0 = "AUTO MODE: ON".to_string();
                color.0 = Color::srgb(0.0, 1.0, 1.0); // Cyan
            } else {
                text.0 = "AUTO MODE: OFF".to_string();
                color.0 = Color::srgba(1., 0.5, 0.5, 0.5); // Gray
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
    mut rhythm_running: ResMut<RhythmRunning>,
    mut start_text_query: Query<&mut Visibility, With<StartText>>,
) {
    if !rhythm_running.0 {
        return;
    }

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

                    // Stop movement, show restart text, but DON'T despawn notes
                    rhythm_running.0 = false;
                    if let Some(mut vis) = start_text_query.iter_mut().next() {
                        *vis = Visibility::Visible;
                    }
                    return; // Stop processing, keep all notes on screen
                }
                // Only despawn notes that were hit
                if *status == HitStatus::Hit {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn cleanup_rhythm_ui(mut commands: Commands, query: Query<Entity, With<RhythmUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_children().despawn();
    }
}
