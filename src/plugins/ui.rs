use bevy::prelude::*;
// use bevy_lunex::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputSubmitEvent};

use crate::components::{
    color::{Dictionary, Modifier},
    core::LockedAxesBundle,
};

const BORDER_COLOR_ACTIVE: Color = Color::rgb(0.75, 0.52, 0.99);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Component)]
pub struct TypingUi;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum TypingState {
    #[default]
    IsMoving,
    IsTyping,
}

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TypingState>()
            .add_systems(OnEnter(TypingState::IsTyping), setup_ui_on_typing)
            .add_systems(OnExit(TypingState::IsTyping), kill_ui_on_typing)
            .add_systems(Update, (typing_toggler, listener));
    }
}

fn kill_ui_on_typing(mut commands: Commands, ui_query: Query<Entity, With<TypingUi>>) {
    if let Ok(entity) = ui_query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_ui_on_typing(mut commands: Commands) {
    commands
        .spawn((
            TypingUi,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    border_color: BORDER_COLOR_ACTIVE.into(),
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default().with_text_style(TextStyle {
                    font_size: 40.,
                    color: TEXT_COLOR,
                    ..default()
                }),
            ));
        });
}

fn listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dictionary: ResMut<Dictionary>,
) {
    for event in events.read() {
        let TextInputSubmitEvent { value, .. } = event;
        let mut parts = value.split_whitespace().rev();

        let noun = parts.next().unwrap_or("ball");

        let shape: Mesh = match noun {
            "cube" => Mesh::from(Cuboid::new(1.0, 1.0, 1.0)),
            "ball" => Mesh::from(Sphere::new(0.5)),
            _ => Mesh::from(Sphere::new(0.5)),
        };

        let collider = match noun {
            "cube" => Collider::cuboid(0.5, 0.5, 0.5),
            "ball" => Collider::ball(0.5),
            _ => Collider::ball(0.5),
        };

        let mut material = StandardMaterial::default();
        let mut transform = Transform::from_xyz(0.0, 20.0, 0.0);

        for adj in parts {
            if let Some(entry) = dictionary.search(adj) {
                match entry.modifier {
                    Modifier::ColorModifier(color) => material.base_color = color,
                    Modifier::ScaleModifier(scale) => {
                        transform.scale = Vec3::splat(scale);
                        transform.translation.y += scale * 0.5;
                    }
                }
            }
        }
        // this should impl Into<Mesh>

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(shape),
                material: materials.add(material),
                transform,
                ..default()
            },
            collider,
            RigidBody::Dynamic,
            LockedAxesBundle::default(),
        ));
    }
}

fn typing_toggler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    typing_state: Res<State<TypingState>>,
    mut next_state: ResMut<NextState<TypingState>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Slash, KeyCode::Escape, KeyCode::Enter]) {
        match typing_state.get() {
            TypingState::IsMoving => next_state.set(TypingState::IsTyping),
            TypingState::IsTyping => next_state.set(TypingState::IsMoving),
        }
    }
}
