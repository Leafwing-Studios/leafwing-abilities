use bevy::prelude::*;
use core::convert::From;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    marker::PhantomData,
};

use crate::abilities::{usability::Usable, Abilities, Ability, AbilityLabel, AbilityState};

pub trait ResourcePoolExt {
    fn add_resource_pool<R: ResourceType + From<f32> + Into<f32>>(&mut self) -> &mut Self;
}

impl ResourcePoolExt for App {
    fn add_resource_pool<R: ResourceType + From<f32> + Into<f32>>(&mut self) -> &mut Self {
        self.add_system_to_stage(
            CoreStage::PreUpdate,
            regen_resource::<R>
                .label(AbilityLabel::Maintain)
                .before(AbilityLabel::Check),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            check_resource::<R>
                .label(AbilityLabel::Check)
                .before(AbilityLabel::Decide),
        )
        .add_system(spend_resource::<R>)
    }
}

/// Marker trait for resource types (like Life, Mana, Energy, Rage...)
pub trait ResourceType:
    Component
    + Clone
    + Copy
    + PartialOrd
    + PartialEq
    + Ord
    + Add<Output = Self>
    + Sub<Output = Self>
    + From<f32>
{
    const ZERO: Self;
    // The value which cannot be exceeded
    const LOGICAL_MAX: Self;
}

#[derive(Component, PartialEq)]
pub struct ResourcePool<R: ResourceType> {
    current: R,
    pub regen_rate: R,
    max: R,
    _phantom: PhantomData<R>,
}

impl<R: ResourceType> ResourcePool<R> {
    pub fn new(current: R, max: R, regen_rate: R) -> Self {
        assert!(current >= R::ZERO);
        assert!(current <= max);
        Self {
            current,
            max,
            regen_rate,
            _phantom: PhantomData::default(),
        }
    }

    pub fn current(&self) -> R {
        self.current
    }

    pub fn max(&self) -> R {
        self.max
    }

    pub fn set_current(&mut self, new_value: R) {
        self.current = new_value.clamp(R::ZERO, self.max);
    }

    pub fn set_max(&mut self, new_max: R) {
        self.max = new_max.clamp(R::ZERO, R::LOGICAL_MAX);
        if self.current > self.max {
            self.current = self.max
        }
    }
}

pub fn regen_resource<R: ResourceType + From<f32> + Into<f32>>(
    mut query: Query<&mut ResourcePool<R>>,
    time: Res<Time>,
) {
    for mut resource_pool in query.iter_mut() {
        let resource_gain_f32: f32 = resource_pool.regen_rate.into() * time.delta_seconds();
        let resource_gain: R = resource_gain_f32.into();
        *resource_pool += resource_gain;
    }
}

pub fn tick_regen_resource<R: ResourceType>(mut query: Query<&mut ResourcePool<R>>) {
    for mut resource_pool in query.iter_mut() {
        let delta_resource = resource_pool.regen_rate;
        *resource_pool += delta_resource;
    }
}

pub fn check_resource<R: ResourceType>(
    unit_query: Query<(&Abilities, &ResourcePool<R>)>,
    mut ability_query: Query<(&R, &mut Usable), With<Ability>>,
) {
    for (abilities, &resource_pool) in unit_query.iter() {
        for ability_entity in abilities.ability_list() {
            let (&resource_cost, mut usable) = ability_query.get_mut(ability_entity).unwrap();
            // Failing to have enough resources of one type can disable an ability,
            // but the converse is not true! An ability may be unusable for other reasons!
            if resource_pool < resource_cost {
                *usable = Usable(false);
            }
        }
    }
}

pub fn spend_resource<R: ResourceType>(
    mut unit_query: Query<(&Abilities, &mut ResourcePool<R>)>,
    ability_query: Query<&R, With<Ability>>,
) {
    for (abilities, mut resource_pool) in unit_query.iter_mut() {
        if abilities.active_ability.state == AbilityState::JustStarted {
            let active_ability_entity = abilities.active_ability.entity.unwrap();
            let &resource_cost = ability_query.get(active_ability_entity).unwrap();

            *resource_pool -= resource_cost;
        }
    }
}

mod trait_impls {
    use super::*;

    impl<R: ResourceType> Clone for ResourcePool<R> {
        fn clone(&self) -> Self {
            Self {
                current: self.current.clone(),
                max: self.max.clone(),
                regen_rate: self.regen_rate.clone(),
                _phantom: self._phantom.clone(),
            }
        }
    }

    impl<R: ResourceType> Copy for ResourcePool<R> {}

    impl<R: ResourceType> Add<R> for ResourcePool<R> {
        type Output = ResourcePool<R>;

        fn add(self, rhs: R) -> ResourcePool<R> {
            ResourcePool {
                current: self.current + rhs.min(self.max),
                max: self.max,
                regen_rate: self.regen_rate,
                _phantom: PhantomData::default(),
            }
        }
    }

    impl<R: ResourceType> Sub<R> for ResourcePool<R> {
        type Output = ResourcePool<R>;

        fn sub(self, other: R) -> ResourcePool<R> {
            let difference: R = self.current - other;

            ResourcePool {
                current: difference.max(R::ZERO),
                max: self.max,
                regen_rate: self.regen_rate,
                _phantom: PhantomData::default(),
            }
        }
    }

    impl<R: ResourceType> AddAssign<R> for ResourcePool<R> {
        fn add_assign(&mut self, other: R) {
            *self = *self + other;
        }
    }

    impl<R: ResourceType> SubAssign<R> for ResourcePool<R> {
        fn sub_assign(&mut self, other: R) {
            *self = *self - other;
        }
    }

    impl<R: ResourceType> PartialEq<R> for ResourcePool<R> {
        fn eq(&self, other: &R) -> bool {
            self.current == *other
        }
    }

    impl<R: ResourceType> PartialOrd<R> for ResourcePool<R> {
        fn partial_cmp(&self, other: &R) -> Option<Ordering> {
            Some(self.current.cmp(&other))
        }
    }
}
