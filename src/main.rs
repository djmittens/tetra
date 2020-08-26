use components::*;
use log::*;
use rltk::{GameState, Rltk, RltkBuilder, VirtualKeyCode};
use specs::prelude::*;


extern crate env_logger;
extern crate log;
extern crate specs;

mod components;
mod draw;
mod gui;
mod systems;
mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> rltk::RltkError {
    let mut context = RltkBuilder::simple80x50().with_title("Tetra").build()?;
    context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };

    env_logger::init();

    gs.ecs.register::<Position>();
    gs.ecs.register::<draw::Renderable>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToDrinkPotion>();
    gs.ecs.register::<WantsToDropItem>();

    {
        let rng: util::RngResource = Box::new(rltk::RandomNumberGenerator::new());
        gs.ecs.insert(rng);
    }

    let starting_room = {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        const MAP_WIDTH: i32 = 80;
        const MAP_HEIGHT: i32 = 43;

        let map: map::TetraMap;

        let res = {
            let mut rng = gs.ecs.write_resource::<util::RngResource>();
            map = map::new_map_rooms_and_corridors(
                MAP_WIDTH,
                MAP_HEIGHT,
                std::iter::from_fn(|| {
                    let w = rng.between(MIN_SIZE, MAX_SIZE);
                    let h = rng.between(MIN_SIZE, MAX_SIZE);
                    let x = rng.between(1, MAP_WIDTH - w) - 1;
                    let y = rng.between(1, MAP_HEIGHT - h) - 1;
                    Some(map::Room::new(x, y, w, h))
                })
                .take(MAX_ROOMS),
            );
            util::choose_element(rng.as_mut(), map.rooms.as_slice())
        };

        for res in res {
            for (_i, room) in map.rooms.iter().enumerate() {
                if room != res {
                    spawner::spawn_room(&mut gs.ecs, room, spawner::SpawnerSettings::default());
                    // let (x, y) = room.center();
                    // spawner::random_monster(&mut gs.ecs, x, y);
                }
            }
        }

        let res = res.map(|x| x.center());
        gs.ecs.insert(map);
        gs.ecs.insert(GameLog {
            entries: vec!["Welcome to tetra, young traveler !".to_string()],
        });
        res
    };

    for (x, y) in starting_room {
        // insert the player location into the global store for some reason.
        // gs.ecs.insert(pos.clone());

        // Player entity yay
        let player_entity = spawner::player(&mut gs.ecs, x, y);
        gs.ecs.insert(player_entity);
    }

    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs)
}

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        delete_the_dead(&mut self.ecs);

        // let map = self.ecs.fetch::<map::TetraMap>();
        draw::draw_map(&self.ecs, ctx);

        // Draw other renderable bs.
        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<draw::Renderable>();
            //TODO only draw when inside of the players viewshed.

            for (pos, render) in (&positions, &renderables).join() {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
        gui::draw_ui(&self.ecs, ctx);

        gui::draw_tooltips(&self.ecs, ctx);

        let mut newrunstate = *self.ecs.fetch::<RunState>();

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            },
            RunState::InventoryScreen => {
                let res = gui::show_inventory(&mut self.ecs, ctx);
                match res {
                    (gui::ItemMenuResult::Cancel, _) => newrunstate = RunState::AwaitingInput,
                    (gui::ItemMenuResult::NoResponse, _) => {},
                    (gui::ItemMenuResult::Selected, Some(item)) => {
                        let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion{potion: item}).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    },
                    (_, None) => {},
                }
            },
            RunState::DropItemScreen => {
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }


        ctx.print(1, 1, format!("Tetra Early Preview v{}", VERSION));
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = systems::VisibilitySystem {};
        let mut melee = systems::MeleeCombatSystem {};
        let mut damage = systems::DamageSystem {};
        let mut ai = systems::MonsterAi {};
        let mut mis = systems::MapIndexingSystem {};
        let mut loot_system = systems::ItemCollectionSystem {};
        let mut potions = systems::PotionUseSystem {};
        let mut drop_items = systems::LootSystem {};

        potions.run_now(&self.ecs);
        ai.run_now(&self.ecs);
        mis.run_now(&self.ecs);
        drop_items.run_now(&self.ecs);
        loot_system.run_now(&self.ecs);
        vis.run_now(&self.ecs);
        melee.run_now(&self.ecs);
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    InventoryScreen,
    DropItemScreen,
}

fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let entities = ecs.entities();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                if let Some(_) = players.get(entity) {
                    log.say("You are dead".into());
                } else {
                    if let Some(victim_name) = names.get(entity) {
                        log.say(format!("{} is dead", &victim_name.name));
                    }
                    dead.push(entity);
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    let mut res = RunState::AwaitingInput;

    if let Some(key) = ctx.key {
        res = RunState::PlayerTurn;
        match key {
            Up | K => try_move_player(0, -1, &mut gs.ecs),
            Left | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | L => try_move_player(1, 0, &mut gs.ecs),
            Down | J => try_move_player(0, 1, &mut gs.ecs),
            Y => try_move_player(-1, -1, &mut gs.ecs),
            U => try_move_player(1, -1, &mut gs.ecs),
            N => try_move_player(1, 1, &mut gs.ecs),
            B => try_move_player(-1, 1, &mut gs.ecs),
            G => get_item(&mut gs.ecs),
            I => res = RunState::InventoryScreen,
            D => res = RunState::DropItemScreen,
            _ => res = RunState::AwaitingInput,
        }
    }

    res
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<map::TetraMap>();
    let entities = ecs.entities();


    fn clamp(m: i32, v: i32) -> i32 {
        use std::cmp::{max, min};
        min(m, max(0, v))
    }

    for (ent, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let new_x = clamp(map.width() - 1, pos.x + delta_x);
        let new_y = clamp(map.height() - 1, pos.y + delta_y);

        for potential_target in map.entities.get(pos.x + delta_x, pos.y + delta_y) {
            if combat_stats.contains(*potential_target) {
                debug!("From Hells heart i stab thee {:?}", potential_target);
                wants_to_melee
                    .insert(
                        ent,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed"); // FIXME i dont like this error handling.
                return; // so we dont move after attacking, i guess thats a way to do it, i dont like it FIXME
            }
        }

        if !map.is_blocked(new_x, new_y) {
            pos.x = new_x;
            pos.y = new_y;
            viewshed.dirty = true;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player = ecs.fetch::<Entity>();
    let player = *player;
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        let p_pos = positions.get(player).unwrap();
        if position.x == p_pos.x && position.y == p_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pickup.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(player, WantsToPickupItem{collected_by: player, item}).expect("Could not notify of item pickup");
        }
    }
}

impl util::Rng for rltk::RandomNumberGenerator {
    fn next_int(&mut self) -> i32 {
        self.rand()
    }
    fn between(&mut self, k: i32, n: i32) -> i32 {
        self.range(k, n)
    }
}
