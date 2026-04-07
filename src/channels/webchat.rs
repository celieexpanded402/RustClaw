use tracing::info;

/// WebChat channel stub.
pub struct WebChatChannel;

impl WebChatChannel {
    pub fn new() -> Self {
        info!("WebChatChannel initialized (stub)");
        Self
    }
}
