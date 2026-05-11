use crate::eggs_config::EggConfig;
use crate::util::test_util::{setup_and_init_test_yolk, TestResult};
use test_log::test;

#[test]
fn test_plan_dependency_tree() -> TestResult {
    let (_home, yolk, _eggs) = setup_and_init_test_yolk()?;

    let mut egg_configs = std::collections::HashMap::new();
    egg_configs.insert("pip".to_string(), EggConfig::default());
    egg_configs.insert(
        "qutebrowser".to_string(),
        EggConfig {
            depends: vec!["pip".to_string()],
            ..EggConfig::default()
        },
    );

    let plan = yolk.plan(&egg_configs)?;

    println!("Actual Plan Output:\n{}", plan);

    // The fixed output should have qutebrowser as root and pip as child.
    // └─ ✓ qutebrowser
    //     └─ ✓ pip

    assert!(plan.contains("└─ ✓ qutebrowser"));
    assert!(plan.contains("    └─ ✓ pip"));
    assert!(!plan.contains("? ✓ (qutebrowser)"));

    Ok(())
}

#[test]
fn test_plan_deep_dependency_tree() -> TestResult {
    let (_home, yolk, _eggs) = setup_and_init_test_yolk()?;

    let mut egg_configs = std::collections::HashMap::new();
    egg_configs.insert("c".to_string(), EggConfig::default());
    egg_configs.insert(
        "b".to_string(),
        EggConfig {
            depends: vec!["c".to_string()],
            ..EggConfig::default()
        },
    );
    egg_configs.insert(
        "a".to_string(),
        EggConfig {
            depends: vec!["b".to_string()],
            ..EggConfig::default()
        },
    );

    let plan = yolk.plan(&egg_configs)?;

    println!("Actual Deep Plan Output:\n{}", plan);

    // a -> b -> c
    // a should be root
    assert!(plan.contains("└─ ✓ a"));
    assert!(plan.contains("    └─ ✓ b"));
    assert!(plan.contains("        └─ ✓ c"));

    Ok(())
}
