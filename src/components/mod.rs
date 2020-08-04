use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};
use std::collections::HashSet;

pub mod map;

#[derive(Component, Debug)]
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
    pub fn try_move(self: &mut Position, map: &map::TetraMap, delta_x: i32, delta_y: i32) {
        if !map.is_blocked(self.x + delta_x, self.y + delta_y) {
            self.x = min(79, max(0, self.x + delta_x));
            self.y = min(49, max(0, self.y + delta_y));
        }
    }
}

#[derive(Component, Debug)]
pub struct Viewshed {
    pub visible_tiles: HashSet<usize>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug)]
pub struct Monster;

#[derive(Component, Debug)]
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