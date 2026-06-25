#[test]
fn skill_discovery_loads_valid_skill_root() {
    let temp = tempfile::tempdir().unwrap();
    let skill_dir = temp.path().join("code-review");
    std::fs::create_dir(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Code Review\ndescription: Review code changes\nversion: 0.1.0\n---\nCheck diffs carefully.\n",
    )
    .unwrap();

    let skills = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: skill_dir.clone(),
            enabled: true,
        }],
    })
    .unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].manifest.id.as_str(), "code-review");
    assert_eq!(skills[0].manifest.name, "Code Review");
    assert_eq!(skills[0].source.root_path, skill_dir);
    assert!(skills[0].prompt.content.contains("Check diffs carefully."));
}

#[test]
fn skill_discovery_ignores_disabled_root() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");

    let skills = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: missing,
            enabled: false,
        }],
    })
    .unwrap();

    assert!(skills.is_empty());
}

#[test]
fn skill_discovery_rejects_invalid_manifest() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("SKILL.md"),
        "---\nname: Broken\n---\nBody\n",
    )
    .unwrap();

    let err = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: temp.path().to_path_buf(),
            enabled: true,
        }],
    })
    .unwrap_err();

    assert!(matches!(err, agent_skill::SkillError::InvalidManifest(_)));
}

#[test]
fn skill_discovery_rejects_missing_enabled_root() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");

    let err = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: missing.clone(),
            enabled: true,
        }],
    })
    .unwrap_err();

    assert_eq!(err, agent_skill::SkillError::RootNotFound(missing));
}

#[test]
fn skill_discovery_rejects_missing_manifest() {
    let temp = tempfile::tempdir().unwrap();

    let err = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: temp.path().to_path_buf(),
            enabled: true,
        }],
    })
    .unwrap_err();

    assert_eq!(
        err,
        agent_skill::SkillError::ManifestNotFound(temp.path().join("SKILL.md"))
    );
}

#[test]
fn skill_discovery_rejects_empty_prompt_body() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("SKILL.md"),
        "---\nname: Empty\ndescription: Empty prompt\n---\n   \n",
    )
    .unwrap();

    let err = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![agent_skill::SkillRoot {
            path: temp.path().to_path_buf(),
            enabled: true,
        }],
    })
    .unwrap_err();

    assert!(matches!(err, agent_skill::SkillError::InvalidPrompt(_)));
}

#[test]
fn skill_discovery_returns_deterministic_path_order() {
    let temp = tempfile::tempdir().unwrap();
    let z_dir = temp.path().join("z-skill");
    let a_dir = temp.path().join("a-skill");
    std::fs::create_dir(&z_dir).unwrap();
    std::fs::create_dir(&a_dir).unwrap();
    std::fs::write(
        z_dir.join("SKILL.md"),
        "---\nname: Z Skill\ndescription: Last skill\n---\nZ body\n",
    )
    .unwrap();
    std::fs::write(
        a_dir.join("SKILL.md"),
        "---\nname: A Skill\ndescription: First skill\n---\nA body\n",
    )
    .unwrap();

    let skills = agent_skill::discover_skills(&agent_skill::SkillDiscoveryConfig {
        roots: vec![
            agent_skill::SkillRoot {
                path: z_dir,
                enabled: true,
            },
            agent_skill::SkillRoot {
                path: a_dir,
                enabled: true,
            },
        ],
    })
    .unwrap();

    let ids: Vec<_> = skills
        .iter()
        .map(|skill| skill.manifest.id.as_str())
        .collect();
    assert_eq!(ids, vec!["a-skill", "z-skill"]);
}

#[test]
fn skill_discovery_converts_prompt_to_context_layer() {
    let contribution = agent_skill::SkillPromptContribution {
        skill_id: agent_skill::SkillId::new("code-review").unwrap(),
        layer_id: "skill:code-review".to_string(),
        content: "Check diffs carefully.".to_string(),
        priority: 600,
        token_estimate: 4,
        source_path: std::path::PathBuf::from("/tmp/SKILL.md"),
    };

    let layer = agent_skill::skill_prompt_layer(&contribution);

    assert_eq!(layer.id, "skill:code-review");
    assert_eq!(layer.kind, agent_context::PromptLayerKind::Skill);
    assert_eq!(layer.priority, 600);
    assert_eq!(layer.content, "Check diffs carefully.");
}
