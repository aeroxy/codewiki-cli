---
name: codewiki
description: Read GitHub repository wikis — list sections, dump the full Markdown, or ask Gemini-backed questions about any public GitHub repo. Use when the user wants to download, read, or query a Wiki for a GitHub repository.
user-invocable: true
---

```bash
codewiki structure <owner>/<repo>          # section titles only
codewiki read <owner>/<repo>               # full wiki as Markdown
codewiki ask <owner>/<repo> "<question>"   # Gemini Q&A, ~10 s
```

Pick `structure` to scout, `read` for context to feed another agent, `ask` for one targeted question. Public repos only; `ask` is single-turn.
