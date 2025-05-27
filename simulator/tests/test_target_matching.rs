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
pub fn test_target_ownership() {
    let player_fang: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Player, Default::default())
        .unwrap();

    let opponent_fang: Card = FANG_CARD_TEMPLATE
        .create_card_on_board(0, PlayerTarget::Opponent, Default::default())
        .unwrap();

    assert!(player_fang.matches(
        &TargetCondition::HasOwner(PlayerTarget::Player),
        Some(&player_fang)
    ));
    assert!(player_fang.matches(
        &TargetCondition::HasOwner(PlayerTarget::Opponent),
        Some(&opponent_fang)
    ));
    assert!(opponent_fang.matches(
        &TargetCondition::HasOwner(PlayerTarget::Opponent),
        Some(&player_fang)
    ));
    assert!(opponent_fang.matches(
        &TargetCondition::HasOwner(PlayerTarget::Player),
        Some(&opponent_fang)
    ));
}
