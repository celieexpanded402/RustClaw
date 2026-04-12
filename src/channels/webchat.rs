use tracing::info;

#[allow(dead_code)]
pub struct WebChatChannel;

#[allow(dead_code)]
impl WebChatChannel {
    pub fn new() -> Self {
        info!("WebChatChannel initialized (stub)");
        Self
    }
}
