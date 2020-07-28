use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

pub mod map;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<&(i32, i32)> for Position {
    fn from(pos: &(i32, i32)) -> Self {
        Position{x: pos.0, y: pos.1}
    }
    
}

impl Position {
    pub fn try_move(self: &mut Position, map: &map::TileBuffer, delta_x: i32, delta_y: i32) {
        let destination_idx = map.xy_idx(self.x + delta_x, self.y + delta_y);
        if map.tiles[destination_idx] != map::TileType::Wall {
            self.x = min(79, max(0, self.x + delta_x));
            self.y = min(49, max(0, self.y + delta_y));
        }
    }
}

#[derive(Component)]
pub struct LeftMover {}

#[derive(Component, Debug)]
pub struct Player {}