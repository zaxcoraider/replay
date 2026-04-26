# Day 8 — Account Mutator UI

## Goal

The other screenshot moment. Click any account in the inspector → see its decoded IDL state as an editable form → change a field → apply → see state update → "Re-run" button.

## Deliverables

1. `web/components/AccountInspector.tsx`:
   - Header: pubkey (with copy), owner program, size, lamports, rent-exempt status.
   - Tabs: "Decoded" (IDL tree-edit UI) | "Raw hex" (hex viewer) | "JSON" (raw JSON).
   - "Decoded" tab renders the IDL-decoded tree as a form. Primitives are editable inputs; structs are collapsible panels.

2. `web/components/FieldEditor.tsx` — recursive component handling every Borsh-IDL primitive:
   - Numbers: number input with `min`/`max` hints based on type (u8 → 0–255, etc.)
   - Pubkey: text input with base58 validation.
   - Bool: switch.
   - String: text input.
   - Array/Vec/Struct/Enum: expandable tree.
   - Every field has an "unchanged" indicator (greyed) vs "modified" indicator (highlighted).

3. Mutation application flow:
   - User edits fields locally (Zustand store holds pending mutations).
   - "Apply" button sends `POST /session/:id/mutate` with a `field` mutation.
   - On success, the inspector re-renders from the server's response.
   - "Discard" button reverts local edits.

4. Non-IDL fallback:
   - If account is not IDL-decodable, show the "Raw hex" tab as primary.
   - Splice-edit UI: select offset, paste hex bytes, apply as `raw_bytes` mutation. With bounds checking.

5. Lamports / owner mutation:
   - Two always-available fields at the top of the inspector. Edit → `lamports` or `owner` mutation type.

6. "Re-run with mutations" button — prominent, top-right.
   - Disabled if no mutations pending.
   - Shows a badge with mutation count.
   - On click: `POST /session/:id/execute`. On response: store new trace. Kick off the diff view (Day 9).

## UX gotchas

- Discriminator bytes (first 8 bytes of Anchor accounts) must NOT be editable. Show them but disable editing.
- When editing, show the byte offset and length of each field next to it. This teaches users Solana's layout semantics — they'll thank you.
- Warn when editing would drop lamports below rent-exempt minimum: "This will garbage-collect the account."
- Warn when editing changes a PDA-expected seed: "This account is a PDA. Mutating this field may break PDA verification in the program."

## Pre-canned mutation templates

Dropdown at the top: "Common mutations" — presets for popular programs.
- Whirlpool config: "Set fee_rate to X"
- Kamino reserve: "Set borrow limit to X"
- Any oracle account: "Set price to X"

This is worth it for the demo — judges see real utility fast.

## End-of-day

Third screen capture. Record: paste sig → replay → select Whirlpool config → change fee → apply → re-run → watch trace change. This recording is the raw material for the 2-min demo video.
