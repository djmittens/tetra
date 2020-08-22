use crate::components::*;

use specs::prelude::*;

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
        ReadExpect<'a, Entity>,
        ReadExpect<'a, crate::RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(
        &mut self,
        (
            mut map,
            player_entity,
            run_state,
            entities,
            mut viewshed,
            monster,
            mut positions,
            mut wants_to_melee,
        ): Self::SystemData,
    ) {
        // TODO get the RNG state out of here
        use rltk::a_star_search;
        if *run_state != crate::RunState::MonsterTurn {
            return;
        }

        let player_entity = *player_entity;
        let Position { x: px, y: py } = *positions
            .get(player_entity)
            .expect("Player is expected to be positional");

        for (ent, viewshed, _monster, pos) in
            (&entities, &mut viewshed, &monster, &mut positions).join()
        {
            let idx = map.nav_buffer.xy_idx(px, py);

            if viewshed.visible_tiles.contains(&idx) {
                let path = a_star_search(
                    // map.nav_buffer.xy_idx(pos.x, pos.y),
                    map.nav_buffer.xy_idx(pos.x, pos.y),
                    map.nav_buffer.xy_idx(px, py),
                    // *target,
                    &mut *map,
                );
                if path.success && path.steps.len() > 2 {
                    pos.x = path.steps[1] as i32 % map.width();
                    pos.y = path.steps[1] as i32 / map.width();
                    viewshed.dirty = true;
                }
            }

            let distance = rltk::DistanceAlg::Pythagoras
                .distance2d(rltk::Point::new(pos.x, pos.y), rltk::Point::new(px, py));

            if distance < 1.5 {
                wants_to_melee
                    .insert(
                        ent,
                        WantsToMelee {
                            target: player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
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
        Entities<'a>,
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

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );
    fn run(
        &mut self,
        (entities, mut game_log, mut want_melee, names, combat_stats, mut suffer_damage): Self::SystemData,
    ) {
        for (_ent, want_melee, name, stats) in
            (&entities, &want_melee, &names, &combat_stats).join()
        {
            let target_stats = combat_stats.get(want_melee.target).unwrap();
            if target_stats.hp > 0 {
                let target_name = names.get(want_melee.target).unwrap();
                let damage = i32::max(0, stats.power - target_stats.defense);

                if damage == 0 {
                    game_log.say(format!(
                        "{} is unable to hurt {}",
                        name.name, target_name.name
                    ));
                } else {
                    game_log.say(format!(
                        "{} hits {}, for {} hp",
                        name.name, target_name.name, damage
                    ));
                    SufferDamage::new_damage(&mut suffer_damage, want_melee.target, damage);
                }
            }
        }

        want_melee.clear();
    }
}

pub struct DamageSystem {}
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, (mut stats, mut damage): Self::SystemData) {
        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }
        damage.clear();
    }
}

pub struct ItemCollectionSystem {}
impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );
    fn run(
        &mut self,
        (player, mut game_log, mut pickup_items, mut positions, names, mut backpacks): Self::SystemData,
    ) {
        for pickup in pickup_items.join() {
            positions.remove(pickup.item);
            backpacks
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert into the backpack");

            if pickup.collected_by == *player {
                game_log.entries.push(format!(
                    "You have picked up {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }
        pickup_items.clear();
    }
}
