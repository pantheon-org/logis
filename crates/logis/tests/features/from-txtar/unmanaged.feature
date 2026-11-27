# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/unmanaged.txtar,
# then specified by hand. chezmoi's source directory is inside its home there (~/.local); here
# ~/.local/share/file stands in for it. The Background ends where the txtar setup does, with
# .dir and .file applied and then removed from the source state.
Feature: unmanaged
  unmanaged lists the entries in the destination that the source state neither manages nor
  ignores, without looking inside unmanaged directories.

  Background:
    Given an empty home directory
    And the source state manages:
      | target             | kind       | contents                       |
      | ~/.create          | create     | # contents of .create          |
      | ~/.dir             | directory  |                                |
      | ~/.dir/file        | file       | # contents of .dir/file        |
      | ~/.dir/subdir      | directory  |                                |
      | ~/.dir/subdir/file | file       | # contents of .dir/subdir/file |
      | ~/.empty           | empty      |                                |
      | ~/.executable      | executable | # contents of .executable      |
      | ~/.file            | file       | # contents of .file            |
      | ~/.private         | private    | # contents of .private         |
      | ~/.symlink         | symlink    | .dir/subdir/file               |
    And I have run "apply --force"
    And the home directory contains:
      | target              | kind | contents |
      | ~/.local/share/file | file | x        |
    And "~/.dir" is removed from the source state
    And "~/.file" is removed from the source state

  # setup (from txtar):
  #   mkhomedir
  #   mksourcedir
  #   exec chezmoi unmanaged
  #   cmp stdout golden/unmanaged
  #   rm $CHEZMOISOURCEDIR/dot_dir
  #   exec chezmoi unmanaged
  #   cmp stdout golden/unmanaged-dir
  #   rm $CHEZMOISOURCEDIR/dot_file
  #   exec chezmoi unmanaged
  #   cmp stdout golden/unmanaged-dir-file

  # archive files:
  #   golden/unmanaged
  #   golden/unmanaged-absolute
  #   golden/unmanaged-dir
  #   golden/unmanaged-dir-file
  #   golden/unmanaged-exclude-dirs
  #   golden/unmanaged-include-dirs
  #   golden/unmanaged-inside-unmanaged
  #   golden/unmanaged-with-args
  #   golden/unmanaged-with-some-managed

  Scenario: unmanaged lists unmanaged entries, not what is inside them
    When I run "unmanaged"
    Then it succeeds
    And the output is:
      """
      .dir
      .file
      .local
      """

  Scenario: unmanaged with arguments
    When I run "unmanaged ~/.dir ~/.file"
    Then it succeeds
    And the output is:
      """
      .dir
      .file
      """

  Scenario: unmanaged, with child of unmanaged dir as argument
    When I run "unmanaged ~/.dir/subdir"
    Then it succeeds
    And the output is:
      """
      .dir/subdir
      """

  Scenario: unmanaged with managed arguments
    When I run "unmanaged ~/.create ~/.file"
    Then it succeeds
    And the output is:
      """
      .file
      """

  Scenario: unmanaged with absent paths should fail
    When I run "unmanaged ~/absent-path"
    Then it fails
    And the output is empty

  @unix
  Scenario: unmanaged --path-style=absolute
    When I run "unmanaged --path-style=absolute"
    Then it succeeds
    And the output is:
      """
      {home}/.dir
      {home}/.file
      {home}/.local
      """

  Scenario: unmanaged --exclude=dirs
    When I run "unmanaged --exclude=dirs"
    Then it succeeds
    And the output is:
      """
      .file
      """

  Scenario: unmanaged --include=dirs
    When I run "unmanaged --include=dirs"
    Then it succeeds
    And the output is:
      """
      .dir
      .local
      """

  Scenario: unmanaged --path-style=source-absolute
    When I run "unmanaged --path-style=source-absolute"
    Then it fails
