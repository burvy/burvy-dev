use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    ReadableStreamDefaultReader, WebTransport, WebTransportBidirectionalStream,
    WebTransportReceiveStream,
};

const SERVER: &str = "https://webtrans.burvy.dev:4433";

#[derive(Clone)]
pub struct WebTrans {
    transport: WebTransport,
}


pub async fn connect(
    room: &str,
    msg_fn: impl Fn(String) + 'static,
) -> Result<WebTrans, JsValue> {
    // this should just be that server then room should have a / on it
    // like SERVER:4433/chat instead of like ...4433chat
    // TODO: verify this
    let transport = WebTransport::new(&format!("{}{}", SERVER, room))?;
    todo!()

}


impl WebTrans {
    pub async fn send() {}
}
