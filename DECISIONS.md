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

## Open (proposed)

- `redb` for persistent state, in place of chezmoi's bolt.
- `.logis*` special files; `~/.config/logis/` and `~/.local/share/logis/` locations.
- Platforms: macOS and Linux first, Windows later.
