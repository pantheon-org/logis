# Seeded by `cargo xtask txtar2feature` from chezmoi internal/cmd/testdata/scripts/diff.txtar,
# then specified by hand. Expected output is chezmoi's golden output for unix with umask 022
# (the adapter fixes the umask). Windows output differs in file modes, so this feature is @unix.
@unix
Feature: diff
  diff prints, as a git-style diff, the changes apply would make to the home directory.

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
    And I have run "apply --force"

  Scenario: diff generates no output when the source and destination states are equal
    When I run "diff"
    Then it succeeds
    And the output is empty

  Scenario: diff generates a diff when a file is added to the source state
    Given the source state manages:
      | target     | kind | contents              |
      | ~/.newfile | file | # contents of .newfile |
    When I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.newfile b/.newfile
      new file mode 100644
      index 0000000000000000000000000000000000000000..06e05235fdd12fd5c367b6d629fef94536c85525
      --- /dev/null
      +++ b/.newfile
      @@ -0,0 +1 @@
      +# contents of .newfile
      """

  Scenario: diff generates a diff when a file is edited
    When I append "# edited" to "~/.file"
    And I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index 5d2730a8850a2db479af83de87cc8345437aef06..8a52cb9ce9551221716a53786ad74104c5902362 100644
      --- a/.file
      +++ b/.file
      @@ -1,2 +1 @@
       # contents of .file
      -# edited
      """

  Scenario: diff --reverse
    When I append "# edited" to "~/.file"
    And I run "diff --reverse"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      index 8a52cb9ce9551221716a53786ad74104c5902362..5d2730a8850a2db479af83de87cc8345437aef06 100644
      --- a/.file
      +++ b/.file
      @@ -1 +1,2 @@
       # contents of .file
      +# edited
      """

  Scenario: diff generates a diff when a file is removed from the destination directory
    When I remove "~/.file"
    And I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      new file mode 100644
      index 0000000000000000000000000000000000000000..8a52cb9ce9551221716a53786ad74104c5902362
      --- /dev/null
      +++ b/.file
      @@ -0,0 +1 @@
      +# contents of .file
      """

  Scenario: diff generates a diff when a directory is removed from the destination directory
    When I remove "~/.dir"
    And I run "diff --recursive=false ~/.dir"
    Then it succeeds
    And the output is:
      """
      diff --git a/.dir b/.dir
      new file mode 40755
      index 0000000000000000000000000000000000000000..e69de29bb2d1d6434b8b29ae775ad8c2e48c5391
      --- /dev/null
      +++ b/.dir
      """

  Scenario: diff generates a diff when the actual state is a file and the target state is a symlink
    When I replace "~/.symlink" with a file containing "# contents of .file"
    And I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.symlink b/.symlink
      deleted file mode 100644
      index 8a52cb9ce9551221716a53786ad74104c5902362..0000000000000000000000000000000000000000
      --- a/.symlink
      +++ /dev/null
      @@ -1 +0,0 @@
      -# contents of .file
      diff --git a/.symlink b/.symlink
      old mode 100644
      new mode 120000
      index 8a52cb9ce9551221716a53786ad74104c5902362..9b91fdbb83798a67fbbc5cc4f120c3f7726c0d70
      --- a/.symlink
      +++ b/.symlink
      @@ -1 +1 @@
      -# contents of .file
      +.dir/subdir/file
      """

  Scenario: diff generates a diff when a file's permissions are changed
    When I change the mode of "~/.file" to 777
    And I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.file b/.file
      old mode 100777
      new mode 100644
      """

  Scenario: diff generates a diff when a directory's permissions are changed
    When I change the mode of "~/.dir" to 700
    And I run "diff"
    Then it succeeds
    And the output is:
      """
      diff --git a/.dir b/.dir
      old mode 40700
      new mode 40755
      """
