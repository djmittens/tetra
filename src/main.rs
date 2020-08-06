use components::*;
use rltk::{GameState, Rltk, RltkBuilder, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::collections::HashSet;
use log::*;

extern crate env_logger;
extern crate log;

mod components;
mod draw;
mod systems;
mod util;

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello World Rust Edition");

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        // let map = self.ecs.fetch::<map::TetraMap>();
        draw::draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<draw::Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        // let mut lw = LeftWalker {};
        // lw.run_now(&self.ecs);
        let mut vis = systems::VisibilitySystem {};
        let mut ai = systems::MonsterAi {};
        let mut mis = systems::MapIndexingSystem {};
        ai.run_now(&self.ecs);
        mis.run_now(&self.ecs);
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    let mut res = RunState::Paused;

    if let Some(key) = ctx.key {
         res = RunState::Running;
         match key {
            Up | K => try_move_player(0, -1, &mut gs.ecs),
            Left | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | L => try_move_player(1, 0, &mut gs.ecs),
            Down | J => try_move_player(0, 1, &mut gs.ecs),
            Y => try_move_player(-1, -1, &mut gs.ecs),
            U => try_move_player(1, -1, &mut gs.ecs),
            N => try_move_player(1, 1, &mut gs.ecs),
            B => try_move_player(-1, 1, &mut gs.ecs),
            _ => res = RunState::Paused,
        }
    }

    res
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<map::TetraMap>();

    let mut player_pos = ecs.write_resource::<(i32, i32)>();

    fn clamp(m: i32, v: i32) -> i32 {
        use std::cmp::{min, max};
        min(m, max(0, v))
    }

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {

        let new_x = clamp(map.width() - 1 , pos.x + delta_x);
        let new_y = clamp(map.height() - 1, pos.y + delta_y);


        for ent in map.entities.get(pos.x + delta_x, pos.y + delta_y) {
            if combat_stats.contains(*ent) {
                info!("From Hells heart i stab thee {:?}", ent);
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
        runstate: RunState::Running,
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

    let starting_room = {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = rltk::RandomNumberGenerator::new();
        let map = map::new_map_rooms_and_corridors(
            std::iter::from_fn(|| {
                let w = rng.range(MIN_SIZE, MAX_SIZE);
                let h = rng.range(MIN_SIZE, MAX_SIZE);
                let x = rng.roll_dice(1, 80 - w - 1) - 1;
                let y = rng.roll_dice(1, 50 - h - 1) - 1;
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
        res
    };

    for pos in starting_room {
        // insert the player location into the global store for some reason.
        gs.ecs.insert(pos.clone());
        gs.ecs
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
    }

    rltk::main_loop(context, gs)
}
