# logis

Manage your dotfiles across machines. logis is a Rust rewrite of the ideas in
[chezmoi](https://github.com/twpayne/chezmoi). It has a source directory, a computed target state
and `apply`, with [Pkl](https://github.com/apple/pkl) configuration and Jinja templates.

Status: scaffolding. Nothing is usable yet.

## Development

```bash
mise install
```

```bash
hk install
```

```bash
mise run test
```

```bash
mise run bdd:oracle
```

Behaviour is specified as BDD features (`crates/logis/tests/features/`). The same features run
against chezmoi, to show current behaviour, and against logis, to show what logis must achieve.
See `CLAUDE.md` for the workflow and `DECISIONS.md` for the decisions.

## Acknowledgements

logis is checked against the behaviour of chezmoi, by Tom Payne and contributors (MIT). No
chezmoi source code is copied.

## License

MIT
