use rand::seq::SliceRandom;
use rand::Rng;

pub fn random_pick<'a, T, R>(choices: &'a[T], rng: &mut R) -> Option<&'a T> 
where R: Rng + Sized
{
    choices.choose(rng)
}

pub fn random_choice<T>(choices: &[T]) -> T
where
    T: Copy,
{
    *random_pick(choices, &mut rand::thread_rng()).expect("Attempted to pick a random choice on an empty slice.")
}
