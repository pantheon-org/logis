# Not from a txtar script: the git diff format that chezmoi's GitDiffSystem writes through
# go-git's unified encoder, found by running chezmoi in a sandbox. File modes are for umask
# 022; symlinks in the home directory are avoided because their mode differs between macOS
# and Linux.
@unix
Feature: diff format
  diff writes one git-style file patch per change apply would make, with three lines of
  context.

  Background:
    Given an empty home directory

  Scenario: changes seven unchanged lines apart are separate hunks
    Given the source state manages "~/.file" with lines 1 to 20
    And the home file "~/.file" has lines 1 to 20, except:
      | line | text    |
      | 5    | LINE 5  |
      | 13   | LINE 13 |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index 6ced3eaca332a6d939bc18e6d278cac420b9e31c..c4352f8b46de5cdb88d0cc96958316db42dd2398 100644
      --- a/.file
      +++ b/.file
      @@ -2,7 +2,7 @@ line 1
       line 2
       line 3
       line 4
      -LINE 5
      +line 5
       line 6
       line 7
       line 8
      @@ -10,7 +10,7 @@ line 9
       line 10
       line 11
       line 12
      -LINE 13
      +line 13
       line 14
       line 15
       line 16
      """

  Scenario: changes six unchanged lines apart share a hunk
    Given the source state manages "~/.file" with lines 1 to 20
    And the home file "~/.file" has lines 1 to 20, except:
      | line | text    |
      | 5    | LINE 5  |
      | 12   | LINE 12 |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index aace4bb77047f29c2288018cf6c1cc1c53ad22af..c4352f8b46de5cdb88d0cc96958316db42dd2398 100644
      --- a/.file
      +++ b/.file
      @@ -2,14 +2,14 @@ line 1
       line 2
       line 3
       line 4
      -LINE 5
      +line 5
       line 6
       line 7
       line 8
       line 9
       line 10
       line 11
      -LINE 12
      +line 12
       line 13
       line 14
       line 15
      """

  Scenario: lines without a final newline are marked
    Given the source state manages "~/.file" containing, without a final newline:
      """
      a
      b
      """
    And the home file "~/.file" contains, without a final newline:
      """
      a
      c
      """
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index 817f660e4423f7df2dfc7d4bff0e01b2092a8ce9..0a207c060e61f3b88eaee0a8cd0696f46fb155eb 100644
      --- a/.file
      +++ b/.file
      @@ -1,2 +1,2 @@
       a
      -c
      \ No newline at end of file
      +b
      \ No newline at end of file
      """

  Scenario: a missing final newline is a changed line
    Given the source state manages "~/.file" containing, without a final newline:
      """
      a
      b
      """
    And the home file "~/.file" contains:
      """
      a
      b
      """
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index 422c2b7ab3b3c668038da977e4e93a5fc623169c..0a207c060e61f3b88eaee0a8cd0696f46fb155eb 100644
      --- a/.file
      +++ b/.file
      @@ -1,2 +1,2 @@
       a
      -b
      +b
      \ No newline at end of file
      """

  Scenario: binary files are not compared line by line
    Given the source state manages "~/.bin" containing binary data
    And the home directory contains:
      | target | kind | contents |
      | ~/.bin | file | text     |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.bin b/.bin
      index 8e27be7d6154a1f68ea9160ef0e18691d20560dc..88768efdf77ec78c9a995f94881793be6a41752b 100644
      Binary files a/.bin and b/.bin differ
      """

  Scenario: a new binary file
    Given the source state manages "~/.bin" containing binary data
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.bin b/.bin
      new file mode 100644
      index 0000000000000000000000000000000000000000..88768efdf77ec78c9a995f94881793be6a41752b
      Binary files /dev/null and b/.bin differ
      """

  # Go's http.DetectContentType calls this application/pdf, so it is binary.
  Scenario: a text file that starts like a PDF is binary
    Given the source state manages:
      | target | kind | contents      |
      | ~/.pdf | file | %PDF-1.4 text |
    And the home directory contains:
      | target | kind | contents |
      | ~/.pdf | file | plain    |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.pdf b/.pdf
      index b9bca019c83a65e6d717d0b6da86215f45dde1b3..510e91eb937429cd30ae4360683c9c2851af66b9 100644
      Binary files a/.pdf and b/.pdf differ
      """

  Scenario: a removed file
    Given the source state manages:
      | target  | kind   | contents |
      | ~/.gone | remove |          |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.gone | file | x        |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.gone b/.gone
      deleted file mode 100644
      index 587be6b4c3f93f93c489c0111bba5596147a26cb..0000000000000000000000000000000000000000
      --- a/.gone
      +++ /dev/null
      @@ -1 +0,0 @@
      -x
      """

  Scenario: a removed directory has no lines
    Given the source state manages:
      | target  | kind   | contents |
      | ~/.gone | remove |          |
    And the home directory contains:
      | target       | kind | contents |
      | ~/.gone/file | file | y        |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.gone b/.gone
      deleted file mode 40755
      index e69de29bb2d1d6434b8b29ae775ad8c2e48c5391..0000000000000000000000000000000000000000
      --- a/.gone
      +++ /dev/null
      """

  Scenario: an entry an exact directory removes
    Given the source state manages:
      | target | kind  | contents |
      | ~/.dir | exact |          |
    And the home directory contains:
      | target       | kind | contents |
      | ~/.dir/extra | file | x        |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.dir/extra b/.dir/extra
      deleted file mode 100644
      index 587be6b4c3f93f93c489c0111bba5596147a26cb..0000000000000000000000000000000000000000
      --- a/.dir/extra
      +++ /dev/null
      @@ -1 +0,0 @@
      -x
      """

  # apply removes the file, then creates the directory: two patches, and the second still
  # compares with the file, because a dry run leaves it in place.
  Scenario: a file replaced by a directory
    Given the source state manages:
      | target     | kind      | contents |
      | ~/.wasfile | directory |          |
    And the home directory contains:
      | target     | kind | contents |
      | ~/.wasfile | file | x        |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.wasfile b/.wasfile
      deleted file mode 100644
      index 587be6b4c3f93f93c489c0111bba5596147a26cb..0000000000000000000000000000000000000000
      --- a/.wasfile
      +++ /dev/null
      @@ -1 +0,0 @@
      -x
      diff --git a/.wasfile b/.wasfile
      old mode 100644
      new mode 40755
      index 587be6b4c3f93f93c489c0111bba5596147a26cb..e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
      --- a/.wasfile
      +++ b/.wasfile
      @@ -1 +0,0 @@
      -x
      """

  Scenario: the mode and the contents change together
    Given the source state manages:
      | target  | kind       | contents |
      | ~/.file | executable | same     |
    And the home directory contains:
      | target  | kind | contents |
      | ~/.file | file | other    |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      old mode 100644
      new mode 100755
      index e45c9c2666d44e0327c1f9c239a74c508336053e..1275430f1765c63e539cb0452565563bd6aef6a6
      --- a/.file
      +++ b/.file
      @@ -1 +1 @@
      -other
      +same
      """

  Scenario: diff --reverse of a new binary file
    Given the source state manages "~/.bin" containing binary data
    When I run "diff --reverse"
    Then it succeeds
    And the output is:
      """
      diff --git a/.bin b/.bin
      deleted file mode 100644
      index 88768efdf77ec78c9a995f94881793be6a41752b..0000000000000000000000000000000000000000
      Binary files a/.bin and /dev/null differ
      """

  Scenario: diff --exclude=files leaves out files
    Given the source state manages:
      | target  | kind      | contents |
      | ~/.dir  | directory |          |
      | ~/.file | file      | # file   |
    When I run "diff --exclude=files"
    Then it succeeds
    And the output is:
      """
      diff --git a/.dir b/.dir
      new file mode 40755
      index 0000000000000000000000000000000000000000..e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
      --- /dev/null
      +++ b/.dir
      """

  Scenario: diff does not need --force
    Given the source state manages:
      | target  | kind | contents |
      | ~/.file | file | x        |
    When I run "diff"
    Then it succeeds
    And "~/.file" does not exist
