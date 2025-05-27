use crate::{GlobalCardId, TICKS_PER_SECOND};

use super::{PlayerHealth, PlayerTemplate};

#[derive(Clone, Debug)]
pub struct Player {
    pub health: PlayerHealth,
    pub shield: i64,
    pub poison_stacks: i64,
    pub burn_stacks: i64,
    pub regeneration_stacks: i64,
    pub card_ids: Vec<GlobalCardId>,
    pub template: PlayerTemplate,
    pub dot_counter: usize,
}

impl Player {
    pub fn burn(&mut self, amount: u32) {
        self.burn_stacks += amount as i64
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health -= amount as i64
    }

    pub fn poison(&mut self, amount: u32) {
        self.poison_stacks += amount as i64
    }

    pub fn shield(&mut self, amount: u32) {
        self.shield += amount as i64
        // TODO reduce poison and/or burn by 10%
    }

    // TODO: This is bad.. increase the tickrate and filter every other tick for cards.
    pub fn burn_tick(&mut self) {
        if self.burn_stacks == 0 {
            return;
        }

        let base_damage = self.burn_stacks as f64;
        let damage = if self.shield > 0 {
            (base_damage * 0.5).round() as i64
        } else {
            base_damage.round() as i64
        };

        if self.shield > 0 {
            let absorb = damage.min(self.shield);
            self.shield -= absorb;
            let leftover = damage - absorb;
            self.health -= leftover;
        } else {
            self.health -= damage;
        }

        self.burn_stacks -= 1;
    }

    pub fn tick(&mut self) {
        if self.dot_counter % *TICKS_PER_SECOND == 0 {
            // TODO Half-ticks, Half-measure (burn)
            for _ in 0..2 {
                self.burn_tick();
            }
            self.health = self.health + self.regeneration_stacks - self.poison_stacks;
        }
        self.dot_counter += 1;
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cards = self
            .template
            .card_templates
            .iter()
            .map(|c| format!("{} ({:?})", c.name, c.tier))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "Player(❤️ {}/{}, 🔥:{}, 🧪:{}, 🌱:{}, 🛡️:{}) [{cards}]",
            self.health.current(),
            self.health.max(),
            self.burn_stacks,
            self.poison_stacks,
            self.regeneration_stacks,
            self.shield,
        )
    }
}
