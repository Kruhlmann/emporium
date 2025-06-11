mod aux;

use aux::FANG_CARD_TEMPLATE;
use models::v2_0_0::{PlayerTarget, TargetCondition};
use simulator::Card;

#[rstest::rstest]
pub fn test_sanity() {
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
