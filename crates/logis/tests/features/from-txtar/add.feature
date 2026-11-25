# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/add.txtar,
# then specified by hand. The home fixture mirrors chezmoi's `mkhomedir`; the source state
# starts empty. Assertions are about what the source state manages, not how it stores it.
@wip
Feature: add
  add brings files from the home directory under management in the source state.

  Background:
    Given an empty home directory
    And the home directory contains:
      | target             | kind       | contents                       |
      | ~/.create          | file       | # contents of .create          |
      | ~/.dir             | directory  |                                |
      | ~/.dir/file        | file       | # contents of .dir/file        |
      | ~/.dir/subdir      | directory  |                                |
      | ~/.dir/subdir/file | file       | # contents of .dir/subdir/file |
      | ~/.empty           | empty      |                                |
      | ~/.executable      | executable | # contents of .executable      |
      | ~/.file            | file       | # contents of .file            |
      | ~/.private         | private    | # contents of .private         |
      | ~/.symlink         | symlink    | .dir/subdir/file               |

  Scenario: add --create
    When I run "add --create ~/.create"
    Then it succeeds
    And the source state manages "~/.create" as a create containing "# contents of .create"

  Scenario: adding a directory creates a .keep file
    # The .keep file is how the source state stores an empty directory; only the outcome is
    # specified here: the directory is managed, its contents are not.
    When I run "add --recursive=false ~/.dir"
    Then it succeeds
    And the source state manages "~/.dir" as a directory
    And the source state does not manage "~/.dir/file"

  Scenario: adding a file in a directory
    When I run "add ~/.dir/file"
    Then it succeeds
    And the source state manages "~/.dir/file" as a file containing "# contents of .dir/file"

  Scenario: adding a subdirectory
    When I run "add --exact ~/.dir/subdir"
    Then it succeeds
    And the source state manages "~/.dir/subdir" as an exact
    And the source state manages "~/.dir/subdir/file" as a file containing "# contents of .dir/subdir/file"

  Scenario: adding an empty file
    When I run "add ~/.empty"
    Then it succeeds
    And the source state manages "~/.empty" as an empty

  @unix
  Scenario: adding an executable file
    When I run "add ~/.executable"
    Then it succeeds
    And the source state manages "~/.executable" as an executable containing "# contents of .executable"

  @unix
  Scenario: adding a private file
    When I run "add ~/.private"
    Then it succeeds
    And the source state manages "~/.private" as a private containing "# contents of .private"

  Scenario: adding a symlink
    When I run "add ~/.symlink"
    Then it succeeds
    And the source state manages "~/.symlink" as a symlink containing ".dir/subdir/file"

  Scenario: adding a symlink with a separator
    When I create a symlink "~/.symlink2" to ".dir/subdir/file"
    And I run "add ~/.symlink2"
    Then it succeeds
    And the source state manages "~/.symlink2" as a symlink containing ".dir/subdir/file"

  Scenario: adding a symlink with --follow
    When I create a symlink "~/.symlink3" to ".file"
    And I run "add --follow ~/.symlink3"
    Then it succeeds
    And the source state manages "~/.symlink3" as a file containing "# contents of .file"

  Scenario: add only creates .keep files in empty directories
    # Specified by outcome: an empty subdirectory is managed as a directory alongside the files.
    When I create the directory "~/.dir/empty_subdir"
    And I run "add ~/.dir"
    Then it succeeds
    And the source state manages "~/.dir/empty_subdir" as a directory
    And the source state manages "~/.dir/file" as a file containing "# contents of .dir/file"

  Scenario: add respects .chezmoiignore
    Given the home directory contains:
      | target        | kind | contents                  |
      | ~/.dir/ignore | file | # contents of .dir/ignore |
    And the source state ignores "**/ignore"
    When I run "add ~/.dir"
    Then it succeeds
    And the source state manages "~/.dir/file" as a file containing "# contents of .dir/file"
    And the source state does not manage "~/.dir/ignore"
    And the error output mentions ".dir/ignore"

  Scenario: add does not overwrite an already-added file
    Given I have run "add ~/.file"
    When I append "# edited" to "~/.file"
    Then the source state manages "~/.file" as a file containing "# contents of .file"

  Scenario: add --force does overwrite an already-added file
    Given I have run "add ~/.file"
    When I append "# edited" to "~/.file"
    And I run "add --force ~/.file"
    Then it succeeds
    And the source state manages "~/.file" as a file containing:
      """
      # contents of .file
      # edited
      """

  Scenario: add --exact only applies exact_ to the target, not parent directories
    Given the home directory contains:
      | target                | kind      | contents                 |
      | ~/.parent             | directory |                          |
      | ~/.parent/child       | directory |                          |
      | ~/.parent/child/file  | file      | # contents of child file |
      | ~/.parent/other_file  | file      | # not tracked            |
    When I run "add --exact ~/.parent/child"
    Then it succeeds
    And the source state manages "~/.parent" as a directory
    And the source state manages "~/.parent/child" as an exact
    And the source state manages "~/.parent/child/file" as a file containing "# contents of child file"
    And the source state does not manage "~/.parent/other_file"

  # Not from the txtar script. ADR-007: chezmoi stores a directory named like an attribute
  # without protecting it, so the target it manages changes name. logis protects the name.
  @chezmoi-only
  Scenario: add of directories named like attributes manages different targets
    Given the home directory contains:
      | target              | kind      | contents |
      | ~/remove_foo        | directory |          |
      | ~/remove_foo/file   | file      | # foo    |
      | ~/external_bar      | directory |          |
      | ~/external_bar/file | file      | # bar    |
    When I run "add ~/remove_foo ~/external_bar"
    And I run "managed"
    Then it succeeds
    And the output is:
      """
      bar
      bar/file
      foo
      foo/file
      """

  @logis-only
  Scenario: add of directories named like attributes manages the same targets
    Given the home directory contains:
      | target              | kind      | contents |
      | ~/remove_foo        | directory |          |
      | ~/remove_foo/file   | file      | # foo    |
      | ~/external_bar      | directory |          |
      | ~/external_bar/file | file      | # bar    |
    When I run "add ~/remove_foo ~/external_bar"
    And I run "managed"
    Then it succeeds
    And the output is:
      """
      external_bar
      external_bar/file
      remove_foo
      remove_foo/file
      """
