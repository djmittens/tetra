// use crate::util::Rng;
use crate::components::*;
use specs::prelude::*;
//TODO clearly  less than ideal
use crate::draw;
use crate::util::{Rng, RngResource, choose_element};
use rltk;
use rltk::RGB;

// pub fn entity(ecs: &mut World, ent: &Entity, pos_x: i32, pos_y: i32)

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with::<Position>((player_x, player_y).into())
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
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            power: 5,
            defense: 2,
        })
        // .with(BlocksTile{})
        .build()
}

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32 = {
        let mut rng = ecs.write_resource::<RngResource>();
        rng.between(1, 2)
    };

    if roll == 1 {
        orc(ecs, x, y);
    } else {
        goblin(ecs, x, y);
    }
}

pub fn orc(ecs: &mut World, x: i32, y: i32){
    monster(ecs, x, y, 'o', "Orc")
}

pub fn goblin(ecs: &mut World, x: i32, y: i32){
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
