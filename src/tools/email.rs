use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::EmailConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSummary {
    pub uid: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailDetail {
    pub uid: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub date: String,
    pub body: String,
}

/// Fetch latest emails via IMAP.
pub async fn fetch_inbox(config: &EmailConfig, count: u32) -> Result<Vec<EmailSummary>> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .context("TLS init failed")?;

    let client =
        imap::connect((&*config.imap_host, config.imap_port), &config.imap_host, &tls)
            .context("IMAP connect failed")?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| anyhow::anyhow!("IMAP login failed: {}", e.0))?;

    session.select("INBOX").context("Failed to select INBOX")?;

    // Search for recent messages
    let search = session.search("ALL").context("IMAP search failed")?;
    let mut uids: Vec<u32> = search.into_iter().collect();
    uids.sort_unstable();
    uids.reverse();
    uids.truncate(count as usize);

    if uids.is_empty() {
        return Ok(Vec::new());
    }

    let uid_range = uids
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let messages = session
        .fetch(&uid_range, "ENVELOPE BODY.PEEK[TEXT]<0.200>")
        .context("IMAP fetch failed")?;

    let mut results = Vec::new();
    for msg in messages.iter() {
        let envelope = match msg.envelope() {
            Some(e) => e,
            None => continue,
        };

        let from = envelope
            .from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .map(|a| {
                let name = a
                    .name
                    .as_ref()
                    .map(|n| String::from_utf8_lossy(n).to_string())
                    .unwrap_or_default();
                let mailbox = a
                    .mailbox
                    .as_ref()
                    .map(|m| String::from_utf8_lossy(m).to_string())
                    .unwrap_or_default();
                let host = a
                    .host
                    .as_ref()
                    .map(|h| String::from_utf8_lossy(h).to_string())
                    .unwrap_or_default();
                if name.is_empty() {
                    format!("{mailbox}@{host}")
                } else {
                    format!("{name} <{mailbox}@{host}>")
                }
            })
            .unwrap_or_else(|| "(unknown)".to_string());

        let subject = envelope
            .subject
            .as_ref()
            .map(|s| String::from_utf8_lossy(s).to_string())
            .unwrap_or_else(|| "(no subject)".to_string());

        let date = envelope
            .date
            .as_ref()
            .map(|d| String::from_utf8_lossy(d).to_string())
            .unwrap_or_default();

        let snippet = msg
            .text()
            .map(|t| {
                let text = String::from_utf8_lossy(t);
                text.chars().take(200).collect::<String>()
            })
            .unwrap_or_default();

        results.push(EmailSummary {
            uid: msg.message.to_string(),
            from,
            subject,
            date,
            snippet,
        });
    }

    session.logout().ok();
    info!(count = results.len(), "Fetched emails");

    Ok(results)
}

/// Read a full email by sequence number.
pub async fn read_email(config: &EmailConfig, uid: &str) -> Result<EmailDetail> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .context("TLS init failed")?;

    let client =
        imap::connect((&*config.imap_host, config.imap_port), &config.imap_host, &tls)
            .context("IMAP connect failed")?;

    let mut session = client
        .login(&config.username, &config.password)
        .map_err(|e| anyhow::anyhow!("IMAP login failed: {}", e.0))?;

    session.select("INBOX").context("Failed to select INBOX")?;

    let messages = session
        .fetch(uid, "ENVELOPE BODY[TEXT]")
        .context("IMAP fetch failed")?;

    let msg = messages.iter().next().context("Email not found")?;

    let envelope = msg.envelope().context("No envelope")?;

    let from = envelope
        .from
        .as_ref()
        .and_then(|a| a.first())
        .map(|a| {
            let mb = a.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
            let host = a.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
            format!("{mb}@{host}")
        })
        .unwrap_or_default();

    let to = envelope
        .to
        .as_ref()
        .and_then(|a| a.first())
        .map(|a| {
            let mb = a.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
            let host = a.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
            format!("{mb}@{host}")
        })
        .unwrap_or_default();

    let subject = envelope.subject.as_ref()
        .map(|s| String::from_utf8_lossy(s).to_string())
        .unwrap_or_default();

    let date = envelope.date.as_ref()
        .map(|d| String::from_utf8_lossy(d).to_string())
        .unwrap_or_default();

    let body = msg
        .text()
        .map(|t| String::from_utf8_lossy(t).to_string())
        .unwrap_or_else(|| "(empty body)".to_string());

    session.logout().ok();

    Ok(EmailDetail {
        uid: uid.to_string(),
        from,
        to,
        subject,
        date,
        body,
    })
}

/// Send an email via SMTP.
pub async fn send_email(
    config: &EmailConfig,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<String> {
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let email = Message::builder()
        .from(config.username.parse().context("Invalid from address")?)
        .to(to.parse().context("Invalid to address")?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .context("Failed to build email")?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let mailer = SmtpTransport::starttls_relay(&config.smtp_host)
        .context("SMTP connect failed")?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    mailer.send(&email).context("Failed to send email")?;

    info!(%to, %subject, "Email sent");
    Ok(format!("Email sent to {to}: {subject}"))
}

/// Format email summaries for Discord notification.
pub fn format_inbox_report(emails: &[EmailSummary]) -> String {
    if emails.is_empty() {
        return "No new emails.".to_string();
    }

    let mut out = format!("**Inbox ({} emails)**\n", emails.len());
    for e in emails.iter().take(10) {
        out.push_str(&format!(
            "- **{}** from {}\n  {}\n",
            e.subject,
            e.from,
            if e.snippet.len() > 100 {
                format!("{}...", &e.snippet[..100])
            } else {
                e.snippet.clone()
            }
        ));
    }
    out
}
