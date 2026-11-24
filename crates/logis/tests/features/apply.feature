Feature: apply makes the home directory match the source directory

  # Seeded from internal/cmd/testdata/scripts/apply.txtar in the chezmoi fork.

  @wip
  Scenario: apply with an empty source directory changes nothing
    Given an empty home directory
    When I run "apply --force"
    Then it succeeds
    And the home directory is still empty
