mod utils;
extern crate js_sys;

use std::cmp::{max, min};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
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

macro_rules! console_error {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
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
    pos: Trace,
    id: u32,
    timestamp: u32, // last activity timestamp
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct DeviceData {
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
    devices: Vec<DeviceData>,
}

//
// Define private structures
//

//
// Implement private traits
//


fn array_insert_pop<T>(arr: &mut [T; MEASURE_DEPTH], new_val: T) -> &[T; MEASURE_DEPTH] where T: Copy {
    for i in 1..MEASURE_DEPTH {
        arr[i] = arr[i-1];
    }
    arr[0] = new_val;
    arr
}

impl MeasureList {
    pub fn new(meas: Measure) -> MeasureList {
        let lo = min(meas.id[0], meas.id[1]);
        let hi = max(meas.id[0], meas.id[1]);
        MeasureList {
            dev: [lo, hi],
            measures_ts: [meas.timestamp; MEASURE_DEPTH],
            measures_val: [meas.distance; MEASURE_DEPTH],
        }
    }

    pub fn update(&mut self, meas: Measure) {
        array_insert_pop(&mut self.measures_val, meas.distance);
        array_insert_pop(&mut self.measures_ts, meas.timestamp);
    }

    pub fn estimate(&self, _timestamp: u32) -> f32 {
        *self.measures_val.last().unwrap()
    }
}

impl DeviceData {
    pub fn new(id: u32) -> DeviceData {
        DeviceData::new_with_pos(id, 0, 0, 0)
    }

    pub fn new_with_pos(id: u32, x: i32, y: i32, z: i32) -> DeviceData {
        let pos = Trace {
                timestamp: 0,
                cord: [x as f32, y as f32, z as f32],
            };
        let mut dev = DeviceData {
            id: id,
            timestamp: 0,
            trace: VecDeque::with_capacity(POSITION_TRACE_DEPTH),
        };
        dev.trace.push_back(pos);
        dev
    }
}

impl Zone {
    pub fn _get_device(&mut self, id: u32) -> &mut DeviceData {
        let dev = self.devices.iter_mut().find(|x| x.id == id);
        dev.unwrap()
    }
}

#[wasm_bindgen]
impl Zone {
    pub fn add_device(&mut self, id: u32, x: i32, y: i32, z: i32) {
        let count = self.devices.iter().filter(|x| x.id == id).count();
        assert_eq!(count, 0);
        let dev = DeviceData::new_with_pos(id, x, y, z);
        self.devices.push(dev);
        console_log!("New device {} at position {}, {}, {}", id, x, y, z);
    }

    pub fn add_measure(&mut self, id1: u32, id2: u32, distance: f32, timestamp: u32) {
        let lo = min(id1, id2);
        let hi = max(id1, id2);
        self.devices.iter_mut()
            .filter(|x| x.id == lo || x.id == hi)
            .for_each(|x| x.timestamp = timestamp);
        let ml = self.measures.iter_mut()
            .find(|x| x.dev[0] == lo && x.dev[1] == hi);
        let meas = Measure{
            distance: distance,
            id: [lo, hi],
            timestamp: timestamp,
        };
        match ml {
            Some(l) => {
                console_log!("Measure update {}-{} {}!", lo, hi, meas.distance);
                l.update(meas);
            },
            None => {
                console_log!("New connection {}-{} {}!", lo, hi, meas.distance);
                let new_ml = MeasureList::new(meas);
                self.measures.push(new_ml);
            },
        }
    }

    pub fn get_device_ptr(&self, id: u32) -> *const DeviceData {
        let dev = self.devices.iter().find(|x| x.id == id);
        dev.unwrap()
    }

    pub fn calculate_device_position(&mut self, id: u32, timestamp: u32) -> JsValue {
        let connected_devices_id: Vec<u32> = self.measures.iter()
            .filter(|&x| (x.dev[0] == id || x.dev[1] == id))
            .map(|x| if x.dev[0] == id { x.dev[1] } else { x.dev[0] } )
            .collect();
        let connected_devices: Vec<&Device> = self.devices.iter()
            .filter(|&x| connected_devices_id.iter().any(|&v| v == x.id))
            .collect();
        assert_ne!(connected_devices.len(), 0);
        let dev = self._get_device(id);
        while dev.trace.len() + 1 >= POSITION_TRACE_DEPTH {
            dev.trace.pop_back();
        }
        // fake calculation
        let mut last_pos = dev.trace.front().unwrap().clone();
        last_pos.cord[1] += 0.5;
        if last_pos.cord[1] > 500.0 {
            last_pos.cord[1] = 0.0;
        }
        last_pos.timestamp = timestamp;
        //todo: Add position calculation
        dev.trace.push_front(last_pos);
        JsValue::from_serde(&last_pos).unwrap()
    }
}

//
// Implement WASM functions
//
#[wasm_bindgen]
pub fn device_serialize(dev: *const DeviceData) -> JsValue {
    unsafe {
        let d: &DeviceData = &*dev;
        JsValue::from_serde(&*d).unwrap()
    }
}

#[wasm_bindgen]
pub fn init() -> Zone {
    console_error_panic_hook::set_once();
    let mut zone = Zone {
        id: 0,
        measures:   Vec::new(),
        devices:    Vec::new(),
    };
    let test_device = DeviceData::new(0);
    zone.devices.push(test_device);
    zone
}

fn timestamp() -> u32 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_millis();
    since_the_epoch as u32
}
