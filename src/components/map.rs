use crate::util::Rect;
use std::cmp::{max, min};

pub struct TetraMap {
    pub buffer: TileBuffer,
    pub rooms: Vec<Room>,
}

impl TetraMap {
    pub fn new(tile_map: TileBuffer) -> TetraMap {
        TetraMap {
            buffer: tile_map,
            rooms: Vec::new(),
        }
    }

    /// try and add the room to the game level, if we cant, then we return
    /// the room that it collides with
    pub fn try_add_room(&mut self, r: Room) -> Option<&Room> {
        let rooms = &mut self.rooms;
        if !rooms.iter().any(|x| r.intersect(x)) {
            TetraMap::apply_room(&r, &mut self.buffer);
            rooms.push(r);
            None
        } else {
            rooms.iter().find(|x| r.intersect(x))
        }
    }

    fn apply_room(room: &Room, map: &mut TileBuffer) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                map.set(x, y, TileType::Floor);
            }
        }
    }
}

pub type TileBuffer = Buffer2D<TileType>;

/// A TileMap is a resource that is shared by the components.
// pub type TileMap = Vec<TileType>;
pub struct Buffer2D<T> {
    pub height: i32,
    pub width: i32,
    pub tiles: Vec<T>,
}

impl<T> Buffer2D<T>
where
    T: Clone,
{
    fn new(width: i32, height: i32, fill: T) -> Buffer2D<T> {
        Buffer2D {
            width,
            height,
            tiles: vec![fill; (width * height) as usize],
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn set(&mut self, x: i32, y: i32, tile: T) {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] = tile;
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub type Room = Rect;

/// Make a new map or something.
pub fn new_map<I>(wall_ids: I) -> TileBuffer
where
    I: IntoIterator<Item = usize>,
{
    // let mut map = vec![TileType::Floor; 80 * 50];
    let mut map = TileBuffer::new(80, 50, TileType::Floor);

    for x in 0..80 {
        map.set(x, 0, TileType::Wall);
        map.set(x, 49, TileType::Wall);
    }

    for y in 0..50 {
        map.set(0, y, TileType::Wall);
        map.set(79, y, TileType::Wall);
    }

    wall_ids.into_iter().for_each(|idx| {
        if idx != map.xy_idx(40, 25) {
            map.tiles[idx] = TileType::Wall;
        }
    });
    map
}

pub fn new_map_rooms_and_corridors<T>(rooms: T) -> TetraMap
where
    T: IntoIterator<Item = Room>,
{
    // let mut map = vec![TileType::Wall; 80 * 50];
    let mut level = TetraMap::new(TileBuffer::new(80, 50, TileType::Wall));
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
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut TileBuffer, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}
