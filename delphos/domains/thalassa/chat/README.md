# bkg-chat

**Chat rooms. Mailbox. Mentions. Direct + ambient responders.**

Chat is a first-class feature. Rooms have members, direct responders (always reply),
and ambient responders (reply only when mentioned).

## Key Types

| Type | Purpose |
|---|---|
| `ChatRoom` | `{ id, name, members, direct_responders, ambient_responders }` |
| `ChatMessage` | `{ sender, content, attachments, mentions }` |
| `Mailbox` | Per-user inbox (VecDeque) |
| `MailItem` | Delivered message with read tracking |

## Routing

```
Mention (@agent) → direct delivery to mentioned agent
No mention       → ambient responders only
```
