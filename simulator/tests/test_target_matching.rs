mod aux;

use aux::{BAR_OF_GOLD_CARD_TEMPLATE, FANG_CARD_TEMPLATE};
use models::v2_0_0::{PlayerTarget, TargetCondition};
use rstest::rstest;
use simulator::Card;

#[rstest]
pub fn test_boolean_operations() {
    let card: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();
    assert!(card.matches(&(TargetCondition::Never | TargetCondition::Always), None));
    assert!(!card.matches(&(TargetCondition::Never & TargetCondition::Always), None));
    assert!(card.matches(&(!TargetCondition::Never), None));
}

#[rstest]
pub fn test_target_ownership() {
    let player_card: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();
    let opponent_card: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Opponent, Default::default())
        .unwrap();

    assert!(player_card.matches(
        &TargetCondition::HasOwner(PlayerTarget::Player),
        Some(&player_card)
    ));
    assert!(player_card.matches(
        &TargetCondition::HasOwner(PlayerTarget::Opponent),
        Some(&opponent_card)
    ));
    assert!(opponent_card.matches(
        &TargetCondition::HasOwner(PlayerTarget::Opponent),
        Some(&player_card)
    ));
    assert!(opponent_card.matches(
        &TargetCondition::HasOwner(PlayerTarget::Player),
        Some(&opponent_card)
    ));
}

#[rstest]
pub fn test_target_cooldown() {
    let card_with_cooldown: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();
    let card_without_cooldown: Card = BAR_OF_GOLD_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();

    assert!(card_with_cooldown.matches(&TargetCondition::HasCooldown, None));
    assert!(!card_without_cooldown.matches(&TargetCondition::HasCooldown, None));
}
