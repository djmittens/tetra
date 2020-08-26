use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
    error::NoError,
};
use specs_derive::*;
use std::collections::HashSet;
use serde::*;

pub mod map;
pub mod gamelog;
pub mod spawner;
pub use gamelog::GameLog;

#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from(pos: (i32, i32)) -> Self {
        Position { x: pos.0, y: pos.1 }
    }
}

impl Position {
}

#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles: HashSet<usize>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack{
    pub owner: Entity
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32
}

#[derive(Component, Debug)]
pub struct Monster;

#[derive(Component, Debug, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Player {
    pub revealed_tiles: HashSet<usize>,
}

#[derive(Component, Debug)]
pub struct BlocksTile {}


#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, ConvertSaveload,  Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}


#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some (suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {amount: vec! [amount]};
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug)]
pub struct WantsToDrinkPotion {
    pub potion: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
}