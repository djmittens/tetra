use specs::prelude::*;
use specs_derive::Component;
use rltk::{Rltk, RGB, Algorithm2D, Point, BaseMap, field_of_view};
use crate::components::{map, Position};

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32
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
impl <'a> System<'a> for VisibilitySystem {
    type SystemData = (ReadExpect<'a, map::TetraMap>, WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>);

    fn run(&mut self, (map, mut viewshed, pos) : Self::SystemData) {
        let map = &map.buffer;
        for(viewshed, pos) in (&mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p|p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height)
        }
    }
}

pub fn draw_map(map: &[map::TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;

    for tile in map.iter() {
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

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}