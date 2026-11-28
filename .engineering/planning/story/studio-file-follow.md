---
format: aep.planning-md/3
id: story:studio-file-follow
kind: story
status: draft
title: Studio drafts follow their files on disk
summary: 'A change on disk to a draft''s file becomes a revision: the difference as a patch, applied by a tool call with caller File and published on the draft''s channel.'
relations:
- decomposes: epic:agenticstudio
- depends_on: story:studio-live
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/agenticstudio
- confidence: inferred
  path: docs/studio.md
- confidence: inferred
  path: studio/domains/session.yaml
revision: 2
---
## Outcome

A draft opened from a file follows it. A change on disk becomes a revision: the difference as a
patch, applied by a tool call with caller `File` (`studio.session.Caller` gains `File`), published
on the draft's channel, so every client sees it (LQ5, recommended option A). Extends `story:studio-live`. Design:
`design/lifecycle.md` L3, L5. Epic: `epic:agenticstudio`.

## Scope

- `studio/domains/session.yaml`: `Caller` `File`.
- `crates/agenticstudio`: watching the draft's file, the patch from the difference, the tool call.
- `docs/studio.md`, tests, `CHANGELOG.md`.
- Builds and tests through `slot`; no test over 60 s, the test step within 120 s, a pool job over
  10 minutes is killed.

## Acceptance

- Given a draft opened from a file and a client on its channel, when the file's `count` of one part
  changes on disk, the client receives a new revision whose tool call has caller `File` and whose
  patch sets that `count`.
- Given a change on disk that makes the document fail the check, when it is picked up, the revision
  is recorded with the check's refusal and the client sees both.
- Given a draft not opened from a file, when any file changes, no revision is made.
- Given the test step, when it runs, no test runs over 60 s and the step ends within 120 s.

## Not in this story

Write-back on release (`story:studio-release`).
