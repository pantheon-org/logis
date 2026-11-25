//! The `managed` query: which targets, in which path style, in which order.

use std::path::Path;

use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::managed::{PathStyle, Query, managed};
use logis_core::source_state::SourceState;
use logis_core::system::MemSystem;

const SOURCE: &str = "/src";
const DEST: &str = "/home/u";

fn state() -> SourceState {
    let system = MemSystem::new()
        .with_dir("/src/dot_dir/exact_subdir", 0o755)
        .with_file("/src/dot_dir/file", "", 0o644)
        .with_file("/src/dot_dir/exact_subdir/file", "", 0o644)
        .with_file("/src/symlink_dot_link", ".dir/file", 0o644)
        .with_file("/src/run_script", "", 0o644);
    SourceState::read(&system, Path::new(SOURCE)).unwrap()
}

fn query<'a>(targets: &'a [&'a str], style: PathStyle, filter: EntryTypeFilter) -> Query<'a> {
    Query {
        filter,
        targets,
        style,
        dest_dir: Path::new(DEST),
    }
}

fn run(targets: &[&str], style: PathStyle, filter: EntryTypeFilter) -> Vec<String> {
    managed(&state(), &query(targets, style, filter))
}

#[test]
fn lists_every_target_in_target_order() {
    assert_eq!(
        run(&[], PathStyle::Relative, EntryTypeFilter::default()),
        [
            ".dir",
            ".dir/file",
            ".dir/subdir",
            ".dir/subdir/file",
            ".link",
            "script"
        ]
    );
}

#[test]
fn filters_by_entry_type() {
    let only = |types: &str| EntryTypeFilter {
        include: types.parse().unwrap(),
        exclude: EntryTypeSet::none(),
    };
    assert_eq!(
        run(&[], PathStyle::Relative, only("dirs")),
        [".dir", ".dir/subdir"]
    );
    assert_eq!(run(&[], PathStyle::Relative, only("symlinks")), [".link"]);
    let no_files = EntryTypeFilter {
        include: EntryTypeSet::all(),
        exclude: "files".parse().unwrap(),
    };
    assert_eq!(
        run(&[], PathStyle::Relative, no_files),
        [".dir", ".dir/subdir", ".link", "script"]
    );
}

#[test]
fn targets_select_themselves_and_their_descendants() {
    assert_eq!(
        run(
            &[".dir/subdir", "script", ".absent"],
            PathStyle::Relative,
            EntryTypeFilter::default()
        ),
        [".dir/subdir", ".dir/subdir/file", "script"]
    );
    // A target is matched as a path, not as a string prefix.
    assert!(run(&[".di"], PathStyle::Relative, EntryTypeFilter::default()).is_empty());
}

#[test]
fn path_styles() {
    let all = EntryTypeFilter::default();
    assert_eq!(
        run(&[".dir/subdir"], PathStyle::Absolute, all),
        ["/home/u/.dir/subdir", "/home/u/.dir/subdir/file"]
    );
    assert_eq!(
        run(&[".dir"], PathStyle::SourceRelative, all),
        [
            "dot_dir",
            "dot_dir/exact_subdir",
            "dot_dir/exact_subdir/file",
            "dot_dir/file"
        ]
    );
    assert_eq!(
        run(&[".link"], PathStyle::SourceAbsolute, all),
        ["/src/symlink_dot_link"]
    );
}

#[test]
fn path_style_names() {
    for (name, style) in [
        ("relative", PathStyle::Relative),
        ("absolute", PathStyle::Absolute),
        ("source-relative", PathStyle::SourceRelative),
        ("source-absolute", PathStyle::SourceAbsolute),
    ] {
        assert_eq!(name.parse::<PathStyle>(), Ok(style));
    }
    assert!("all".parse::<PathStyle>().is_err());
}

#[test]
fn the_destination_itself_selects_everything() {
    assert_eq!(
        run(&[""], PathStyle::Relative, EntryTypeFilter::default()).len(),
        6
    );
}
