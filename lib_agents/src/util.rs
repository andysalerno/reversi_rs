use rand::seq::SliceRandom;
use rand::Rng;

pub fn random_pick<'a, T, R>(choices: &'a[T], rng: &mut R) -> Option<&'a T> 
where R: rand_core::RngCore
{
    let l = choices.len() as u32;

    if l == 0 {
        return None;
    }

    let n = rng.next_u32() % l;

    return Some(&choices[n as usize])
}

pub fn random_choice<T, R>(choices: &[T], rng: &mut R) -> T
where
    T: Copy,
    R: rand_core::RngCore,
{
    *random_pick(choices, rng).expect("Attempted to pick a random choice, but failed")
}

pub(crate) fn weak_rng() -> rand_xorshift::XorShiftRng {
    use rand_core::SeedableRng;
    use rand::RngCore;
    use rand_xorshift::XorShiftRng;

    let mut t_rng = rand::thread_rng();
    let mut pure_rng_seed = [0; 16];
    t_rng.fill_bytes(&mut pure_rng_seed);

    let mut rnn = XorShiftRng::from_seed(pure_rng_seed);

    rnn
}