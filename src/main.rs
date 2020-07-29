use components::*;
use rltk::{GameState, Rltk, RltkBuilder, VirtualKeyCode, RGB};
use specs::prelude::*;

mod components;
mod util;
mod draw;

struct State {
    ecs: World,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello World Rust Edition");

        self.run_systems();
        player_input(self, ctx);
        let map = self.ecs.fetch::<map::TileBuffer>();
        draw::draw_map(&map.tiles, ctx);

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
        let mut vis = draw::VisibilitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<map::TileBuffer>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.try_move(&map, delta_x, delta_y);
    }
}


fn main() -> rltk::RltkError {
    let context = RltkBuilder::simple80x50().with_title("Tetra").build()?;
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<draw::Renderable>();
    // gs.ecs.register::<LeftMover>();
    gs.ecs.register::<draw::Viewshed>();
    gs.ecs.register::<Player>();

    let starting_room = {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = rltk::RandomNumberGenerator::new();
        let map::TetraMap{buffer, rooms} = map::new_map_rooms_and_corridors(
            std::iter::from_fn(|| {
                let w = rng.range(MIN_SIZE, MAX_SIZE);
                let h = rng.range(MIN_SIZE, MAX_SIZE);
                let x = rng.roll_dice(1, 80 - w - 1) - 1;
                let y = rng.roll_dice(1, 50 - h - 1) - 1;
                Some(map::Room::new(x, y, w, h))
            })
            .take(MAX_ROOMS),
        );
        gs.ecs.insert(buffer);
        rng.random_slice_entry(rooms.as_slice())
            .map(|x| x.center())
    };

    starting_room.iter().for_each(|pos|{
        gs.ecs
            .create_entity()
            .with::<Position>(pos.into())
            .with(draw::Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Player {})
            .with(draw::Viewshed{visible_tiles: Vec::new(), range: 8})
            .build();
    });


    rltk::main_loop(context, gs)
}
