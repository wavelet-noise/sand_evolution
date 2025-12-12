use rand::Rng;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}
#[cfg(feature = "wasm")]
pub fn my_rand() -> i64 {
    (random() * 10000.0) as i64
}

#[cfg(not(feature = "wasm"))]
pub fn my_rand() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..10000) // Generating a random number between 0 and 9999
}
