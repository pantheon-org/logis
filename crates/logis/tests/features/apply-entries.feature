# Not from a txtar script: how apply changes an existing target into its target state, one
# entry kind at a time (chezmoi's TargetState*.Apply in internal/chezmoi/targetstateentry.go).
Feature: apply to existing targets
  apply changes only what differs: contents, permissions, or the kind of entry.

  Background:
    Given an empty home directory

  Scenario: apply overwrites a file with different contents
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # new    |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | # old    |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.file" has contents "# new"

  Scenario: apply fixes the permissions of a file with the right contents
    Given the source state manages:
      | target  | kind       | contents |
      | ~/.file | executable | #!/bin/sh |
    And the home directory contains:
      | target  | kind | contents  |
      | ~/.file | file | #!/bin/sh |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.file" is executable

  Scenario: apply makes a private file private
    Given the source state manages:
      | target     | kind    | contents  |
      | ~/.private | private | # secret  |
    And the home directory contains:
      | target     | kind | contents |
      | ~/.private | file | # old    |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.private" has contents "# secret"
    And the file "~/.private" is private

  Scenario: apply replaces a directory with a file
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # file   |
    And the home directory contains:
      | target       | kind | contents |
      | ~/.file/sub  | file | # sub    |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.file" has contents "# file"

  Scenario: apply replaces a file with a directory
    Given the source state manages:
      | target     | kind      | contents |
      | ~/.dir     | directory |          |
      | ~/.dir/sub | file      | # sub    |
    And the home directory contains:
      | target | kind | contents |
      | ~/.dir | file | # file   |
    When I run "apply --force"
    Then it succeeds
    And "~/.dir" is a directory
    And the file "~/.dir/sub" has contents "# sub"

  Scenario: apply points a symlink at its new target
    Given the source state manages:
      | target     | kind    | contents |
      | ~/.symlink | symlink | new      |
    And the home directory contains:
      | target     | kind    | contents |
      | ~/.symlink | symlink | old      |
    When I run "apply --force"
    Then it succeeds
    And "~/.symlink" is a symlink to "new"

  Scenario: apply removes a target whose source file is empty
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file |          |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | # old    |
    When I run "apply --force"
    Then it succeeds
    And "~/.file" does not exist

  Scenario: apply keeps an empty file that is meant to be empty
    Given the source state manages:
      | target   | kind  | contents |
      | ~/.empty | empty |          |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.empty" is:
      """
      """

  Scenario: apply does not overwrite an existing create target
    Given the source state manages:
      | target    | kind   | contents  |
      | ~/.create | create | # source  |
    And the home directory contains:
      | target    | kind | contents   |
      | ~/.create | file | # existing |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.create" has contents "# existing"

  Scenario: apply removes a remove target
    Given the source state manages:
      | target  | kind   | contents |
      | ~/.gone | remove |          |
    And the home directory contains:
      | target       | kind | contents |
      | ~/.gone/file | file | # gone   |
    When I run "apply --force"
    Then it succeeds
    And "~/.gone" does not exist

  Scenario: apply leaves a target that is already right alone
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # same   |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | # same   |
    When I run "apply --force"
    Then it succeeds
    And the file "~/.file" has contents "# same"

  Scenario: apply fails for a target that is not managed
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # file   |
    When I run "apply --force ~/.nope"
    Then it fails
    And the error output mentions "not managed"

  # ADR-009: without a record of what it last wrote, logis cannot tell a target the user has
  # edited from one it wrote, so it applies only with --force.
  @chezmoi-only
  Scenario: apply without --force overwrites a target it never wrote
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # new    |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | # old    |
    When I run "apply"
    Then it succeeds
    And the file "~/.file" has contents "# new"

  @logis-only
  Scenario: apply without --force refuses and changes nothing
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | # new    |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | # old    |
    When I run "apply"
    Then it fails
    And the error output mentions "--force"
    And the file "~/.file" has contents "# old"
