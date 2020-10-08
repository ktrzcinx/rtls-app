mod utils;

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

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

//
// Define WASM structures
//

const MEASURE_DEPTH : usize = 5;
const POSITION_TRACE_DEPTH : usize = 3;

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone)]
pub struct Trace {
    cord: [f32; 3],
    timestamp: u32,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Device {
    trace: VecDeque<Trace>,
    id: u32,
    timestamp: u32, // last activity timestamp
}

#[wasm_bindgen]
pub struct Measure {
    id: [u32; 2],
    timestamp: u32, // time can't be passed as float, because of rounding effect
    distance: f32,
}

#[wasm_bindgen]
pub struct MeasureList {
    dev: [u32; 2],
    measures_ts: [u32; MEASURE_DEPTH],
    measures_val: [f32; MEASURE_DEPTH],
}

#[wasm_bindgen]
pub struct Zone {
    id: i32,
    measures: Vec<MeasureList>,
    devices: Vec<Device>,
}

//
// Define private structures
//

