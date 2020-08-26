use crate::components::*;
use crate::systems::*;
use rltk::SmallVec;
use rltk::{field_of_view, Algorithm2D, BaseMap, Point, Rltk, RGB};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub order: i32,
}

impl Algorithm2D for map::TetraMap {
    fn dimensions(&self) -> Point {
        Point::new(self.width(), self.height())
    }
}

impl BaseMap for map::TetraMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.buffer.data[idx as usize] == map::TileType::Wall
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width() as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::<[(usize, f32); 10]>::new();

        let x = idx as i32 % self.width();
        let y = idx as i32 / self.width();
        let w = self.width() as usize;

        if is_exit_valid(self, x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if is_exit_valid(self, x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if is_exit_valid(self, x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if is_exit_valid(self, x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        if is_exit_valid(self, x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45))
        };
        if is_exit_valid(self, x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45))
        };
        if is_exit_valid(self, x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45))
        };
        if is_exit_valid(self, x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45))
        };
        // info!("{:?}", exits);
        exits
    }
}

fn is_exit_valid(map: &map::TetraMap, x: i32, y: i32) -> bool {
    let map = &map.nav_buffer;
    if x < 1 || x >= map.width || y < 1 || y >= map.height {
        false
    } else {
        !map.data[map.xy_idx(x, y)]
    }
}

impl FogOfWarAlgorithm for VisibilitySystem {
    fn generate_viewshed(map: &map::TetraMap, viewshed: &mut Viewshed, position: &Position) {
        viewshed.visible_tiles =
            field_of_view(Point::new(position.x, position.y), viewshed.range, &*map)
                .iter()
                .filter(|p| p.x >= 0 && p.x < map.width() && p.y >= 0 && p.y < map.height())
                .map(|p| map.buffer.xy_idx(p.x, p.y))
                .collect();
    }

    fn update_fog_of_war(viewshed: &Viewshed, player: &mut Player) {
        viewshed.visible_tiles.iter().cloned().for_each(|x| {
            player.revealed_tiles.insert(x);
        });
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<map::TetraMap>();
    let players = ecs.read_storage::<Player>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    for (player, viewshed) in (&players, &viewsheds).join() {
        let mut x = 0;
        let mut y = 0;
        for (idx, tile) in map.buffer.data.iter().enumerate() {
            let _pt = Point::new(x, y);
            if player.revealed_tiles.contains(&idx) {
                let glyph;
                let mut fg;
                match tile {
                    map::TileType::Floor => {
                        fg = RGB::from_f32(0.35, 0.5, 0.5);
                        glyph = rltk::to_cp437('.');
                    }
                    map::TileType::Wall => {
                        fg = RGB::from_f32(0.0, 1.0, 0.0);
                        glyph = rltk::to_cp437('#');
                    }
                }

                if !viewshed.visible_tiles.contains(&idx) {
                    fg = fg.to_greyscale();
                }

                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }

            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
}
