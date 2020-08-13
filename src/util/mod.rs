pub use rectangle::Rect;
pub mod rectangle;


pub trait Rng {
    fn next_int(&mut self) -> i32;
    fn between(&mut self, k: i32, n: i32 ) -> i32;
}


pub fn choose_element<'a,R : Rng + ?Sized, T> (rng: &mut R, slice: &'a [T]) -> Option<&'a T> {
    if slice.is_empty() {
        None
    } else {
        Some(&slice[rng.between(0, slice.len()  as i32 ) as usize])
    }
}