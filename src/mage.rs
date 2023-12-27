use std::iter::Cycle;
use std::ops::RangeInclusive;

use bevy::sprite::Anchor;
use bevy::{prelude::*, utils::HashMap};
use bevy_xpbd_2d::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;

use crate::prelude::TILE_SIZE;

pub struct MagePlugin;

#[derive(Component)]
struct Mage {
    firing_spell: bool,
    is_walking: bool,
    spell_cooldown: Timer,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Debug, Hash, Copy, Reflect)]
enum MageActions {
    Up,
    Down,
    Left,
    Right,
    SpellPrimary,
    SpellSecondary,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Debug, Hash, Copy, Reflect)]
enum Spell {
    BlastLaunch,
    BlastActivate,
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
struct SpellSlotMap {
    map: HashMap<MageActions, Spell>,
}

#[derive(Component)]
struct RepeatingAnimation {
    next_frame_index: Cycle<RangeInclusive<i32>>,
    frame_timer: Timer,
}

#[derive(Bundle)]
struct MageBundle {
    mage: Mage,
    walk_animation: RepeatingAnimation,
    slot_input_map: InputMap<MageActions>,
    slot_action_state: ActionState<MageActions>,
    spell_action_state: ActionState<Spell>,
    spell_slot_map: SpellSlotMap,
}

fn setup_mage(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    use KeyCode::*;

    let mage_spritesheet = asset_server.load("mage.png");
    let mage_texture_atlas = TextureAtlas::from_grid(
        mage_spritesheet,
        Vec2::new(TILE_SIZE, TILE_SIZE),
        3,
        1,
        None,
        None,
    );

    let mut spell_slot_map = SpellSlotMap::default();
    spell_slot_map.insert(MageActions::SpellPrimary, Spell::BlastLaunch);
    spell_slot_map.insert(MageActions::SpellSecondary, Spell::BlastActivate);

    // Animation is already on the 0 frame, so start iterating at 1.
    let mut walk_animation_frames = (0..=1).cycle();
    walk_animation_frames.next();

    // Move sprite up so the collider is at the bottom.
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.anchor = Anchor::Custom(Vec2::new(0.0, -0.25));

    commands.spawn((
        MageBundle {
            mage: Mage {
                firing_spell: false,
                is_walking: false,
                spell_cooldown: Timer::from_seconds(0.5, TimerMode::Once),
            },
            walk_animation: RepeatingAnimation {
                next_frame_index: walk_animation_frames,
                frame_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
            slot_input_map: InputMap::new([
                (Q, MageActions::SpellPrimary),
                (E, MageActions::SpellSecondary),
                (W, MageActions::Up),
                (A, MageActions::Left),
                (S, MageActions::Down),
                (D, MageActions::Right),
            ])
            .build(),
            slot_action_state: ActionState::default(),
            spell_action_state: ActionState::default(),
            spell_slot_map,
        },
        SpriteSheetBundle {
            sprite,
            texture_atlas: texture_atlases.add(mage_texture_atlas),
            ..default()
        },
        RigidBody::Kinematic,
        Collider::ball(8.0),
        LinearVelocity::default(),
    ));
}

fn copy_action_state(
    mut query: Query<(
        &ActionState<MageActions>,
        &mut ActionState<Spell>,
        &SpellSlotMap,
    )>,
) {
    for (slot_state, mut spell_state, spell_slot_map) in query.iter_mut() {
        for slot in MageActions::variants() {
            if let Some(&matching_ability) = spell_slot_map.get(&slot) {
                // This copies the `ActionData` between the ActionStates,
                // including information about how long the buttons have been pressed or released
                spell_state.set_action_data(matching_ability, slot_state.action_data(slot).clone());
            }
        }
    }
}

fn report_spells_used(query: Query<&ActionState<Spell>>) {
    for ability_state in query.iter() {
        for ability in ability_state.get_just_pressed() {
            dbg!(ability);
        }
    }
}

fn use_spell(mut commands: Commands, mut mage_query: Query<(&ActionState<Spell>, &mut Mage)>) {
    let (action_state, mut mage) = mage_query.single_mut();

    if action_state.just_pressed(Spell::BlastLaunch) {
        mage.firing_spell = true;
    }
}

fn movevement(mut mage_query: Query<(&ActionState<MageActions>, &mut Mage, &mut LinearVelocity)>) {
    let (action_state, mut mage, mut velocity) = mage_query.single_mut();

    velocity.x = 0.0;
    velocity.y = 0.0;

    if mage.firing_spell {
        return;
    }
    mage.is_walking = false;

    if action_state.pressed(MageActions::Right) {
        velocity.x = 10.0;
        mage.is_walking = true;
    }
    if action_state.pressed(MageActions::Left) {
        velocity.x = -10.0;
        mage.is_walking = true;
    }
    if action_state.pressed(MageActions::Up) {
        velocity.y = 10.0;
        mage.is_walking = true;
    }
    if action_state.pressed(MageActions::Down) {
        velocity.y = -10.0;
        mage.is_walking = true;
    }
}

fn animate_mage(
    time: Res<Time>,
    mut mage_query: Query<(&mut Mage, &mut RepeatingAnimation, &mut TextureAtlasSprite)>,
) {
    let (mut mage, mut walking_animation, mut sprite) = mage_query.single_mut();

    if mage.firing_spell {
        if mage.spell_cooldown.elapsed().is_zero() {
            sprite.index = 2;
        }

        mage.spell_cooldown.tick(time.delta());
        if mage.spell_cooldown.just_finished() {
            mage.firing_spell = false;
            mage.spell_cooldown.reset();
            sprite.index = 0;
        }
    } else if mage.is_walking {
        walking_animation.frame_timer.tick(time.delta());

        if walking_animation.frame_timer.just_finished() {
            sprite.index = walking_animation.next_frame_index.next().unwrap() as usize;
        }
    }
}

impl Plugin for MagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_mage)
            .add_plugins(InputManagerPlugin::<MageActions>::default())
            .add_plugins(InputManagerPlugin::<Spell>::default())
            .add_systems(
                PreUpdate,
                copy_action_state.after(InputManagerSystem::ManualControl),
            )
            .add_systems(Update, report_spells_used)
            .add_systems(Update, (use_spell, movevement, animate_mage).chain());
    }
}
