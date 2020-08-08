use components::*;
use rltk::{GameState, Rltk, RltkBuilder, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::collections::HashSet;
use log::*;

extern crate env_logger;
extern crate log;
extern crate specs;

mod components;
mod gui;
mod draw;
mod systems;
mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

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
            }

        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        delete_the_dead(&mut self.ecs);

        // let map = self.ecs.fetch::<map::TetraMap>();
        draw::draw_map(&self.ecs, ctx);
        gui::draw_ui(&self.ecs, ctx);

        // Draw other renderable bs.
        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<draw::Renderable>();
            //TODO only draw when inside of the players viewshed.

            for (pos, render) in (&positions, &renderables).join() {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        ctx.print(1, 1, format!("Tetra Early Preview v{}", VERSION));
    }
}

impl State {
    fn run_systems(&mut self) {
        // let mut lw = LeftWalker {};
        // lw.run_now(&self.ecs);
        let mut vis = systems::VisibilitySystem {};
        let mut melee = systems::MeleeCombatSystem {};
        let mut damage = systems::DamageSystem {};
        let mut ai = systems::MonsterAi {};
        let mut mis = systems::MapIndexingSystem {};
        mis.run_now(&self.ecs);
        ai.run_now(&self.ecs);
        melee.run_now(&self.ecs);
        damage.run_now(&self.ecs);
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    // Paused,
    // Running,
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let entities = ecs.entities();
        let players = ecs.read_storage::<Player>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                if let Some(_) = players.get(entity) {
                    info!("You are dead");
                } else {
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

    let mut player_pos = ecs.write_resource::<(i32, i32)>();

    fn clamp(m: i32, v: i32) -> i32 {
        use std::cmp::{min, max};
        min(m, max(0, v))
    }

    for (ent, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {

        let new_x = clamp(map.width() - 1 , pos.x + delta_x);
        let new_y = clamp(map.height() - 1, pos.y + delta_y);


        for potential_target in map.entities.get(pos.x + delta_x, pos.y + delta_y) {
            if combat_stats.contains(*potential_target) {
                debug!("From Hells heart i stab thee {:?}",  potential_target);
                wants_to_melee.insert(ent, WantsToMelee{target: *potential_target}).expect("Add target failed"); // FIXME i dont like this error handling.
                return; // so we dont move after attacking, i guess thats a way to do it, i dont like it FIXME
            }
        }

        if !map.is_blocked(new_x, new_y) {
            pos.x = new_x;
            pos.y = new_y;
            player_pos.0 = new_x;
            player_pos.1 = new_y;
            viewshed.dirty = true;
        }
    }
}

fn main() -> rltk::RltkError {
    let context = RltkBuilder::simple80x50().with_title("Tetra").build()?;
    let mut gs = State {
        ecs: World::new(),
    };

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
    gs.ecs.register::<SufferDamage>();

    let starting_room = {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        const MAP_WIDTH: i32 = 80;
        const MAP_HEIGHT: i32 = 43;

        let mut rng = rltk::RandomNumberGenerator::new();

        let map = map::new_map_rooms_and_corridors(
            MAP_WIDTH, MAP_HEIGHT,  
            std::iter::from_fn(|| {
                let w = rng.range(MIN_SIZE, MAX_SIZE);
                let h = rng.range(MIN_SIZE, MAX_SIZE);
                let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
                let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;
                Some(map::Room::new(x, y, w, h))
            })
            .take(MAX_ROOMS),
        );
        let res = rng.random_slice_entry(map.rooms.as_slice());

        for res in res {
            for (i, room) in map.rooms.iter().enumerate() {
                if room != res {
                    let (x, y) = room.center();
                    let glyph: u16;
                    let name: String;

                    match rng.roll_dice(1, 2) {
                        1 => {
                            glyph = rltk::to_cp437('g');
                            name = format!("{} #{}", "Globlin", i);
                        }
                        _ => {
                            glyph = rltk::to_cp437('o');
                            name = format!("{} #{}", "Orc", i);
                        }
                    }

                    gs.ecs
                        .create_entity()
                        .with(Viewshed {
                            visible_tiles: HashSet::new(),
                            range: 8,
                            dirty: true,
                        })
                        .with(Position { x, y })
                        .with(Monster {})
                        .with(Name { name })
                        .with(draw::Renderable {
                            glyph,
                            fg: RGB::named(rltk::RED),
                            bg: RGB::named(rltk::BLACK),
                        })
                        .with(BlocksTile {})
                        .with(CombatStats{
                            max_hp: 16,
                            hp: 16,
                            power: 4,
                            defense: 1,
                        })
                        .build();
                }
            }
        }

        let res = res.map(|x| x.center());
        gs.ecs.insert(map);
        gs.ecs.insert(GameLog{entries: vec!["Welcome to tetra, young traveler !".to_string()]});
        res
    };

    for pos in starting_room {
        // insert the player location into the global store for some reason.
        gs.ecs.insert(pos.clone());
        let player_entity = gs.ecs
            .create_entity()
            .with::<Position>(pos.into())
            .with(draw::Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Player {
                revealed_tiles: HashSet::new(),
            })
            .with(Viewshed {
                visible_tiles: HashSet::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: "Player".into(),
            })
            .with(CombatStats{
                max_hp: 30,
                hp: 30,
                power: 5,
                defense: 2,
            })
            .with(BlocksTile{})
            .build();
        gs.ecs.insert(player_entity);
    }

    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs)
}
