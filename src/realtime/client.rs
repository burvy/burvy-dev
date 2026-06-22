use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    ReadableStream, ReadableStreamDefaultReader
};


pub async fn connect() {}

pub struct Realtime {}
impl Realtime {
    pub async fn send() {}
}
