use sha1::{Digest, Sha1};
use base64::prelude::*;
pub(crate) fn compute_sec_websocket_accept(sec_ws_key: String) -> String {
    let mut res = sec_ws_key;
    res.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let res = Sha1::digest(res);
    BASE64_STANDARD.encode(res)
}