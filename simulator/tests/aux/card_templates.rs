use models::v2_0_0::Tier;
use simulator::CardTemplate;

lazy_static::lazy_static! {
    pub static ref FANG_CARD_TEMPLATE: CardTemplate = CardTemplate {
        name: "Fang".to_string(),
        tier: Tier::Bronze,
        modifications: vec![],
    };
    pub static ref BAR_OF_GOLD_CARD_TEMPLATE: CardTemplate = CardTemplate {
        name: "Bar of Gold".to_string(),
        tier: Tier::Bronze,
        modifications: vec![],
    };
}
