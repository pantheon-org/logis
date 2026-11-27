# Decisions

Architecture decision records. Each one gives its status, the decision, and why. An intended
divergence from chezmoi always gets an entry, plus tagged BDD scenarios.

## ADR-001: Separate repository, chezmoi as oracle

- Status: accepted
- logis lives in its own repository. The chezmoi fork is read-only reference, and a mise-pinned
  `chezmoi` release is the behavioural oracle.
- Why: no runtime fallback to chezmoi, no shared names, and a clean history for the rewrite.

## ADR-002: BDD first, then TDD

- Status: accepted
- Every behaviour starts as a Gherkin scenario run against chezmoi (what we have). Tags mark what
  logis must do differently. Units are built test-first underneath.
- Why: it pins down current behaviour before rewriting, and makes every divergence explicit.

## ADR-003: Black-box oracle with rust-anything

- Status: accepted
- Use `rust-anything app record` and `app replay --build` for byte-level parity. Do not use
  `front-door` or `lift`.
- Why: chezmoi's interface-heavy Go code does not suit unit bridging, and logis changes config and
  templating on purpose.

## ADR-004: Pkl for configuration

- Status: accepted. The integration approach is proposed.
- Config is Pkl, validated by `pkl/Logis.pkl`. Start by running `pkl eval -f json` and
  deserializing with serde.
- Why: typed and validated config, and the same language as `hk.pkl`. There is no official Rust
  binding, so the CLI is the simplest robust option.

## ADR-005: Jinja templates through minijinja

- Status: proposed (to be confirmed)
- Why: minijinja is written by the author of Jinja2, is pure Rust, and supports strict undefined
  handling.

## ADR-006: Code health tooling

- Status: accepted
- The tools:
  - ArchUnitRust for layering, boundaries, cycles and size
  - clippy `cognitive_complexity` at 15, because bonsai-lint has no Rust support
  - bonsai-lint for Go and helper scripts
  - agnix for agent configs
  - cargo-deny for dependencies and licences

## ADR-007: Keep chezmoi's source-name grammar, and protect three more prefixes

- Status: accepted (2026-11-25)
- logis reads and writes chezmoi's source names (`dot_`, `private_`, `exact_`, `run_once_`,
  `.tmpl`, ...), ported from `internal/chezmoi/attr.go` into `logis_core::source_name`.
  Existing chezmoi source directories therefore parse the same way.
- Divergence: when writing a source name, logis also protects target names that start with
  these prefixes, by adding `literal_`:
  - `remove_` and `external_`, for directories
  - `onchange_`, for files and scripts
- Why: chezmoi's protection lists (`dirPrefixRegexp`, `filePrefixRegexp`) lack these prefixes,
  so such names do not round-trip.
  - With chezmoi 2.74.1, `add ~/remove_foo` stores the directory as `remove_foo`, which reads
    back as "remove `foo`". `managed` and `status` then report `foo`, and `external_bar`
    likewise becomes `bar`.
  - The script `onchange_x` reads back as the onchange script `x`.
  - Property tests over random names (`crates/logis-core/tests/source_name_props.rs`) found
    exactly these cases and no others.
- Compatibility:
  - Only these names are written differently, and only by logis.
  - chezmoi reads the protected names (`literal_remove_foo`) correctly.
  - Names chezmoi already wrote still parse as chezmoi parses them.
  - Regular files named `onchange_*` also get `literal_`, as chezmoi already does for `once_*`.
    chezmoi parses both forms to the same target.
- Specified by: `attribute_like_dir_names_are_protected` and `onchange_names_are_protected`
  (`crates/logis-core/tests/source_name.rs`). The BDD scenarios are in
  `crates/logis/tests/features/from-txtar/add.feature`: "...manages different targets"
  (`@chezmoi-only`) and "...manages the same targets" (`@logis-only`).

## ADR-008: `.logis*` special files

- Status: accepted (2026-11-26)
- logis's special files in the source directory use the prefix `.logis` where chezmoi uses
  `.chezmoi`, with the same suffixes and the same meaning:

  | logis | chezmoi |
  | --- | --- |
  | `.logisignore` | `.chezmoiignore` |
  | `.logisremove` | `.chezmoiremove` |
  | `.logisscripts/` | `.chezmoiscripts/` |
  | `.logisversion` | `.chezmoiversion` |
  | `.logisdata.<format>`, `.logisdata/` | `.chezmoidata.<format>`, `.chezmoidata/` |
  | `.logistemplates/` | `.chezmoitemplates/` |
  | `.logisexternal.<format>`, `.logisexternals/` | `.chezmoiexternal.<format>`, `.chezmoiexternals/` |
  | `.logisroot` | `.chezmoiroot` |

- Why: logis must not reuse chezmoi's names (ADR-001), and a one-to-one rename keeps chezmoi's
  documentation and a future `logis migrate` simple.
- Like any other name starting with `.`, chezmoi's `.chezmoi*` files are not part of a logis
  source state. logis ignores them; it does not report them as errors.
- The names live in `logis_core::special` and, for the BDD steps, in
  `crates/logis/tests/steps/adapter.rs`.
- Not yet decided: the data formats (`.pkl` alongside or instead of JSON, TOML and YAML) and the
  template suffix (phase 3 and phase 4).
- Until templates arrive (phase 4), logis reads `.logisignore` and `.logisremove` as plain
  text. chezmoi always runs them as templates. This is a gap, not a divergence.

## ADR-009: Until logis records what it wrote, `apply` needs `--force` and `status` has one column

- Status: accepted (2026-11-26), temporary; extended to `status` on 2026-11-27
- chezmoi decides whether to prompt before overwriting a target from the entry state it
  recorded when it last wrote it. A target that changed since then is a conflict. A target
  with no record is overwritten without asking, as on a first apply.
- logis has no state store yet (redb is still proposed), so every apply would be a first
  apply: a target the user edited after the last apply would be overwritten silently.
- Divergence: until the state store exists, `logis apply` without `--force` fails before
  changing anything, with an error that names `--force`. `--dry-run` changes nothing, so it
  does not need `--force`.
- Specified by: "apply without --force overwrites a target it never wrote" (`@chezmoi-only`)
  and "apply without --force refuses and changes nothing" (`@logis-only`) in
  `crates/logis/tests/features/apply-entries.feature`.
- `status`: chezmoi's first column compares that same record with the destination (the
  target changed since the last apply). Without a record, logis leaves the first column
  blank and fills only the second (the change apply would make), as chezmoi does for a
  target it has never written. Specified by "status identifies files that have been
  modified in the destination directory" (`@chezmoi-only`) and "...shows only the change
  apply would make" (`@logis-only`) in `crates/logis/tests/features/from-txtar/status.feature`.
- Revisit when the state store lands: logis should then prompt as chezmoi does and fill both
  status columns, and this ADR is superseded.

## Open (proposed)

- `redb` for persistent state, in place of chezmoi's bolt.
- `~/.config/logis/` and `~/.local/share/logis/` locations.
- Platforms: macOS and Linux first, Windows later.
