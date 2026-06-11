## Before doing anything
- read `state.md` — current project snapshot
- read `decisions.md` — architectural decisions + unresolved ADRs
- read `context.md` — operational context, known issues, important commands
- read `todos.md` — live prioritized execution list
- read `AGENTS.md` — detailed backend/frontend structure and architecture
- read `roadmap.md` — prioritized audit findings (P0-P3)

## After each task
- update `state.md` if project status changed
- update `todos.md` — mark completed items
- log new decisions in `decisions.md` if architecture changed
- sync summarized mirror to `autonomous-app-builder/memory/*`

## Additional Guidance
- P0/P1 roadmap items take priority over new features
- Flag doc drift between README, AGENTS, and actual code when found
- Run `pnpm test` after changes to verify frontend tests
