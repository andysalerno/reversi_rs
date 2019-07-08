use rand::seq::SliceRandom;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::FromEntropy;

pub fn random_pick<'a, T, R>(choices: &'a[T], rng: &mut R) -> Option<&'a T> 
where R: Rng
{
    choices.choose(rng)
}

pub fn random_choice<T, R>(choices: &[T], rng: &mut R) -> T
where
    T: Copy,
    R: Rng,
{
    *random_pick(choices, rng).expect("Attempted to pick a random choice, but failed")
}

pub(crate) fn get_rng() -> impl rand::Rng {
    SmallRng::from_entropy()
}

pub(crate) fn get_rng_deterministic() -> impl rand::Rng {
    use rand::SeedableRng;
    SmallRng::from_seed([0; 16])
}