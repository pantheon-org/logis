# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/status.txtar,
# then specified by hand. The fixture mirrors chezmoi's `mksourcedir` without the template,
# readonly and remove entries.
@wip
Feature: status
  status lists what apply would change, one line per target: two status columns (the
  destination's change since the last apply, then the change apply would make) and the path.

  Background:
    Given an empty home directory
    And the source state manages:
      | target             | kind       | contents                    |
      | ~/.create          | create     | # contents of .create       |
      | ~/.dir             | directory  |                             |
      | ~/.dir/file        | file       | # contents of .dir/file     |
      | ~/.dir/subdir      | exact      |                             |
      | ~/.dir/subdir/file | file       | # contents of .dir/subdir/file |
      | ~/.empty           | empty      |                             |
      | ~/.executable      | executable | # contents of .executable   |
      | ~/.file            | file       | # contents of .file         |
      | ~/.private         | private    | # contents of .private      |
      | ~/.symlink         | symlink    | .dir/subdir/file            |

  # The txtar script runs this test twice; it is specified once here.
  Scenario: status lists all files to be added
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | A .create
      | A .dir
      | A .dir/file
      | A .dir/subdir
      | A .dir/subdir/file
      | A .empty
      | A .executable
      | A .file
      | A .private
      | A .symlink
      """

  @unix
  Scenario: the --path-style=absolute works as expected
    When I run "status --path-style=absolute"
    Then it succeeds
    And the output is:
      """
      | A {home}/.create
      | A {home}/.dir
      | A {home}/.dir/file
      | A {home}/.dir/subdir
      | A {home}/.dir/subdir/file
      | A {home}/.empty
      | A {home}/.executable
      | A {home}/.file
      | A {home}/.private
      | A {home}/.symlink
      """

  Scenario: status omits applied files
    Given I have run "apply --force ~/.file"
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | A .create
      | A .dir
      | A .dir/file
      | A .dir/subdir
      | A .dir/subdir/file
      | A .empty
      | A .executable
      | A .private
      | A .symlink
      """

  Scenario: status is empty after apply
    Given I have run "apply --force"
    When I run "status"
    Then it succeeds
    And the output is empty

  Scenario: status identifies files that have been modified in the destination directory
    Given I have run "apply --force"
    When I append "# edited" to "~/.file"
    And I run "status"
    Then it succeeds
    And the output is:
      """
      |MM .file
      """

  Scenario: status does not emit status for equivalent modifications
    Given I have run "apply --force"
    When I append "# edited" to "~/.file"
    And I append "# edited" to the source state of "~/.file"
    And I run "status"
    Then it succeeds
    And the output is empty

  # Needs config: specify once config scenarios can be written for both tools (roadmap phase 3).
  #   [unix] chhome home2/user ; mksourcedir ; exec chezmoi status ; cmpenv stdout golden/status-absolute-config
  #   with home2/user/.config/chezmoi/chezmoi.toml: [status] pathStyle = "absolute"
  @skeleton
  Scenario: the pathStyle config option works as expected
