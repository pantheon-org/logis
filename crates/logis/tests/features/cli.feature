Feature: The command-line entry point

  Scenario: --version names the tool and succeeds
    Given an empty home directory
    When I run "--version"
    Then it succeeds
    And the output names the tool
