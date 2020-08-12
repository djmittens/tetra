use rltk::{RGB, Rltk, Point};
use specs::prelude::*;
use crate::components::{map::TetraMap, *};


pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for(_player, stats) in (&players, &combat_stats).join() {
        let health = format!("Hp: {} / {}", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let log = ecs.fetch::<GameLog>();
    let y = 44; // FIXME should definitely not be hardcoded.
    for (x, s) in log.entries.iter().rev().enumerate().take(5) {
        ctx.print(2, y + x, s);
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<TetraMap>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mx, my) = ctx.mouse_pos();
    // FIXME this is a prime example of where NaN poisoning can come in
    if mx >= map.width() || my >= map.height() {return;}
    let mut tooltip: Vec<String> = Vec::new();
    for(name, pos) in (&names, &positions).join() {
        if pos.x == mx && pos.y == my {
            tooltip.push(name.name.clone());
        }
    }
    if !tooltip.is_empty() {
        let mut width : i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {width = s.len() as i32;}
        }

        width += 3;

        if mx > 40 {
            let arrow_pos = Point::new(mx - 2, my);
            let left_x = mx - width;
            let mut y = my;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0 .. padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mx + 1, my);
            let left_x = mx + 3;
            let mut y = my;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
        }

    }
}