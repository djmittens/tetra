use components::*;
use rltk::{GameState, Rltk, RltkBuilder, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::collections::HashSet;

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
            self.runstate = RunState::Paused ;
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
        vis.run_now(&self.ecs);
        ai.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => {return RunState::Paused}
        Some(key) => match key {
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            _ => {return RunState::Paused}
        },
    }

    RunState::Running
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<components::Viewshed>();
    let map = ecs.fetch::<map::TetraMap>();

    let mut player_pos = ecs.write_resource::<(i32, i32)>();


    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        player_pos.0 = pos.x;
        player_pos.1 = pos.y;

        pos.try_move(&map.buffer, delta_x, delta_y);
        viewshed.dirty = true;
    }
}

fn main() -> rltk::RltkError {
    let context = RltkBuilder::simple80x50().with_title("Tetra").build()?;
    let mut gs = State { ecs: World::new(), runstate: RunState::Running};

    env_logger::init();

    gs.ecs.register::<Position>();
    gs.ecs.register::<draw::Renderable>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Player>();

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
        let res = rng
            .random_slice_entry(map.rooms.as_slice())
            .map(|x| x.center());

        for (i, room) in map.rooms.iter().skip(1).enumerate() {
            let (x, y) = room.center();
            let glyph: u16;
            let name: String;

            match rng.roll_dice(1, 2) {
                1 => {
                    glyph = rltk::to_cp437('g');
                    name = format!("{} #{}", "Globlin", i);
                },
                _ => {
                    glyph = rltk::to_cp437('o');
                    name = format!("{} #{}", "Orc", i);
                },
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
                .with(Name {name})
                .with(draw::Renderable {
                    glyph,
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                })
                .build();
        }

        gs.ecs.insert(map);
        res
    };

    starting_room.iter().for_each(|pos| {
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
            .with (Name{name: "Player".into()})
            .build();
    });

    rltk::main_loop(context, gs)
}
