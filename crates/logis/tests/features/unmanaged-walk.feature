# Not from a txtar script: how unmanaged walks the destination (chezmoi's runUnmanagedCmd in
# internal/cmd/unmanagedcmd.go), found by running chezmoi in a sandbox.
Feature: unmanaged walk
  unmanaged looks inside managed directories only, skips ignored entries, and does not follow
  symlinks.

  Background:
    Given an empty home directory
    And the source state manages:
      | target           | kind      | contents |
      | ~/.managed       | directory |          |
      | ~/.managed/known | file      | x        |
    And the home directory contains:
      | target             | kind    | contents |
      | ~/.managed/known   | file    | x        |
      | ~/.managed/extra   | file    | x        |
      | ~/.managed/sub/f   | file    | x        |
      | ~/.dir/file        | file    | x        |
      | ~/.dirlink         | symlink | .dir     |
      | ~/.dangling        | symlink | nowhere  |
      | ~/.ignored         | file    | x        |
      | ~/.ign/x           | file    | x        |
    And the ignore file contains:
      """
      .ignored
      .ign
      .managed/extra
      """

  Scenario: unmanaged looks inside managed directories and skips ignored entries
    When I run "unmanaged"
    Then it succeeds
    And the output is:
      """
      .dangling
      .dir
      .dirlink
      .managed/sub
      """

  Scenario: unmanaged --include=symlinks lists symlinks, dangling or not
    When I run "unmanaged --include=symlinks"
    Then it succeeds
    And the output is:
      """
      .dangling
      .dirlink
      """

  Scenario: unmanaged of a managed directory looks inside it
    When I run "unmanaged ~/.managed"
    Then it succeeds
    And the output is:
      """
      .managed/sub
      """

  # The pattern .ign matches the directory, not the file given.
  Scenario: unmanaged of a file inside an ignored directory lists it
    When I run "unmanaged ~/.ign/x"
    Then it succeeds
    And the output is:
      """
      .ign/x
      """

  Scenario: unmanaged of the destination directory is unmanaged of everything
    When I run "unmanaged ~"
    Then it succeeds
    And the output is:
      """
      .dangling
      .dir
      .dirlink
      .managed/sub
      """

  Scenario: unmanaged lists each path once
    When I run "unmanaged ~/.dir ~/.dir ~/.dirlink"
    Then it succeeds
    And the output is:
      """
      .dir
      .dirlink
      """

  Scenario: unmanaged arguments are relative to the working directory
    When I run "unmanaged file" in "~/.dir"
    Then it succeeds
    And the output is:
      """
      .dir/file
      """

  Scenario: unmanaged --tree prints the paths as a tree
    When I run "unmanaged --tree"
    Then it succeeds
    And the output is:
      """
      .dangling
      .dir
      .dirlink
      .managed
        sub
      """

  # A file, not a directory such as /etc: chezmoi walks into a directory outside the
  # destination first, so what it reports depends on the machine.
  Scenario: unmanaged fails for a path outside the destination directory
    When I run "unmanaged {source-of:~/.managed/known}"
    Then it fails
    And the error output mentions "not in {home}"
    And the output is empty
