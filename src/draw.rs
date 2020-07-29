use crate::components::{map, Player, Position};
use rltk::{field_of_view, Algorithm2D, BaseMap, Point, Rltk, RGB};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<usize>,
    pub range: i32,
}

impl Algorithm2D for map::TileBuffer {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for map::TileBuffer {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == map::TileType::Wall
    }
}

pub struct VisibilitySystem {}
impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, map::TetraMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (mut map, mut viewshed, pos): Self::SystemData) {
        let map = &map.buffer;
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map)
                .iter()
                .filter(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height)
                .map(|p| map.xy_idx(p.x, p.y))
                .collect();
        }
    }
}

pub struct FogOfWarSystem {}
impl<'a> System<'a> for FogOfWarSystem {
    type SystemData = (
        WriteStorage<'a, Player>,
        ReadStorage<'a, Viewshed>,
    );

    fn run(&mut self, (mut players, sheds): Self::SystemData) {
        for (player, viewshed) in (&mut players, &sheds).join() {
            viewshed.visible_tiles.iter().cloned().for_each(|x| {player.revealed_tiles.insert(x); });
        }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {

    let map = ecs.fetch::<map::TetraMap>();
    let players = ecs.read_storage::<Player>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    for (player, _viewshed) in (&players, &viewsheds).join() {
        let mut x = 0;
        let mut y = 0;
        for (idx, tile) in map.buffer.tiles.iter().enumerate() {
            let _pt = Point::new(x, y);
            if player.revealed_tiles.contains(&idx) {
                match tile {
                    map::TileType::Floor => {
                        ctx.set(
                            x,
                            y,
                            RGB::from_f32(0.35, 0.5, 0.5),
                            RGB::from_f32(0., 0., 0.),
                            rltk::to_cp437('.'),
                        );
                    }
                    map::TileType::Wall => {
                        ctx.set(
                            x,
                            y,
                            RGB::from_f32(0.0, 1.0, 0.0),
                            RGB::from_f32(0., 0., 0.),
                            rltk::to_cp437('#'),
                        );
                    }
                }
            }

            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}
