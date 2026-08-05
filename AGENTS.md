# Agent Instructions

## Commit discipline

- Do not finish a task with modified or untracked files created by the agent.
- Commit every completed change before reporting it as done. Use one or more coherent, reviewable commits.
- Run `git status` before completion and explicitly resolve any remaining changes.
- If pre-existing user work cannot be safely included, leave it untouched and clearly identify it instead of mixing it into a commit.
- Never push directly to `main`; push a feature branch and use a pull request.
