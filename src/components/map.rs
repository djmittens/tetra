use crate::util::Rect;
use specs::prelude::*;
use std::cmp::{max, min};

pub struct TetraMap {
    pub buffer: TileBuffer,
    pub rooms: Vec<Room>,
    pub nav_buffer: Buffer2D<bool>,
    pub entities: Buffer2D<Vec<Entity>>,
}

impl TetraMap {
    pub fn new(buffer: TileBuffer) -> TetraMap {
        let mut nav_buffer = Buffer2D::new(buffer.width, buffer.height, false);
        update_nav_buffer(&buffer.data, &mut nav_buffer.data);
        TetraMap {
            entities: Buffer2D::new(buffer.width, buffer.height, Vec::new()),
            rooms: Vec::new(),
            nav_buffer,
            buffer,
        }
    }

    pub fn xy(&self, idx: usize)-> (i32, i32) {
        (idx as i32 % self.buffer.width,  idx as i32 / self.buffer.width)
    }

    pub fn width(&self) -> i32 {
        self.buffer.width
    }

    pub fn height(&self) -> i32 {
        self.buffer.height
    }

    pub fn _dimensions(&self) -> (i32, i32) {
        (self.width(), self.height())
    }

    pub fn gen_nav_buffer(&mut self) {
        update_nav_buffer(&self.buffer.data, &mut self.nav_buffer.data);
    }

    pub fn clear_entities(&mut self) {
        for e in self.entities.data.iter_mut() {
            e.clear();
        }
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.nav_buffer.data[self.nav_buffer.xy_idx(x, y)]
    }

    /// try and add the room to the game level, if we cant, then we return
    /// the room that it collides with
    pub fn try_add_room(&mut self, r: Room) -> Option<&Room> {
        let rooms = &mut self.rooms;
        if !rooms.iter().any(|x| r.intersect(x)) {
            TetraMap::apply_room(&r, &mut self.buffer, &mut self.nav_buffer);
            rooms.push(r);
            None
        } else {
            rooms.iter().find(|x| r.intersect(x))
        }
    }

    fn apply_room(room: &Room, map: &mut TileBuffer, nav_map: &mut Buffer2D<bool>) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                map.set(x, y, TileType::Floor);
                nav_map.set(x, y, false);
            }
        }
    }
}

fn update_nav_buffer(tiles: &Vec<TileType>, nav: &mut Vec<bool>) {
    for (i, tile) in tiles.iter().enumerate() {
        nav[i] = tile == &TileType::Wall;
    }
}

pub type TileBuffer = Buffer2D<TileType>;

/// A TileMap is a resource that is shared by the components.
// pub type TileMap = Vec<TileType>;
pub struct Buffer2D<T> {
    pub height: i32,
    pub width: i32,
    pub data: Vec<T>,
}

impl<T> Buffer2D<T>
where
    T: Clone + PartialEq,
{
    fn new(width: i32, height: i32, fill: T) -> Buffer2D<T> {
        Buffer2D {
            width,
            height,
            data: vec![fill; (width * height) as usize],
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn set(&mut self, x: i32, y: i32, tile: T) {
        let idx = self.xy_idx(x, y);
        self.data[idx] = tile;
    }

    pub fn get(&self, x: i32, y: i32) -> &T {
        let idx = self.xy_idx(x, y);
        &self.data[idx]
    }

    pub fn mutate<F>(&mut self, x: i32, y: i32, func: F)
    where
        F: Fn(&mut T) -> (),
    {
        let idx = self.xy_idx(x, y);
        func(&mut self.data[idx]);
    }

    pub fn _contains_at(&self, x: i32, y: i32, tile: &T) -> bool {
        let idx = self.xy_idx(x, y);
        &self.data[idx] == tile
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

pub type Room = Rect;


pub fn new_map_rooms_and_corridors<T>(width: i32, height: i32, rooms: T) -> TetraMap
where
    T: IntoIterator<Item = Room>,
{
    // let mut map = vec![TileType::Wall; 80 * 50];
    let mut level = TetraMap::new(TileBuffer::new(width, height, TileType::Wall));
    rooms.into_iter().for_each(|r| {
        level.try_add_room(r);
    });

    for (r, p) in level.rooms.iter().skip(1).zip(level.rooms.iter()) {
        let (r_x, r_y) = r.center();
        let (p_x, p_y) = p.center();

        apply_horizontal_tunnel(&mut level.buffer, p_x, r_x, p_y);
        apply_vertical_tunnel(&mut level.buffer, p_y, r_y, r_x);
    }
    level
}

fn apply_horizontal_tunnel(map: &mut TileBuffer, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.data.len() {
            map.data[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut TileBuffer, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.data.len() {
            map.data[idx as usize] = TileType::Floor;
        }
    }
}
