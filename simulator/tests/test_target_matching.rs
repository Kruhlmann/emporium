use models::v2_0_0::{PlayerTarget, TargetCondition, Tier};
use rstest::rstest;
use simulator::{Card, CardTemplate};

lazy_static::lazy_static! {
    static ref FANG_CARD_TEMPLATE: CardTemplate = CardTemplate {
        name: "Fang".to_string(),
        tier: Tier::Bronze,
        modifications: vec![],
    };
}

#[rstest]
pub fn test_sanitytarget_ownership() {
    let card: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();
    let other_card: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();
    assert!(card.matches(&TargetCondition::Always, None));
    assert!(!card.matches(&TargetCondition::Never, None));
    assert!(card.matches(&TargetCondition::IsSelf, Some(&card)));
    assert!(!card.matches(&TargetCondition::IsSelf, Some(&other_card)));
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
