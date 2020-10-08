mod utils;
extern crate js_sys;

use std::cmp::{max, min};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (nodejs_helper::console::log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (nodejs_helper::console::error(&format_args!($($t)*).to_string()))
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

    pub fn calc_position(&self, measures: &Vec<&MeasureList>,
            devices: &Vec<&DeviceData>, timestamp: u32) -> Trace
    {
        // new position calculation or extrapolation when calculation is fresher than measurements
        // fake calculation or estimation when calculation is fresher than measurements
        //todo: Add position calculation
        let mut new_pos = self.trace.front().unwrap().clone();
        new_pos.cord[0] += 0.35;
        new_pos.cord[1] += 0.55;
        if new_pos.cord[0] > 700.0 {
            new_pos.cord[0] = 0.0;
        }
        if new_pos.cord[1] > 700.0 {
            new_pos.cord[1] = 0.0;
        }
        new_pos.timestamp = timestamp;
        new_pos
    }

    pub fn estimate_position(&self, timestamp: u32) -> Trace
    {
        let mut pos = *self.trace.front().unwrap();
        pos.timestamp = timestamp;
        pos
    }

    pub fn save_position(&mut self, pos: Trace)
    {
        // limit trace length
        while self.trace.len() + 1 >= POSITION_TRACE_DEPTH {
            self.trace.pop_back();
        }
        self.trace.push_front(pos);
    }
}

impl Device {
    pub fn new(dev: &DeviceData, timestamp: u32) -> Device {
        Device {
            id: dev.id,
            pos: dev.estimate_position(timestamp),
            timestamp: timestamp,
        }
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

    fn calc_dev_position(&self, dev: &DeviceData, timestamp: u32) -> Trace
    {
        let measures: Vec<&MeasureList> = self.measures.iter()
            .filter(|&x| (x.dev[0] == dev.id || x.dev[1] == dev.id))
            .collect();
        let connected_devices_id: Vec<u32> = measures.iter()
            .map(|x| if x.dev[0] == dev.id { x.dev[1] } else { x.dev[0] } )
            .collect();
        let devices: Vec<&DeviceData> = self.devices.iter()
            .filter(|&x| connected_devices_id.iter().any(|&v| v == x.id))
            .collect();
        let pos = dev.calc_position(&measures, &devices, timestamp);
        pos
    }

    fn update_dev_position(&mut self, id: u32, timestamp: u32)
    {
        let dev_index = self.devices.iter()
            .position(|x| x.id == id)
            .unwrap();
        let pos = self.calc_dev_position(&self.devices[dev_index], timestamp);
        self.devices[dev_index].save_position(pos);
    }

    pub fn add_measure(&mut self, id1: u32, id2: u32, distance: f32, timestamp: u32) {
        let id = [min(id1, id2), max(id1, id2)];
        let ml = self.measures.iter_mut()
            .find(|x| x.dev[0] == id[0] && x.dev[1] == id[1]);
        let meas = Measure{
            distance: distance,
            id: [id[0], id[1]],
            timestamp: timestamp,
        };
        match ml {
            Some(l) => {
                l.update(meas);
                for &i in id.iter() {
                    self.update_dev_position(i, timestamp);
                }
            },
            None => {
                console_log!("New connection {}-{} {}!", id[0], id[1], meas.distance);
                let new_ml = MeasureList::new(meas);
                self.measures.push(new_ml);
            },
        }
    }

    pub fn get_device_ptr(&self, id: u32) -> *const DeviceData {
        let dev = self.devices.iter().find(|x| x.id == id);
        dev.unwrap()
    }

    pub fn get_all_devices_position(&mut self, timestamp: u32) -> JsValue {
        let mut pos: Vec<Device> = Vec::new();
        pos.reserve(self.devices.len());
        for dev in self.devices.iter() {
            pos.push(Device::new(dev, timestamp));
        }
        JsValue::from_serde(&pos).unwrap()
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
