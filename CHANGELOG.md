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

### Unchanged

- CLI commands (`yacli mail send`, etc.) keep sending immediately and accept
  any path.
