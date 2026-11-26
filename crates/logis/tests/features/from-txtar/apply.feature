# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/apply.txtar,
# then specified by hand. The fixture mirrors chezmoi's `mksourcedir` without the template,
# readonly and remove entries.
Feature: apply
  apply makes the home directory match the source state.

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

  Scenario: apply --dry-run does not create any files
    When I run "apply --dry-run --force"
    Then it succeeds
    And the home directory is still empty

  Scenario: apply file creates a single file only
    When I run "apply --force ~/.file"
    Then it succeeds
    And the file "~/.file" has contents "# contents of .file"
    And "~/.create" does not exist
    And "~/.dir" does not exist
    And "~/.empty" does not exist
    And "~/.executable" does not exist
    And "~/.private" does not exist
    And "~/.symlink" does not exist

  Scenario: apply dir --recursive=false creates only the directory
    When I run "apply --force --recursive=false ~/.dir"
    Then it succeeds
    And "~/.dir" is a directory
    And "~/.dir/file" does not exist
    And "~/.dir/subdir" does not exist

  Scenario: apply dir creates all files in the directory
    When I run "apply --force ~/.dir"
    Then it succeeds
    And the file "~/.dir/file" has contents "# contents of .dir/file"
    And "~/.dir/subdir" is a directory
    And the file "~/.dir/subdir/file" has contents "# contents of .dir/subdir/file"
    And "~/.file" does not exist

  @unix
  Scenario: apply creates all files
    When I run "apply --force"
    Then it succeeds
    And the file "~/.create" has contents "# contents of .create"
    And the file "~/.dir/file" has contents "# contents of .dir/file"
    And the file "~/.dir/subdir/file" has contents "# contents of .dir/subdir/file"
    And the file "~/.empty" is:
      """
      """
    And the file "~/.executable" has contents "# contents of .executable"
    And the file "~/.executable" is executable
    And the file "~/.file" has contents "# contents of .file"
    And the file "~/.private" has contents "# contents of .private"
    And the file "~/.private" is private
    And "~/.symlink" is a symlink to ".dir/subdir/file"

  Scenario: apply after edit
    Given I have run "apply --force"
    When I append "# edited" to the source state of "~/.file"
    And I run "apply --force"
    Then it succeeds
    And the file "~/.file" is:
      """
      # contents of .file
      # edited
      """

  Scenario: apply --source-path applies a file based on its source path
    Given I have run "apply --force"
    When I append "# edited" to the source state of "~/.file"
    And I run "apply --force --source-path {source-of:~/.file}"
    Then it succeeds
    And the file "~/.file" is:
      """
      # contents of .file
      # edited
      """

  Scenario: apply --source-path fails when called with a targetDirAbsPath
    When I run "apply --force --source-path ~/.file"
    Then it fails
    And the error output mentions "~/.file"

  # Not from the txtar script.
  Scenario: apply with an empty source directory changes nothing
    Given the source state is empty
    When I run "apply --force"
    Then it succeeds
    And the home directory is still empty
