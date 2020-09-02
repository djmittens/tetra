// use crate::util::Rng;
use crate::components::*;

//TODO clearly  less than ideal
use crate::draw;
use crate::util::{Rect, RngResource};
use rltk;
use rltk::RGB;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with::<Position>((player_x, player_y).into())
        .with(draw::Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            order: 0,
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
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            power: 5,
            defense: 2,
        })
        // .with(BlocksTile{})
        .build()
}

pub fn spawn_room(
    ecs: &mut World,
    room: &Rect,
    SpawnerSettings {
        max_monsters,
        max_items,
    }: SpawnerSettings,
) {
    let mut spawn_points: HashSet<(usize, usize)> = HashSet::new();
    let mut item_points: HashSet<(usize, usize)> = HashSet::new();

    //TODO i can make this so much better by instead generating an infinite stream of valid spawn points, and then taking the right amount for each type
    {
        let mut rng = ecs.write_resource::<RngResource>();
        let n_monsters = rng.between(0, max_monsters);
        let n_items = rng.between(0, max_items);

        for _i in 0..n_monsters {
            let mut added = false;
            while !added {
                let xy = (
                    rng.between(room.x1 + 1, room.x2) as usize,
                    rng.between(room.y1 + 1, room.y2) as usize,
                );

                if !spawn_points.contains(&xy) {
                    spawn_points.insert(xy);
                    added = true;
                }
            }
        }

        for _i in 0..n_items {
            let mut added = false;
            while !added {
                let xy = (
                    rng.between(room.x1 + 1, room.x2) as usize,
                    rng.between(room.y1 + 1, room.y2) as usize,
                );
                if !item_points.contains(&xy) && !spawn_points.contains(&xy) {
                    item_points.insert(xy);
                    added = true;
                }
            }
        }
    }

    for (x, y) in spawn_points.iter() {
        random_monster(ecs, *x as i32, *y as i32);
    }

    for (x, y) in item_points.iter() {
        random_item(ecs, *x as i32, *y as i32);
    }
}
// pub fn entity(ecs: &mut World, ent: &Entity, pos_x: i32, pos_y: i32)

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32 = {
        let mut rng = ecs.write_resource::<RngResource>();
        rng.between(0, 2)
    };

    if roll == 1 {
        orc(ecs, x, y);
    } else {
        goblin(ecs, x, y);
    }
}

pub fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, 'o', "Orc")
}

pub fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, 'g', "Goblin")
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: char, name: S) {
    ecs.create_entity()
        .with(Viewshed {
            visible_tiles: HashSet::new(),
            range: 8,
            dirty: true,
        })
        .with(Position { x, y })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(draw::Renderable {
            glyph: rltk::to_cp437(glyph),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            order: 1,
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            power: 4,
            defense: 1,
        })
        .build();
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32 = {
        let mut rng = ecs.write_resource::<RngResource>();
        rng.between(0, 3)
    };

    match roll {
        1 => {health_potion(ecs, x, y)}
        2 => {fireball_scroll(ecs, x, y)}
        _ => {magic_missile_scroll(ecs, x, y)}
    }
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(draw::Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {
            name: "Health Potion".into(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(draw::Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {
            name: "AOE Fireball Scroll".into(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged{ range: 6})
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(draw::Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".into(),
        })
        .with(Item {})
        .with(Ranged{ range: 6})
        .with(Consumable {})
        .with(InflictsDamage { damage: 8 })
        .build();
}

pub struct SpawnerSettings {
    pub max_monsters: i32,
    pub max_items: i32,
}

impl Default for SpawnerSettings {
    fn default() -> Self {
        SpawnerSettings {
            max_monsters: 4,
            max_items: 4,
        }
    }
}
