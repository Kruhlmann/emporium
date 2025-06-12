use regex::Regex;

lazy_static::lazy_static! {
    pub static ref NUMERIC_REGEX: Regex = Regex::new(r"[-+]?\d*\.?\d+").unwrap();
    pub static ref STATIC_WEAPON_DAMAGE: Regex = Regex::new(r"^your weapons deal \+(\d+) damage\.$").unwrap();
    pub static ref HASTE_N_FOR_M: Regex = Regex::new(r"^haste (\d+) items? for (\d+) second\(s\)\.$").unwrap();
    pub static ref SLOW_N_FOR_M: Regex = Regex::new(r"^slow (\d+) items? for (\d+) second\(s\)\.$").unwrap();
    pub static ref FREEZE_N_FOR_M: Regex = Regex::new(r"^freeze\s+(\d+)\s+item(?:s|\(s\))?\s+for\s+(\d+)\s+second(?:s|\(s\))?\.$").unwrap();
    pub static ref FREEZE_N_FOR_M_OF_SIZE: Regex = Regex::new(r"^freeze\s+(\d+)\s+(small|medium|large)\s+item(?:s|\(s\))?\s+for\s+(\d+)\s+second(?:s|\(s\))?\.$").unwrap();
    pub static ref EFFECT_REDUCE_CD_FLAT: Regex = Regex::new(r"^reduce this item's cooldown by (\d+) second.s. for the fight\.?$").unwrap();
    pub static ref EFFECT_DEAL_DAMAGE_WEIRD: Regex = Regex::new(r"^deal damage (\d+)\.?$").unwrap(); // TODO
    pub static ref EFFECT_GET_ITEMS_REGEX: Regex = Regex::new(r"^get\s+(a|\d+)\s+([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_GET_TAG_CONDITIONAL_ITEMS_REGEX: Regex = Regex::new(r"^get a ([\p{L} ]+). if you have a ([\p{L} ]+), get a second ([\p{L} ]+)\.?").unwrap();
    pub static ref EFFECT_GET_TRIPLE_SINGULAR_ITEMS_REGEX: Regex = Regex::new(r"^get a ([\p{L} ]+), ([\p{L} ]+) and ([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_GAIN_PERMANENT_MAX_HP: Regex = Regex::new(r"^permanently gain (\d+) max health\.?$").unwrap();
    pub static ref EFFECT_SPEND_GOLD_FOR_EFFECT: Regex = Regex::new(r"^spend (\d+) gold to ([\p{L} ]+)\.?$").unwrap();
    pub static ref EFFECT_THIS_GAINS_MAX_AMMO: Regex = Regex::new(r"^this gains (\d+) max ammo\.?$").unwrap();
    pub static ref EFFECT_POISON_SELF: Regex = Regex::new(r"^poison yourself (\d+)\.?$").unwrap();
    pub static ref EFFECT_UPGRADE_RANDOM_PIGGLE: Regex = Regex::new(r"^upgrade a random piggle\.?$").unwrap();
    pub static ref EFFECT_GAIN_GOLD: Regex = Regex::new(r"^gain (\d+) gold\.?$").unwrap();
    pub static ref EFFECT_UPGRADE_LOWER_TIER_TAGGED: Regex = Regex::new(r"^upgrade a ([\p{L} ]+) of a lower tier\.?$").unwrap();
    pub static ref EFFECT_BURN_FROM_DAMAGE: Regex = Regex::new(r"burn equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_HEAL_FROM_DAMAGE: Regex = Regex::new(r"heal equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_HEAL_FROM_DAMAGE_FULL: Regex = Regex::new(r"heal equal to this item's damage.").unwrap();
    pub static ref EFFECT_SHIELD_FROM_DAMAGE: Regex = Regex::new(r"shield equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_SHIELD_FROM_DAMAGE_FULL: Regex = Regex::new(r"shield equal to this item's damage.").unwrap();
    pub static ref EFFECT_POISON_FROM_DAMAGE: Regex = Regex::new(r"poison equal to (\d+)% of this item's damage.").unwrap();
    pub static ref EFFECT_DEAL_DAMAGE: Regex = Regex::new(r"^deal (\d+) damage\.?$").unwrap();
    pub static ref EFFECT_BURN: Regex = Regex::new(r"^burn (\d+)\.?$").unwrap();
    pub static ref EFFECT_POISON: Regex = Regex::new(r"^poison (\d+)\.?$").unwrap();
    pub static ref EFFECT_HEAL: Regex = Regex::new(r"^heal (\d+)\.?$").unwrap();
    pub static ref EFFECT_SHIELD: Regex = Regex::new(r"^shield (\d+)\.?$").unwrap();
    pub static ref EFFECT_REGEN: Regex = Regex::new(r"^gain (\d+) regen for the fight\.?$").unwrap();
}
