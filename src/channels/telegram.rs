use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::agent::{AgentRunner, Message as AgentMessage};
use crate::config::TelegramConfig;
use crate::session::memory::MemoryManager;

const MAX_RETRIES: u32 = 3;
const EDIT_INTERVAL: Duration = Duration::from_millis(800);

pub struct TelegramChannel {
    config: TelegramConfig,
    memory: MemoryManager,
}

impl TelegramChannel {
    pub fn new(config: TelegramConfig, memory: MemoryManager) -> Self {
        Self { config, memory }
    }

    pub async fn start(self, runner: Arc<AgentRunner>) -> Result<()> {
        if self.config.bot_token.is_empty() {
            anyhow::bail!("Telegram bot_token is empty");
        }

        let bot = Bot::new(&self.config.bot_token);

        // Validate token before starting dispatcher (avoids panic inside teloxide)
        info!("Validating Telegram bot token...");
        match bot.get_me().await {
            Ok(me) => info!(
                bot_name = %me.username(),
                "Telegram bot authenticated"
            ),
            Err(e) => {
                anyhow::bail!("Telegram bot token invalid: {e}");
            }
        }

        info!("Starting Telegram bot (long polling)");

        let config = Arc::new(self.config);
        let memory = self.memory;

        let handler = Update::filter_message()
            .filter(|msg: teloxide::types::Message| msg.text().is_some())
            .endpoint(handle_message);

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![runner, memory, config])
            .default_handler(|_upd| async {})
            .error_handler(LoggingErrorHandler::with_custom_text(
                "Telegram dispatcher error",
            ))
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

async fn handle_message(
    bot: Bot,
    msg: teloxide::types::Message,
    runner: Arc<AgentRunner>,
    memory: MemoryManager,
    config: Arc<TelegramConfig>,
) -> ResponseResult<()> {
    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };

    let chat_id = msg.chat.id;
    let user_id = msg.from.as_ref().map(|u| u.id.0);

    // ACL check
    if !config.allowed_user_ids.is_empty() {
        match user_id {
            Some(uid) if config.allowed_user_ids.contains(&uid) => {}
            _ => {
                info!(chat_id = %chat_id, user_id = ?user_id, "Rejected: user not in allowed list");
                return Ok(());
            }
        }
    }

    let session_id = format!("telegram:{}", chat_id);
    memory.get_or_create(&session_id).await;

    let history = memory.get_history(&session_id).await;
    let recalled = memory.recall(&session_id, text).await;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    let mut context_parts = Vec::new();
    if history.is_empty() {
        context_parts.push(format!("[Current time: {now}]"));
    }
    if !recalled.is_empty() {
        context_parts.push(format!("[Memory]\n{recalled}"));
    }
    let input_with_context = if context_parts.is_empty() {
        text.to_string()
    } else {
        context_parts.push(text.to_string());
        context_parts.join("\n\n")
    };

    // Record user message
    memory
        .push_message(
            &session_id,
            AgentMessage {
                role: "user".to_string(),
                content: text.to_string(),
            },
        )
        .await;

    if config.stream_edit {
        stream_with_edit(&bot, chat_id, &runner, &input_with_context, &history, &memory, &session_id).await?;
    } else {
        send_oneshot(&bot, chat_id, &runner, &input_with_context, &history, &memory, &session_id).await?;
    }

    Ok(())
}

/// Streaming mode: send placeholder, then edit with accumulated tokens.
async fn stream_with_edit(
    bot: &Bot,
    chat_id: ChatId,
    runner: &AgentRunner,
    input: &str,
    history: &[AgentMessage],
    memory: &MemoryManager,
    session_id: &str,
) -> ResponseResult<()> {
    // Send initial placeholder
    let placeholder = retry_send(bot, chat_id, "▍").await?;
    let msg_id = placeholder.id;

    // Start streaming
    let (tx, mut rx) = mpsc::channel::<String>(64);
    let input_owned = input.to_string();
    let history_owned = history.to_vec();
    let runner_config = runner.config().clone();

    tokio::spawn(async move {
        let r = AgentRunner::new(runner_config);
        let tx2 = tx.clone();
        let result = r
            .run_streaming(&input_owned, &history_owned, move |token| {
                let _ = tx2.try_send(token);
            })
            .await;
        if let Err(e) = result {
            error!(%e, "Agent streaming error in Telegram handler");
        }
    });

    let mut full = String::new();
    let mut last_edit = tokio::time::Instant::now();

    loop {
        tokio::select! {
            token = rx.recv() => {
                match token {
                    Some(t) => {
                        full.push_str(&t);
                        // Throttle edits
                        if last_edit.elapsed() >= EDIT_INTERVAL {
                            let display = format!("{full}▍");
                            retry_edit(bot, chat_id, msg_id, &display).await;
                            last_edit = tokio::time::Instant::now();
                        }
                    }
                    None => break, // stream done
                }
            }
        }
    }

    // Final edit with complete text
    if full.is_empty() {
        full.push_str("(no response)");
    }
    retry_edit(bot, chat_id, msg_id, &full).await;

    // Record assistant message + learn long-term memories
    memory
        .push_message(
            session_id,
            AgentMessage {
                role: "assistant".to_string(),
                content: full.clone(),
            },
        )
        .await;
    memory.learn(session_id, input, &full).await;

    Ok(())
}

/// Non-streaming mode: wait for full response, send once.
async fn send_oneshot(
    bot: &Bot,
    chat_id: ChatId,
    runner: &AgentRunner,
    input: &str,
    history: &[AgentMessage],
    memory: &MemoryManager,
    session_id: &str,
) -> ResponseResult<()> {
    let result = runner
        .run_streaming(input, history, |_| {})
        .await;

    let response = match result {
        Ok(text) if !text.is_empty() => text,
        Ok(_) => "(no response)".to_string(),
        Err(e) => {
            error!(%e, "Agent error");
            format!("Error: {e}")
        }
    };

    retry_send(bot, chat_id, &response).await?;

    memory
        .push_message(
            session_id,
            AgentMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        )
        .await;
    memory.learn(session_id, input, &response).await;

    Ok(())
}

// ── Telegram API helpers with retry ──────────────────────────────────

async fn retry_send(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
) -> ResponseResult<teloxide::types::Message> {
    let mut last_err = None;
    for attempt in 0..MAX_RETRIES {
        match bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .await
        {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                warn!(attempt, %e, "send_message failed, retrying");
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt + 1))).await;
            }
        }
    }
    Err(last_err.unwrap())
}

async fn retry_edit(bot: &Bot, chat_id: ChatId, msg_id: MessageId, text: &str) {
    for attempt in 0..MAX_RETRIES {
        match bot.edit_message_text(chat_id, msg_id, text).await {
            Ok(_) => return,
            Err(e) => {
                // "message is not modified" is benign — skip retry
                let err_str = e.to_string();
                if err_str.contains("message is not modified") {
                    return;
                }
                warn!(attempt, %e, "edit_message_text failed, retrying");
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt + 1))).await;
            }
        }
    }
}
