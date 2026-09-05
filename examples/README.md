# dbmd examples

These are complete dbmd projects whose generated artifacts are committed for
inspection. Each example exposes the same workflow:

```sh
just render
just verify
just down
```

`render` creates disposable databases from the readable SQL in `schema/`, then
runs dbmd. Server-backed examples start exact-version containers and initialize
them automatically. `down` removes only resources owned by that example.

Start with [the SQLite quickstart](quickstart/sqlite/README.md), inspect one of
the [backend showcases](backends/), explore a focused project under
`workflows/`, or run the [full multi-backend showcase](full/README.md).

Generated Markdown is part of each example. You can evaluate dbmd's output
without installing a database or starting Docker.
