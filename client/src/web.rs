use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use wasm_bindgen_futures::spawn_local;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::ClientOptions;

/// Client for the web platform.
//#[wasm_bindgen]
pub struct WebClient {
    options: ClientOptions,
    ws: WebSocket,
}

//#[wasm_bindgen]
impl WebClient {
    /// Create a new web client.
    //#[wasm_bindgen(constructor)]

    pub async fn new(
        server: &str,
        options: ClientOptions,
    ) -> Result<WebClient, JsValue> {
        let ws = WebSocket::new(server)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let onmessage_callback =
            Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {

        });

        // set message event handler on WebSocket
        ws.set_onmessage(Some(
            onmessage_callback.as_ref().unchecked_ref(),
        ));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        let onerror_callback =
            Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                log::error!("error event: {:?}", e);
            });
        ws.set_onerror(Some(
            onerror_callback.as_ref().unchecked_ref(),
        ));
        onerror_callback.forget();

        let (open_tx, mut open_rx) = mpsc::channel(1);

        let cloned_ws = ws.clone();
        let onopen_callback =
            Closure::once(move || {
                log::info!("websocket got open event");
                spawn_local(async move {
                    open_tx.send(()).await.unwrap();
                });
                /*
                log::info!("socket opened");
                // send off binary message
                match cloned_ws.send_with_u8_array(&[0, 1, 2, 3]) {
                    Ok(_) => {
                        log::info!("binary message successfully sent")
                    }
                    Err(err) => {
                        log::info!("error sending message: {:?}", err)
                    }
                }
                */
            });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let _ = open_rx.recv().await;
        drop(open_rx);

        log::info!("socket opened, returning the client");

        let client = WebClient { options, ws };
        Ok(client)
    }
}
