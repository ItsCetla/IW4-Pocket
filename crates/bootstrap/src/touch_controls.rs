//! On-screen touch controls for phones without a controller.
//!
//! Layout follows the usual mobile shooter: a movement stick under the left thumb, look
//! by dragging the right half, and round buttons on the right. The buttons send the
//! same keyboard and mouse input the game already binds (W/A/S/D, Space, C, R, F, G, V,
//! left and right mouse, Escape), so no controller is faked and no pad prompts appear.
//!
//! Setting: `IW4L_TOUCH_CONTROLS` in iw4l-env.txt, shown in Options -> IW4 Pocket options
//! as "On-screen controls". Unset (Auto) shows them in a match when no controller is
//! connected; 1 shows them always; 0 never.
use std::collections::HashMap;

use bevy::input::ButtonState;
use bevy::input::InputSystems;
use bevy::input::gamepad::Gamepad;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ui::UiLayer;

/// True while the touch controls are on screen. The iOS touch-to-mouse bridge stands
/// down while this is set, so a thumb on a button never also clicks.
#[derive(Resource, Default, Clone, Copy)]
pub struct TouchControlsActive(pub bool);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Auto,
    Always,
    Never,
}

#[derive(Component)]
struct TouchRoot;

/// What a button presses.
#[derive(Clone, Copy)]
enum Out {
    Key(KeyCode),
    Mouse(MouseButton),
}

struct Btn {
    label: &'static str,
    out: Out,
    /// Distance from the right and bottom edges, and diameter, in logical pixels.
    right: f32,
    bottom: f32,
    size: f32,
}

const BUTTONS: [Btn; 11] = [
    Btn {
        label: "FIRE",
        out: Out::Mouse(MouseButton::Left),
        right: 30.0,
        bottom: 40.0,
        size: 88.0,
    },
    Btn {
        label: "AIM",
        out: Out::Mouse(MouseButton::Right),
        right: 140.0,
        bottom: 70.0,
        size: 62.0,
    },
    Btn {
        label: "JUMP",
        out: Out::Key(KeyCode::Space),
        right: 36.0,
        bottom: 150.0,
        size: 58.0,
    },
    Btn {
        label: "RELOAD",
        out: Out::Key(KeyCode::KeyR),
        right: 120.0,
        bottom: 150.0,
        size: 52.0,
    },
    Btn {
        label: "CROUCH",
        out: Out::Key(KeyCode::KeyC),
        right: 214.0,
        bottom: 56.0,
        size: 52.0,
    },
    Btn {
        label: "FRAG",
        out: Out::Key(KeyCode::KeyF),
        right: 30.0,
        bottom: 300.0,
        size: 48.0,
    },
    Btn {
        label: "TACT",
        out: Out::Key(KeyCode::KeyG),
        right: 90.0,
        bottom: 300.0,
        size: 48.0,
    },
    Btn {
        label: "KNIFE",
        out: Out::Key(KeyCode::KeyV),
        right: 150.0,
        bottom: 300.0,
        size: 48.0,
    },
    Btn {
        label: "SWAP",
        out: Out::Mouse(MouseButton::Middle),
        right: 210.0,
        bottom: 300.0,
        size: 48.0,
    },
    Btn {
        label: "BOARD",
        out: Out::Key(KeyCode::Tab),
        right: 270.0,
        bottom: 300.0,
        size: 48.0,
    },
    Btn {
        label: "MENU",
        out: Out::Key(KeyCode::Escape),
        right: 12.0,
        bottom: 368.0,
        size: 44.0,
    },
];

/// Stick travel that counts as full deflection, and how far it must move to press a key.
const STICK_RADIUS: f32 = 70.0;
const STICK_PRESS: f32 = 0.35;
const SPRINT_AT: f32 = 0.95;
/// Mouse counts sent per pixel of drag. Raise it to turn faster.
const LOOK_GAIN: f32 = 1.6;
/// Touches starting left of this share of the screen width steer the stick.
const STICK_ZONE: f32 = 0.42;

#[derive(Clone, Copy)]
enum Role {
    Stick { start: Vec2 },
    Look { last: Vec2 },
    Button(usize),
}

#[derive(Default)]
struct Held {
    roles: HashMap<u64, Role>,
    stick_keys: Vec<KeyCode>,
}

pub struct TouchControlsPlugin;

impl Plugin for TouchControlsPlugin {
    fn build(&self, app: &mut App) {
        let mode = match std::env::var("IW4L_TOUCH_CONTROLS")
            .ok()
            .as_deref()
            .map(str::trim)
        {
            Some("1") => Mode::Always,
            Some("0") => Mode::Never,
            _ => Mode::Auto,
        };
        app.insert_resource(mode)
            .init_resource::<TouchControlsActive>()
            .add_systems(Startup, spawn_overlay)
            .add_systems(
                PreUpdate,
                (decide_active, drive_touch).chain().before(InputSystems),
            )
            .add_systems(PostUpdate, sync_visibility);
    }
}

fn spawn_overlay(mut commands: Commands, font: Option<Res<ui::GameUiFont>>) {
    let font = font.map(|f| f.0.clone()).unwrap_or_default();
    commands
        .spawn((
            TouchRoot,
            UiLayer::Overlay,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(18_000),
        ))
        .with_children(|root| {
            // Faint ring where the movement stick lives.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(60),
                    bottom: px(50),
                    width: px(140),
                    height: px(140),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
            ));
            for button in &BUTTONS {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(button.right),
                        bottom: px(button.bottom),
                        width: px(button.size),
                        height: px(button.size),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.42)),
                ))
                .with_child((
                    Text::new(button.label),
                    ui::game_text_font(&font, if button.size > 60.0 { 15.0 } else { 11.0 }),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                ));
            }
        });
}

fn decide_active(
    mode: Res<Mode>,
    screen: Option<Res<frame::AppScreen>>,
    menus: Option<Res<hud::ScriptMenus>>,
    pads: Query<(), With<Gamepad>>,
    mut active: ResMut<TouchControlsActive>,
) {
    let in_match = screen
        .is_some_and(|s| matches!(*s, frame::AppScreen::InGame | frame::AppScreen::ClassSelect));
    let menu_open = menus.is_some_and(|m| m.captures_input());
    let want = match *mode {
        Mode::Never => false,
        Mode::Always => in_match && !menu_open,
        Mode::Auto => in_match && !menu_open && pads.is_empty(),
    };
    if active.0 != want {
        active.0 = want;
    }
}

fn sync_visibility(
    active: Res<TouchControlsActive>,
    mut roots: Query<&mut Visibility, With<TouchRoot>>,
) {
    let want = if active.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut roots {
        if *visibility != want {
            *visibility = want;
        }
    }
}

fn key_message(window: Entity, key: KeyCode, state: ButtonState) -> KeyboardInput {
    KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window,
    }
}

struct Sinks<'a, 'w> {
    window: Entity,
    keys: &'a mut MessageWriter<'w, KeyboardInput>,
    mouse: &'a mut MessageWriter<'w, MouseButtonInput>,
}

impl Sinks<'_, '_> {
    fn press(&mut self, out: Out, state: ButtonState) {
        match out {
            Out::Key(key) => {
                self.keys.write(key_message(self.window, key, state));
            }
            Out::Mouse(button) => {
                self.mouse.write(MouseButtonInput {
                    button,
                    state,
                    window: self.window,
                });
            }
        }
    }
}

fn drive_touch(
    active: Res<TouchControlsActive>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut touches: MessageReader<TouchInput>,
    mut keys: MessageWriter<KeyboardInput>,
    mut mouse: MessageWriter<MouseButtonInput>,
    mut motion: MessageWriter<MouseMotion>,
    mut held: Local<Held>,
) {
    let Ok((window_entity, window)) = windows.single() else {
        for _ in touches.read() {}
        return;
    };
    let mut sinks = Sinks {
        window: window_entity,
        keys: &mut keys,
        mouse: &mut mouse,
    };

    if !active.0 {
        for _ in touches.read() {}
        if !held.roles.is_empty() || !held.stick_keys.is_empty() {
            for (_, role) in held.roles.drain() {
                if let Role::Button(index) = role {
                    sinks.press(BUTTONS[index].out, ButtonState::Released);
                }
            }
            for key in held.stick_keys.drain(..) {
                sinks.press(Out::Key(key), ButtonState::Released);
            }
        }
        return;
    }

    let (width, height) = (window.width(), window.height());
    let mut stick = None;
    for touch in touches.read() {
        match touch.phase {
            TouchPhase::Started => {
                let hit = BUTTONS.iter().position(|b| {
                    let center = Vec2::new(
                        width - b.right - b.size * 0.5,
                        height - b.bottom - b.size * 0.5,
                    );
                    touch.position.distance(center) <= b.size * 0.5 + 8.0
                });
                let has = |wanted: fn(&Role) -> bool| held.roles.values().any(wanted);
                if let Some(index) = hit {
                    held.roles.insert(touch.id, Role::Button(index));
                    sinks.press(BUTTONS[index].out, ButtonState::Pressed);
                } else if touch.position.x < width * STICK_ZONE
                    && !has(|r| matches!(r, Role::Stick { .. }))
                {
                    held.roles.insert(
                        touch.id,
                        Role::Stick {
                            start: touch.position,
                        },
                    );
                    stick = Some(Vec2::ZERO);
                } else if !has(|r| matches!(r, Role::Look { .. })) {
                    held.roles.insert(
                        touch.id,
                        Role::Look {
                            last: touch.position,
                        },
                    );
                }
            }
            TouchPhase::Moved => match held.roles.get_mut(&touch.id) {
                Some(Role::Stick { start }) => {
                    stick = Some(((touch.position - *start) / STICK_RADIUS).clamp_length_max(1.0));
                }
                Some(Role::Look { last }) => {
                    let delta = (touch.position - *last) * LOOK_GAIN;
                    *last = touch.position;
                    if delta != Vec2::ZERO {
                        motion.write(MouseMotion { delta });
                    }
                }
                _ => {}
            },
            TouchPhase::Ended | TouchPhase::Canceled => match held.roles.remove(&touch.id) {
                Some(Role::Button(index)) => sinks.press(BUTTONS[index].out, ButtonState::Released),
                Some(Role::Stick { .. }) => stick = Some(Vec2::ZERO),
                _ => {}
            },
        }
    }

    if let Some(vector) = stick {
        let mut want: Vec<KeyCode> = Vec::new();
        if vector.y < -STICK_PRESS {
            want.push(KeyCode::KeyW);
        }
        if vector.y > STICK_PRESS {
            want.push(KeyCode::KeyS);
        }
        if vector.x < -STICK_PRESS {
            want.push(KeyCode::KeyA);
        }
        if vector.x > STICK_PRESS {
            want.push(KeyCode::KeyD);
        }
        if vector.length() >= SPRINT_AT && want.contains(&KeyCode::KeyW) {
            want.push(KeyCode::ShiftLeft);
        }
        for key in held.stick_keys.iter().filter(|k| !want.contains(k)) {
            sinks.press(Out::Key(*key), ButtonState::Released);
        }
        for key in want.iter().filter(|k| !held.stick_keys.contains(k)) {
            sinks.press(Out::Key(*key), ButtonState::Pressed);
        }
        held.stick_keys = want;
    }
}
