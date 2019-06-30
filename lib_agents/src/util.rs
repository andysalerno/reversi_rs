use rand::seq::SliceRandom;

pub fn random_pick<'a, T>(choices: &'a[T]) -> Option<&'a T> {
    choices.choose(&mut rand::thread_rng())
}

pub fn random_choice<T>(choices: &[T]) -> T
where
    T: Copy,
{
    *random_pick(choices).expect("Attempted to pick a random choice on an empty slice.")
}
