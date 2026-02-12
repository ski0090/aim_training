use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::GameState;
use crate::actions::game_control::{GameControl, get_movement};

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions.
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>().add_systems(
            Update,
            set_movement_actions.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Resource)]
pub struct Actions {
    pub player_movement: Option<Vec2>,
    pub player_rotation: Option<Vec2>,
    pub player_aim: bool,
}

pub fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    params: Option<Res<AccumulatedMouseMotion>>,
) {
    let player_movement = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    if player_movement != Vec2::ZERO {
        actions.player_movement = Some(player_movement.normalize());
    } else {
        actions.player_movement = None;
    }

    if let Some(mouse_motion) = params {
        if mouse_motion.delta != Vec2::ZERO {
            actions.player_rotation = Some(mouse_motion.delta);
        } else {
            actions.player_rotation = None;
        }
    } else {
        actions.player_rotation = None;
    }

    actions.player_aim = mouse_button_input.pressed(MouseButton::Right);
}
