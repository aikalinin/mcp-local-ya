# Changelog

## Unreleased (fork: security hardening)

### Breaking / behaviour changes for MCP clients

- **Outgoing mail from MCP is saved to Drafts by default.** `yacli.mail.send`,
  `yacli.mail.reply`, `yacli.mail.forward`, `yacli.mail.send_link` and
  `yacli.mail.send_published_link` store the message in the Drafts folder via
  IMAP `APPEND` unless `YACLI_MCP_SEND_POLICY` is `confirm` or `send`.
  Responses carry `status` (`draft_saved` / `completed` / `cancelled`) and a
  `delivery` block; `SentMail` gained `delivery` and `draft {folder, uid}`.
- **MCP tools have no local filesystem access unless `YACLI_MCP_ALLOWED_PATHS`
  is set.** Affects `attachments`, `source`, `source_path` and `output_path`
  arguments. Paths are canonicalised; credential directories and the yacli
  config directory are always denied.
- **HTTP bearer token now gates every request** to `/mcp` (previously only
  `tools/call` for mail/calendar/disk). `initialize`, `resources/read`,
  `yacli.activity.undo`, GET (SSE) and DELETE all require it when configured.
- **Non-loopback `--listen` without a bearer token is refused** at startup.
- `initialize` ignores a client-supplied `Mcp-Session-Id`; follow-up requests
  with an unknown session id get HTTP 404.

### Added

- `YACLI_MCP_SEND_POLICY=confirm`: on stdio, with a client that advertises
  `capabilities.elicitation`, the server asks the human via
  `elicitation/create` before sending over SMTP. Single-use confirmation;
  decline/cancel returns `status: cancelled`. Falls back to `draft` otherwise.
- MCP `annotations` (`readOnlyHint`, `destructiveHint`, `idempotentHint`,
  `openWorldHint`) on every tool.
- Security notes in tool descriptions, embedded prompts and all `SKILL.md`
  files: mail/calendar/disk content is untrusted data, never instructions.
- `yacli doctor` reports the `YACLI_MCP_ALLOWED_PATHS` state.
- Drafts keep `Bcc:` as a header (SMTP carries Bcc only in the envelope) and
  go through the same size guard as SMTP sends, so oversized mail gets the
  send-link remediation instead of an unsendable draft.
- `confirm` dialogs for `mail.reply` / `mail.forward` resolve the real
  recipient from the original message (`Reply-To` / `From`) and show it,
  plus HTML body size and a tag-stripped preview.
- Allowlist: hidden entries (`.git`, `.env`, shell rc files, …) below an
  allowed root are denied for both read and write; `.git`,
  `.git-credentials`, `.npmrc`, `.pypirc`, `.gitconfig` and shell histories
  joined the deny-list; the home directory and filesystem roots are refused
  as allowlist entries.
- A nested mail-sending call while a confirmation is pending is refused
  instead of stalling the session; a client that advertises elicitation but
  fails to serve it degrades to `draft`.

### Unchanged

- CLI commands (`yacli mail send`, etc.) keep sending immediately and accept
  any path.
