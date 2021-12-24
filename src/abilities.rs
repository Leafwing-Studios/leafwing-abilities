/// All combat actions other than movement are abilities.
/// Abilities have several important design constraints:
///  - only one ability may be used at once
///  - abilities are stronger when used in time with the beat
///  - input actions do not map one-to-one with abilities: chords may be required
///  - the player (but not enemies) should use the first action input if there are conflicts
///  - abilities are gated: on cooldowns, resources or other factors
///  - each ability must store its own data in complex and reusable ways
///  - multiple units can have the same ability, and be active on the battlefield at once
///
/// This leads us to the following core architecture for actions:
/// - each possible action is stored as its own entity, with the `Ability` marker component
/// - each action of the same type has a unique marker component
/// - data such as cooldown, resource cost and so on are stored on the ability entity in the form of components
/// - these are updated and managed in broad systems which perform standard logic like ticking down cooldowns
/// - each unit tracks which abilities it can and is using in its `Abilities` component
/// - abilties may only be used if their `Usable` component == true
/// - systems that cause abilities to take effects are always enabled, but rely on the presence of the `JustStarted` component to know when to take effect
use bevy::prelude::*;

use bevy::utils::HashMap;
use core::hash::Hash;

use crate::input::{ActionState, InputLabel};
use ability_mapping::{AbilityInputMap, NullAbilityMap};
use usability::Usable;

pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            systems::set_all_abilities_usable
                .label(AbilityLabel::Maintain)
                .before(AbilityLabel::Check),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            cooldowns::tick_cooldowns
                .label(AbilityLabel::Maintain)
                .before(AbilityLabel::Check),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            cooldowns::check_cooldowns
                .label(AbilityLabel::Check)
                .after(AbilityLabel::Maintain),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            disabled::check_for_disabled_abilities
                .label(AbilityLabel::Check)
                .after(AbilityLabel::Maintain),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            usability::update_ability_usability
                .after(AbilityLabel::Check)
                .before(AbilityLabel::Decide),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            ability_mapping::choose_ability_from_input
                .label(AbilityLabel::Decide)
                .after(InputLabel::Processing)
                .after(AbilityLabel::Check),
        )
        .add_system_to_stage(CoreStage::Last, systems::active_ability_cleanup);
    }
}

#[derive(SystemLabel, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum AbilityLabel {
    /// Runs in CoreStage::PreUpdate
    Maintain,
    /// Runs in CoreStage::PreUpdate
    Check,
    /// Runs in CoreStage::PreUpdate
    Decide,
}

/// Marker component for Ability entities
#[derive(Component, Clone, Copy)]
pub struct Ability;

/// Component that stores the abilities that can be used by the unit
#[derive(Component)]
pub struct Abilities {
    ability_list: Vec<Entity>,
    usable: HashMap<Entity, bool>,
    pub active_ability: ActiveAbility,
    input_map: Box<dyn AbilityInputMap>,
}

impl Abilities {
    pub fn from_ability_list(ability_list: Vec<Entity>) -> Self {
        let mut usable = HashMap::default();
        for &entity in ability_list.iter() {
            usable.insert(entity, false);
        }

        Self {
            ability_list,
            usable,
            active_ability: ActiveAbility::NONE,
            input_map: Box::new(NullAbilityMap),
        }
    }

    pub fn from_ability_map(map: impl AbilityInputMap) -> Self {
        let ability_list = map.ability_list();

        let mut usable = HashMap::default();
        for &entity in ability_list.iter() {
            usable.insert(entity, false);
        }

        Self {
            ability_list,
            usable,
            active_ability: ActiveAbility::NONE,
            input_map: Box::new(map),
        }
    }

    pub fn active_ability(&self) -> ActiveAbility {
        self.active_ability
    }

    pub fn ability_list(&self) -> Vec<Entity> {
        self.ability_list.clone()
    }

    pub(crate) fn process_input(&self, action_state: &ActionState) -> Option<Entity> {
        self.input_map
            .process_input(action_state, self.usable.clone())
    }

    pub(crate) fn set_usable(&mut self, ability_entity: Entity, usable: Usable) {
        self.usable.insert(ability_entity, usable.0);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ActiveAbility {
    pub entity: Option<Entity>,
    pub state: AbilityState,
}

impl ActiveAbility {
    const NONE: Self = Self {
        entity: None,
        state: AbilityState::Idle,
    };
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            ability_list: Vec::default(),
            usable: HashMap::default(),
            active_ability: ActiveAbility::NONE,
            input_map: Box::new(NullAbilityMap),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityState {
    JustStarted,
    Active,
    Idle,
}

pub mod usability {
    use bevy::prelude::*;

    use super::{Abilities, Ability};

    #[derive(Component, Clone, Copy)]
    pub struct Usable(pub(crate) bool);

    pub(crate) fn update_ability_usability(
        mut unit_query: Query<&mut Abilities>,
        ability_query: Query<&Usable, With<Ability>>,
    ) {
        for mut unit_abilties in unit_query.iter_mut() {
            for ability_entity in unit_abilties.ability_list() {
                let usable = *ability_query.get(ability_entity).unwrap();
                unit_abilties.set_usable(ability_entity, usable);
            }
        }
    }
}

pub mod systems {
    use super::*;

    /// Abilities start life each frame as `Usable`, and then are disabled by various systems
    pub fn set_all_abilities_usable(mut query: Query<&mut Usable>) {
        for mut usable in query.iter_mut() {
            *usable = Usable(true);
        }
    }

    /// Abilities are no longer `JustStarted` after one frame
    pub fn active_ability_cleanup(mut query: Query<&mut Abilities>) {
        for mut abilities in query.iter_mut() {
            if abilities.active_ability.state == AbilityState::JustStarted {
                abilities.active_ability.state = AbilityState::Active;
            }
        }
    }
}

pub mod disabled {
    use super::*;

    /// Marker component for abilities which cannot be used for miscallaneous reasons
    #[derive(Component)]
    pub struct Disabled;

    pub fn check_for_disabled_abilities(mut query: Query<&mut Usable, With<Disabled>>) {
        for mut usable in query.iter_mut() {
            *usable = Usable(false);
        }
    }
}

pub mod ability_mapping {
    use super::*;
    use crate::input::{ActionState, InputAction};
    use bevy::utils::HashMap;

    /// Used for deciding which ability the character should use, given the inputs received
    pub trait AbilityInputMap: Send + Sync + 'static {
        /// Spawns an ability entity,
        /// and returns its entity if and only if an ability was selected
        fn process_input(
            &self,
            _action_state: &ActionState,
            usable: HashMap<Entity, bool>,
        ) -> Option<Entity>;

        fn ability_list(&self) -> Vec<Entity>;
    }

    /// Abilities do not respond to inputs
    ///
    /// Used for NPCs
    #[derive(Default)]
    pub struct NullAbilityMap;

    impl AbilityInputMap for NullAbilityMap {
        fn process_input(
            &self,
            _action_state: &ActionState,
            _usable: HashMap<Entity, bool>,
        ) -> Option<Entity> {
            None
        }

        fn ability_list(&self) -> Vec<Entity> {
            Vec::default()
        }
    }

    /// Only one ability can be used at once,
    /// and each ability corresponds to one input
    #[derive(Default)]
    pub struct SimpleAbilityMap {
        map: HashMap<InputAction, Entity>,
    }

    impl AbilityInputMap for SimpleAbilityMap {
        fn process_input(
            &self,
            action_state: &ActionState,
            usable: HashMap<Entity, bool>,
        ) -> Option<Entity> {
            for action in InputAction::ABILITIES {
                if action_state.just_pressed(action) {
                    let ability_entity = *self.map.get(&action).unwrap();

                    // Only attempt to use abilities if they can currently be used
                    // If they can't, try another matching ability
                    if *usable.get(&ability_entity).unwrap() {
                        return Some(ability_entity);
                    }
                }
            }
            None
        }

        fn ability_list(&self) -> Vec<Entity> {
            self.map.values().cloned().collect()
        }
    }

    impl SimpleAbilityMap {
        pub fn new(map: HashMap<InputAction, Entity>) -> Self {
            Self { map }
        }
    }

    #[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
    struct InputControlled;

    pub fn choose_ability_from_input(
        action_state: Res<ActionState>,
        mut player_query: Query<&mut Abilities, With<InputControlled>>,
    ) {
        let mut abilities = player_query.single_mut();

        // Only pick a new ability if none are active
        if abilities.active_ability == ActiveAbility::NONE {
            abilities.active_ability = ActiveAbility {
                entity: abilities.process_input(&*action_state),
                state: AbilityState::JustStarted,
            };
        }
    }
}

pub mod cooldowns {
    use bevy::prelude::*;
    use core::time::Duration;

    use super::usability::Usable;
    use super::Ability;

    #[derive(Component, Clone)]
    pub struct Cooldown {
        timer: Timer,
        charges: u8,
        max_charges: u8,
    }

    impl Cooldown {
        pub fn new(seconds: f32) -> Self {
            let mut timer = Timer::from_seconds(seconds, false);
            // All abilities should be available for use on new entities
            timer.tick(Duration::from_secs_f32(seconds));

            Self {
                timer,
                charges: 1,
                max_charges: 1,
            }
        }

        pub fn new_with_charges(seconds: f32, max_charges: u8) {}

        pub fn tick(&mut self, delta: Duration) {
            self.timer.tick(delta);
        }

        pub fn start(&mut self) {
            self.timer.reset()
        }

        pub fn remaining(&self) -> f32 {
            self.timer.percent_left()
        }

        pub fn finished(&self) -> bool {
            self.timer.finished()
        }
    }

    pub(crate) fn tick_cooldowns(mut query: Query<&mut Cooldown>, time: Res<Time>) {
        for mut cooldown in query.iter_mut() {
            // Extra check here avoids change-detection false positives
            if !cooldown.finished() {
                cooldown.tick(time.delta());
            }
        }
    }

    pub(crate) fn check_cooldowns(
        mut query: Query<(&Cooldown, &mut Usable), (With<Ability>, Changed<Cooldown>)>,
    ) {
        for (cooldown, mut usable) in query.iter_mut() {
            if !cooldown.finished() {
                *usable = Usable(false);
            }
        }
    }
}
