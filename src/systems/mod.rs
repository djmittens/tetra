
use crate::components::*;
use specs::prelude::*;
use specs::System;
use log::*;

pub struct VisibilitySystem {}
impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, map::TetraMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (map, mut viewshed, mut players, pos): Self::SystemData) {
        let map = &map.buffer;

        for (player, pos, viewshed) in (&mut players, &pos, &mut viewshed).join() {
            if viewshed.dirty {
                Self::generate_viewshed(map, viewshed, pos);
                Self::update_fog_of_war(viewshed, player);
                viewshed.dirty = false;
            }
        }
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            if viewshed.dirty {
                Self::generate_viewshed(map, viewshed, pos);
                viewshed.dirty = false;
            }
        }
    }
}

pub struct MonsterAi {}

impl<'a> System<'a> for MonsterAi {
    type SystemData = (
        ReadExpect<'a, map::TetraMap>,
        ReadExpect<'a, (i32, i32)>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>);

    fn run(&mut self, (map, player_pos, name, viewshed, monster): Self::SystemData) {
        let map = &map.buffer;
        for (viewshed, name, _monster) in (&viewshed, &name, &monster).join() {
            let idx = map.xy_idx(player_pos.0, player_pos.1);
            if viewshed.visible_tiles.contains(&idx) {
                info!("{} considers their own existance", name.name);
            }
        }
    }
}



pub trait FogOfWarAlgorithm {
    fn generate_viewshed(map: &map::TileBuffer, viewshed: &mut Viewshed, position: &Position);
    fn update_fog_of_war(viewshed: &Viewshed, player: &mut Player);
}