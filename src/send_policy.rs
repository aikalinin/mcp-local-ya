//! Outgoing-mail policy for MCP tools.
//!
//! An AI agent driving `yamcli.mail.send` / `reply` / `forward` / `send_link` /
//! `send_published_link` can be steered by untrusted content it just read (a
//! malicious e-mail, a document on Disk). The server therefore decides how an
//! outgoing message leaves the account instead of trusting the agent:
//!
//! - `draft` (default): the message is stored in the Drafts folder via IMAP;
//!   a human opens it in the mail client and presses send. Works with every
//!   MCP host.
//! - `confirm`: the server asks the human through MCP `elicitation/create`
//!   before submitting over SMTP. Only possible when the client advertises the
//!   `elicitation` capability on a transport that can carry server-initiated
//!   requests (stdio). Otherwise it degrades to `draft`, never to `send`.
//! - `send`: submit over SMTP immediately (pre-hardening behaviour).
//!
//! The CLI is unaffected: a human typing `yamcli mail send` already decided.

use std::env;

use crate::error::{Result, YamcliError};

pub const SEND_POLICY_ENV: &str = "YAMCLI_MCP_SEND_POLICY";

pub const MAIL_SENDING_TOOLS: &[&str] = &[
    "yamcli.mail.send",
    "yamcli.mail.reply",
    "yamcli.mail.forward",
    "yamcli.mail.send_link",
    "yamcli.mail.send_published_link",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendPolicy {
    Draft,
    Confirm,
    Send,
}

impl SendPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Confirm => "confirm",
            Self::Send => "send",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" | "drafts" => Some(Self::Draft),
            "confirm" | "ask" => Some(Self::Confirm),
            "send" | "smtp" => Some(Self::Send),
            _ => None,
        }
    }
}

pub fn configured_send_policy() -> Result<SendPolicy> {
    match env::var(SEND_POLICY_ENV) {
        Err(_) => Ok(SendPolicy::Draft),
        Ok(raw) if raw.trim().is_empty() => Ok(SendPolicy::Draft),
        Ok(raw) => SendPolicy::parse(&raw).ok_or_else(|| {
            YamcliError::Config(format!(
                "unsupported {SEND_POLICY_ENV} `{raw}`; expected `draft`, `confirm` or `send`"
            ))
        }),
    }
}

pub fn is_mail_sending_tool(tool_name: &str) -> bool {
    MAIL_SENDING_TOOLS.contains(&tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_policies_case_insensitively() {
        assert_eq!(SendPolicy::parse("Draft"), Some(SendPolicy::Draft));
        assert_eq!(SendPolicy::parse(" confirm "), Some(SendPolicy::Confirm));
        assert_eq!(SendPolicy::parse("SEND"), Some(SendPolicy::Send));
        assert_eq!(SendPolicy::parse("yolo"), None);
    }

    #[test]
    fn recognises_every_tool_that_can_send_mail() {
        for tool in [
            "yamcli.mail.send",
            "yamcli.mail.reply",
            "yamcli.mail.forward",
            "yamcli.mail.send_link",
            "yamcli.mail.send_published_link",
        ] {
            assert!(is_mail_sending_tool(tool), "{tool}");
        }
        assert!(!is_mail_sending_tool("yamcli.mail.read"));
        assert!(!is_mail_sending_tool("yamcli.disk.publish"));
    }
}
