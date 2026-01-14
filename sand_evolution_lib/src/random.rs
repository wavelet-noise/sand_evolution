use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use js_sys::Math;

// Thread-local storage for deterministic RNG in tests
thread_local! {
    static DETERMINISTIC_RNG: RefCell<Option<Rc<RefCell<rand::rngs::StdRng>>>> = RefCell::new(None);
}

/// Set a deterministic RNG for testing purposes
#[cfg(not(target_arch = "wasm32"))]
pub fn set_deterministic_rng(rng: Rc<RefCell<rand::rngs::StdRng>>) {
    DETERMINISTIC_RNG.with(|cell| {
        *cell.borrow_mut() = Some(rng);
    });
}

/// Clear deterministic RNG (restore to non-deterministic mode)
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_deterministic_rng() {
    DETERMINISTIC_RNG.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

#[cfg(target_arch = "wasm32")]
pub fn my_rand() -> i64 {
    (Math::random() * 10000.0) as i64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn my_rand() -> i64 {
    DETERMINISTIC_RNG.with(|cell| {
        if let Some(rng_rc) = cell.borrow().as_ref() {
            // Use deterministic RNG if set
            rng_rc.borrow_mut().gen_range(0..10000)
        } else {
            // Use thread-local RNG (non-deterministic)
            let mut rng = rand::thread_rng();
            rng.gen_range(0..10000) // Generating a random number between 0 and 9999
        }
    })
}
