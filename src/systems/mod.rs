use crate::components::*;
use log::*;
use rltk::Point;
use specs::prelude::*;
use specs::System;

pub struct VisibilitySystem {}
impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, map::TetraMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (map, mut viewshed, mut players, pos): Self::SystemData) {
        for (player, pos, viewshed) in (&mut players, &pos, &mut viewshed).join() {
            if viewshed.dirty {
                Self::generate_viewshed(&map, viewshed, pos);
                Self::update_fog_of_war(viewshed, player);
                viewshed.dirty = false;
            }
        }
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            if viewshed.dirty {
                Self::generate_viewshed(&map, viewshed, pos);
                viewshed.dirty = false;
            }
        }
    }
}

pub struct MonsterAi {}

impl<'a> System<'a> for MonsterAi {
    type SystemData = (
        WriteExpect<'a, map::TetraMap>,
        ReadExpect<'a, (i32, i32)>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (mut map, player_pos, name, mut viewshed, monster, mut pos): Self::SystemData,
    ) {
        for (viewshed, name, _monster, pos) in (&mut viewshed, &name, &monster, &mut pos).join() {
            let idx = map.nav_buffer.xy_idx(player_pos.0, player_pos.1);

            let distance =  rltk::DistanceAlg::Pythagoras.distance2d(rltk::Point::new(pos.x, pos.y), rltk::Point::new(player_pos.0, player_pos.1));
            if distance < 1.5 {
                // info!("{} Shouts insults towards {}", name.name, idx);
                // info!("{:?}\n {}", viewshed, viewshed.visible_tiles.contains(&idx));
            }

            if viewshed.visible_tiles.contains(&idx) {
                let path = rltk::a_star_search(
                    map.nav_buffer.xy_idx(pos.x, pos.y),
                    map.nav_buffer.xy_idx(player_pos.0, player_pos.1),
                    &mut *map,
                );
                // info!("{} {:?}", path.success, path.steps);
                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width();
                    pos.y = path.steps[1] as i32 / map.width();
                    viewshed.dirty = true;
                }
            }
        }
    }
}

pub trait FogOfWarAlgorithm {
    fn generate_viewshed(map: &map::TetraMap, viewshed: &mut Viewshed, position: &Position);
    fn update_fog_of_war(viewshed: &Viewshed, player: &mut Player);
}

pub struct MapIndexingSystem {}
impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, map::TetraMap>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>
    );

    fn run(&mut self, (mut map, pos, tile, ent): Self::SystemData) {
        map.gen_nav_buffer();
        map.clear_entities();
        for (pos, ent) in (&pos, &ent).join() {
            if tile.contains(ent) {
                map.nav_buffer.set(pos.x, pos.y, true);
            }
            map.entities.mutate(pos.x, pos.y, |x| x.push(ent));
        }
    }
}
