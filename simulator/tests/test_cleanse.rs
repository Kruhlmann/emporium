use rstest::rstest;
use simulator::PlayerTemplate;

/// Based on:
/// Hotfix: June 6, 2025
/// Heal Cleanse
///   * Reduced to 5% of the amount Healed
/// https://playthebazaar-cdn.azureedge.net/thebazaar/PatchNotes.html
#[rstest]
fn test_cleanse() -> Result<(), Box<dyn std::error::Error>> {
    let mut player = PlayerTemplate {
        health: 100,
        regen: 0,
        card_templates: vec![],
        skill_templates: vec![],
    }
    .create_player(vec![])?;
    player.burn_stacks = 100;
    player.poison_stacks = 100;
    player.heal(100);
    assert_eq!(player.burn_stacks, 95);
    assert_eq!(player.poison_stacks, 95);

    player.burn_stacks = 0;
    player.poison_stacks = 0;
    player.heal(100);
    assert_eq!(player.burn_stacks, 0);
    assert_eq!(player.poison_stacks, 0);
    Ok(())
}
