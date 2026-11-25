# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/managed.txtar,
# then specified by hand. The fixture mirrors chezmoi's `mksourcedir` without the encrypted,
# readonly, remove and template entries, plus a script.
Feature: managed
  managed lists the targets the source state manages, sorted, filtered by entry type and by
  the targets given as arguments.

  Background:
    Given an empty home directory
    And the source state manages:
      | target             | kind       | contents                       |
      | ~/.create          | create     | # contents of .create          |
      | ~/.dir             | directory  |                                |
      | ~/.dir/file        | file       | # contents of .dir/file        |
      | ~/.dir/subdir      | exact      |                                |
      | ~/.dir/subdir/file | file       | # contents of .dir/subdir/file |
      | ~/.empty           | empty      |                                |
      | ~/.executable      | executable | # contents of .executable      |
      | ~/.file            | file       | # contents of .file            |
      | ~/.private         | private    | # contents of .private         |
      | ~/.symlink         | symlink    | .dir/subdir/file               |
      | ~/script           | script     | #!/bin/sh                      |

  Scenario: managed
    When I run "managed"
    Then it succeeds
    And the output is:
      """
      .create
      .dir
      .dir/file
      .dir/subdir
      .dir/subdir/file
      .empty
      .executable
      .file
      .private
      .symlink
      script
      """

  Scenario: managed --include=all
    When I run "managed --include=all"
    Then it succeeds
    And the output is:
      """
      .create
      .dir
      .dir/file
      .dir/subdir
      .dir/subdir/file
      .empty
      .executable
      .file
      .private
      .symlink
      script
      """

  Scenario: managed --include=dirs
    When I run "managed --include=dirs"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/subdir
      """

  Scenario: managed --include=files
    When I run "managed --include=files"
    Then it succeeds
    And the output is:
      """
      .create
      .dir/file
      .dir/subdir/file
      .empty
      .executable
      .file
      .private
      """

  Scenario: managed --include=symlinks
    When I run "managed --include=symlinks"
    Then it succeeds
    And the output is:
      """
      .symlink
      """

  # Not from the txtar script.
  Scenario: managed --include=scripts
    When I run "managed --include=scripts"
    Then it succeeds
    And the output is:
      """
      script
      """

  Scenario: managed --exclude=files
    When I run "managed --exclude=files"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/subdir
      .symlink
      script
      """

  Scenario: managed with arguments
    When I run "managed ~/.dir ~/.create"
    Then it succeeds
    And the output is:
      """
      .create
      .dir
      .dir/file
      .dir/subdir
      .dir/subdir/file
      """

  Scenario: managed with child of managed dir as argument
    When I run "managed ~/.dir/subdir"
    Then it succeeds
    And the output is:
      """
      .dir/subdir
      .dir/subdir/file
      """

  Scenario: managed --exclude=dir with arguments
    When I run "managed --exclude=dirs ~/.dir ~/.create"
    Then it succeeds
    And the output is:
      """
      .create
      .dir/file
      .dir/subdir/file
      """

  Scenario: managed with absent arguments
    When I run "managed ~/.dir ~/.non-exist"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/file
      .dir/subdir
      .dir/subdir/file
      """

  @unix
  Scenario: managed --path-style=absolute
    When I run "managed --path-style=absolute ~/.dir"
    Then it succeeds
    And the output is:
      """
      {home}/.dir
      {home}/.dir/file
      {home}/.dir/subdir
      {home}/.dir/subdir/file
      """

  Scenario: managed --path-style=source-absolute
    When I run "managed --path-style=source-absolute ~/.dir"
    Then it succeeds
    And the output is:
      """
      {source}/dot_dir
      {source}/dot_dir/exact_subdir
      {source}/dot_dir/exact_subdir/file
      {source}/dot_dir/file
      """

  Scenario: managed --path-style=source-relative
    When I run "managed --path-style=source-relative"
    Then it succeeds
    And the output is:
      """
      create_dot_create
      dot_dir
      dot_dir/exact_subdir
      dot_dir/exact_subdir/file
      dot_dir/file
      dot_file
      empty_dot_empty
      executable_dot_executable
      private_dot_private
      run_script
      symlink_dot_symlink
      """

  # Not from the txtar script: TestSourceStateRead "ignore_file" and "ignore_dir".
  Scenario: managed ignores hidden files in the source directory
    Given the source directory contains a hidden file ".hidden"
    When I run "managed"
    Then it succeeds
    And the output is:
      """
      .create
      .dir
      .dir/file
      .dir/subdir
      .dir/subdir/file
      .empty
      .executable
      .file
      .private
      .symlink
      script
      """

  # Not from the txtar script: TestSourceStateRead "duplicate_target_file".
  Scenario: managed fails when two source entries have the same target
    Given the source state manages:
      | target  | kind       | contents            |
      | ~/.file | executable | # contents of .file |
    When I run "managed"
    Then it fails
    And the error output mentions ".file: inconsistent state"

  # Need encryption or templates, which the steps cannot write yet.
  @skeleton
  Scenario: managed --exclude-encrypted

  @skeleton
  Scenario: managed --exclude=files,templates

  @skeleton
  Scenario: managed --include=encrypted

  @skeleton
  Scenario: managed --include=files --exclude=templates

  @skeleton
  Scenario: managed --include=templates

  @skeleton
  Scenario: managed does not evaluate templates

  # Output formats: specify with the json/yaml output work.
  @skeleton
  Scenario: managed --path-style=all

  @skeleton
  Scenario: managed --path-style=all --format=yaml
