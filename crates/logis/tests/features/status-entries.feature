# Not from a txtar script: the status letter for each kind of change (chezmoi's statusRune in
# internal/cmd/statuscmd.go), found by running chezmoi in a sandbox. Only the second column
# is filled, as for targets chezmoi has never written (ADR-009).
Feature: status of each kind of change
  status marks additions A, modifications M, removals D and scripts R.

  Background:
    Given an empty home directory

  Scenario: a target to remove is D
    Given the source state manages:
      | target    | kind   | contents |
      | ~/.gone   | remove |          |
      | ~/.absent | remove |          |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.gone | file | x        |
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | D .gone
      """

  Scenario: an entry an exact directory removes is D
    Given the source state manages:
      | target | kind  | contents |
      | ~/.dir | exact |          |
    And the home directory contains:
      | target       | kind | contents |
      | ~/.dir/extra | file | x        |
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | D .dir/extra
      """

  Scenario: a target whose source file is empty is D
    Given the source state manages:
      | target     | kind | contents |
      | ~/.file    | file |          |
      | ~/.nothing | file |          |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | x        |
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | D .file
      """

  Scenario: a change of mode, kind or link target is M
    Given the source state manages:
      | target     | kind       | contents |
      | ~/.mode    | executable | x        |
      | ~/.wasfile | directory  |          |
      | ~/.link    | symlink    | new      |
    And the home directory contains:
      | target     | kind    | contents |
      | ~/.mode    | file    | x        |
      | ~/.wasfile | file    | x        |
      | ~/.link    | symlink | old      |
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | M .link
      | M .mode
      | M .wasfile
      """

  Scenario: an existing create target is unchanged
    Given the source state manages:
      | target    | kind   | contents |
      | ~/.create | create | # source |
    And the home directory contains:
      | target    | kind | contents |
      | ~/.create | file | # other  |
    When I run "status"
    Then it succeeds
    And the output is empty

  Scenario: a script is R
    Given the source state manages:
      | target  | kind   | contents  |
      | ~/script | script | #!/bin/sh |
    When I run "status"
    Then it succeeds
    And the output is:
      """
      | R script
      """

  Scenario: status --include=files lists only files
    Given the source state manages:
      | target     | kind      | contents  |
      | ~/.dir     | directory |           |
      | ~/.file    | file      | x         |
      | ~/.gone    | remove    |           |
      | ~/script   | script    | #!/bin/sh |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.gone | file | x        |
    When I run "status --include=files"
    Then it succeeds
    And the output is:
      """
      | A .file
      """

  Scenario: status --exclude=remove leaves out removals
    Given the source state manages:
      | target  | kind   | contents |
      | ~/.file | file   | x        |
      | ~/.gone | remove |          |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.gone | file | x        |
    When I run "status --exclude=remove"
    Then it succeeds
    And the output is:
      """
      | A .file
      """

  Scenario: status of a directory includes what is under it
    Given the source state manages:
      | target      | kind      | contents |
      | ~/.dir      | directory |          |
      | ~/.dir/file | file      | x        |
      | ~/.file     | file      | x        |
    When I run "status ~/.dir"
    Then it succeeds
    And the output is:
      """
      | A .dir
      | A .dir/file
      """

  Scenario: status --recursive=false lists only the directory
    Given the source state manages:
      | target      | kind      | contents |
      | ~/.dir      | directory |          |
      | ~/.dir/file | file      | x        |
    When I run "status --recursive=false ~/.dir"
    Then it succeeds
    And the output is:
      """
      | A .dir
      """

  Scenario: status fails for a target that is not managed
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | x        |
    When I run "status ~/.nope"
    Then it fails
    And the error output mentions "not managed"

  Scenario: status takes only the relative and absolute path styles
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | x        |
    When I run "status --path-style=source-relative"
    Then it fails

  Scenario: status does not need --force and changes nothing
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | x        |
    When I run "status"
    Then it succeeds
    And "~/.file" does not exist
