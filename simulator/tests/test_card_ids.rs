use models::v2_0_0::{PlayerTarget, Tier};
use simulator::{CardTemplate, PlayerTemplate, Simulation, SimulationTemplate};

lazy_static::lazy_static! {
    static ref FANG_CARD_TEMPLATE: CardTemplate = CardTemplate {
        name: "Fang".to_string(),
        tier: Tier::Bronze,
        modifications: vec![],
    };
}

#[test]
fn simulation_assigns_card_ids_to_correct_players() {
    let template = SimulationTemplate {
        player: PlayerTemplate {
            health: 20,
            regen: 0,
            card_templates: vec![FANG_CARD_TEMPLATE.clone()],
            skill_templates: vec![],
        },
        opponent: PlayerTemplate {
            health: 20,
            regen: 0,
            card_templates: vec![FANG_CARD_TEMPLATE.clone()],
            skill_templates: vec![],
        },
        seed: None,
    };

    let sim: Simulation = template.try_into().expect("simulation should build");
    assert_eq!(sim.cards.len(), 2);
    assert_eq!(sim.player.card_ids.len(), 1);
    assert_eq!(sim.opponent.card_ids.len(), 1);
    assert_ne!(sim.player.card_ids[0], sim.opponent.card_ids[0]);

    let player_card = sim.cards.get(&sim.player.card_ids[0]).unwrap();
    assert_eq!(player_card.owner, PlayerTarget::Player);

    let opponent_card = sim.cards.get(&sim.opponent.card_ids[0]).unwrap();
    assert_eq!(opponent_card.owner, PlayerTarget::Opponent);
}
