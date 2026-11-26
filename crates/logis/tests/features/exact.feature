# Not from a txtar script: the exact-directory part of chezmoi's SourceState.Read, observed
# through managed. The apply side is in from-txtar/applyexact.feature.
Feature: exact directories
  An exact directory manages its whole contents: every entry in the destination that the
  source state does not manage is to be removed. managed lists those entries as the remove
  entry type.

  Background:
    Given an empty home directory
    And the source state manages:
      | target        | kind  | contents                 |
      | ~/.dir        | exact |                          |
      | ~/.dir/file   | file  | # contents of .dir/file  |
      | ~/.dir/subdir | exact |                          |
    And the home directory contains:
      | target                  | kind      | contents                |
      | ~/.dir/file             | file      | # contents of .dir/file |
      | ~/.dir/extra            | file      | # extra                 |
      | ~/.dir/.hidden          | file      | # hidden                |
      | ~/.dir/extradir/deep/f  | file      | # deep                  |
      | ~/.dir/link             | symlink   | nowhere                 |
      | ~/.dir/subdir/gone      | file      | # gone                  |

  Scenario: managed lists unmanaged entries of exact directories
    When I run "managed"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/.hidden
      .dir/extra
      .dir/extradir
      .dir/file
      .dir/link
      .dir/subdir
      .dir/subdir/gone
      """

  Scenario: managed --include=remove lists only the entries to remove
    When I run "managed --include=remove"
    Then it succeeds
    And the output is:
      """
      .dir/.hidden
      .dir/extra
      .dir/extradir
      .dir/link
      .dir/subdir/gone
      """

  Scenario: managed --exclude=remove leaves out the entries to remove
    When I run "managed --exclude=remove"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/file
      .dir/subdir
      """

  Scenario: managed --include=files does not list entries to remove
    When I run "managed --include=files"
    Then it succeeds
    And the output is:
      """
      .dir/file
      """

  Scenario: managed --path-style=absolute lists entries to remove by destination path
    When I run "managed --include=remove --path-style=absolute"
    Then it succeeds
    And the output is:
      """
      {home}/.dir/.hidden
      {home}/.dir/extra
      {home}/.dir/extradir
      {home}/.dir/link
      {home}/.dir/subdir/gone
      """

  # chezmoi's SourceStateAbsent has an empty source path, so each entry prints as an empty
  # line. Kept for parity; a candidate divergence.
  Scenario: managed --path-style=source-relative prints entries to remove as empty paths
    When I run "managed --include=remove --path-style=source-relative"
    Then it succeeds
    And the output is:
      """
      |
      |
      |
      |
      |
      """

  # As above: the empty source path joined to the source directory is the source directory.
  Scenario: managed --path-style=source-absolute prints entries to remove as the source directory
    When I run "managed --include=remove --path-style=source-absolute"
    Then it succeeds
    And the output is:
      """
      {source}
      {source}
      {source}
      {source}
      {source}
      """

  Scenario: ignored entries of exact directories are not removed
    Given the ignore file contains:
      """
      .dir/extra
      .dir/subdir/**
      """
    When I run "managed --include=remove"
    Then it succeeds
    And the output is:
      """
      .dir/.hidden
      .dir/extradir
      .dir/link
      """

  Scenario: an exact directory missing from the destination removes nothing
    When I remove "~/.dir"
    And I run "managed"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/file
      .dir/subdir
      """

  Scenario: an exact directory that is a file in the destination removes nothing
    When I replace "~/.dir" with a file containing "# a file"
    And I run "managed"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/file
      .dir/subdir
      """

  Scenario: a directory that is not exact removes nothing
    Given the source state is empty
    And the source state manages:
      | target      | kind      | contents                |
      | ~/.dir      | directory |                         |
      | ~/.dir/file | file      | # contents of .dir/file |
    When I run "managed"
    Then it succeeds
    And the output is:
      """
      .dir
      .dir/file
      """
