use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    ReadableStreamDefaultReader, WebTransport, WebTransportBidirectionalStream,
    WebTransportReceiveStream,
};

/// the server address
/// server handles all the encryption and stuff already
/// note that webtransport only wants to work through an HTTPS connection
const SERVER: &str = "https://webtrans.burvy.dev:4433";


#[derive(Clone)]
pub struct WebTrans {
    transport: WebTransport,
}


pub async fn connect(
    room: &str,
    msg_fn: impl Fn(String) + 'static,
) -> Result<WebTrans, JsValue> {
    let transport = WebTransport::new(&format!("{}/{}", SERVER, room.trim_start_matches("/")))?;
    JsFuture::from(transport.ready()).await?; // wait for the room to be ready
    spawn_local(recv_loop(transport.clone(), msg_fn));
    Ok(WebTrans::new(transport))
}


impl WebTrans {
    /// this is NOT WebTransport::new(), TWO DIFFERENT THINGS!!!!!!
    pub fn new(transport: WebTransport) -> Self {
        Self { transport }
    }


    pub async fn send(&self, msg: &str) -> Result<(), JsValue> {
        // opens a bidirectional stream for each message
        let stream = JsFuture::from(self.transport.create_bidirectional_stream())
            .await?
            .unchecked_into::<WebTransportBidirectionalStream>();
        // we use the same text as bytes protocol as the server
        let writer = stream.writable().get_writer()?;
        let bytes: JsValue = Uint8Array::from(msg.as_bytes()).into();
        // write it to the stream and close
        // if we dont close then the server keeps waiting
        JsFuture::from(writer.write_with_chunk(&bytes)).await?;
        JsFuture::from(writer.close()).await?;
        writer.release_lock();
        // success
        Ok(())
    }
    pub fn close(&self) {
        self.transport.close();
    }
}

/// receives info from the server
async fn recv_loop(transport: WebTransport, msg_fn: impl Fn(String) + 'static) {
    // server sends back unidir streams and we must accept as such
    let reader = transport
        .incoming_unidirectional_streams()
        .get_reader()
        .unchecked_into::<ReadableStreamDefaultReader>();

    while let Ok(Some(stream)) = read_next(&reader).await {
        let stream = stream.unchecked_into::<WebTransportReceiveStream>();
        // if we read the message successfully
        if let Ok(msg) = read_string(stream).await {
            // handle the message with the function passed in
            msg_fn(msg);
        }
    }
    reader.release_lock();
}

/// reads the next chunk from the reader by using js magic and returning none when done
async fn read_next(reader: &ReadableStreamDefaultReader) -> Result<Option<JsValue>, JsValue> {
    let result = JsFuture::from(reader.read()).await?;
    let done = Reflect::get(&result, &"done".into())?
        .as_bool()
        .unwrap_or(false);
    if done {
        return Ok(None);
    }
    Ok(Some(Reflect::get(&result, &"value".into())?))
}
/// reads a string from the stream by reading chunks until done
async fn read_string(stream: WebTransportReceiveStream) -> Result<String, JsValue> {
    let reader = stream.get_reader().unchecked_into::<ReadableStreamDefaultReader>();
    let mut bytes = Vec::new();
    // while there is still chunk ahead
    while let Ok(Some(chunk)) = read_next(&reader).await {
        bytes.extend_from_slice(&Uint8Array::new(&chunk).to_vec());
    }
    // conglomerate all those bytes from the chunks into a string and have errors converted to JsValue
    String::from_utf8(bytes).map_err(|e| JsValue::from_str(&e.to_string()))
}
