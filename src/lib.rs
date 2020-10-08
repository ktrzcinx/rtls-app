mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  // Use `js_namespace` here to bind `console.log(..)` instead of just
  // `log(..)`
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);

  #[wasm_bindgen(js_namespace = console)]
  pub fn error(s: &str);

  #[wasm_bindgen(js_namespace = console)]
  pub fn time(s: &str);

  #[wasm_bindgen(js_namespace = console)]
  pub fn timeEnd(s: &str);

  #[wasm_bindgen(js_namespace = console)]
  pub fn timeLog(s: &str, v: &str);

  pub fn alert(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, rtls!");
}
