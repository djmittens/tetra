use rltk::{Rltk, GameState, RltkBuilder};

struct State{}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1,1, "Hello World Rust Edition");
    }
}

fn main() -> rltk::RltkError {
    let context = RltkBuilder::simple80x50()
        .with_title("Tetra")
        .build()?;
    let gs = State{ };
    rltk::main_loop(context, gs)
}
