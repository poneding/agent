use ruse_cli::parse_cli_invocation;

#[test]
fn daily_driver_recipe_agent_cli_commands_are_parseable() {
    let docs = format!(
        "{}\n{}",
        include_str!("../../../docs/cli/daily-driver-recipes.md"),
        include_str!("../../../docs/cli/recovery-workflows.md")
    );
    let mut parsed = 0_usize;

    for line in docs.lines().map(str::trim) {
        let Some(command) = line.strip_prefix("ruse-cli ") else {
            continue;
        };
        let args = command.split_whitespace().collect::<Vec<_>>();
        parse_cli_invocation(args).unwrap_or_else(|error| {
            panic!("recipe command should parse: ruse-cli {command}: {error}")
        });
        parsed += 1;
    }

    assert!(
        parsed >= 10,
        "expected recipe coverage for daily-driver commands"
    );
    assert!(docs.contains("cargo xtask eval"));
    assert!(!docs.contains("sk-"));
    assert!(!docs.contains("api_key"));
}
