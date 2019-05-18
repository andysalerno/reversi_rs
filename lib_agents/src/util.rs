use rand::seq::SliceRandom;

pub fn random_pick<'a, T>(choices: &'a[T]) -> &'a T {
    choices.choose(&mut rand::thread_rng()).unwrap()
}

pub fn random_choice<T>(choices: &[T]) -> T
where
    T: Copy,
{
    *random_pick(choices)
}
