# Keepjay project cards

Cards about how this repository works. `keepjay scan` picks them up
(docs/specs/13-code-sources.md §4).

Q: Which features are frozen by D-028?
A: Workflows, dashboards, presets, notes and the planner (specs 03, 04 and 05). Building them needs a new decision.
<!-- id: da0s7add -->

Q: How are pull requests merged here (D-021)?
A: Squash-merged. The PR title becomes the commit message, so it must be a Conventional Commit: `<type>(<scope>): <summary>`.
<!-- id: jpw47j04 -->

Q: What does Keepjay Pro charge for (D-033)?
A: Hosted sync, backups and AI drafting. Local features stay free forever.
<!-- id: k74rgeyh -->

Q: In what order do the Keepjay surfaces ship (D-034)?
A: CLI, then the VS Code extension, then the mobile review app, then web. There's no standalone desktop app.
<!-- id: 71vetbwq -->

C: Every database change goes through {{c1::the oplog}}, stamped with a {{c2::hybrid logical clock}}, so sync can merge per field (D-009).
<!-- id: gv0cbcm0 -->
