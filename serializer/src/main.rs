use gamedata::v2_0_0::CardFang;
use models::v2_0_0::Card;

// pub struct WikiData {
//     content: String,
//     identity: String,
// }

// impl Into<WikiData> for Card {
//     fn into(self) -> WikiData {
//         format!(
//             r#"{{{{Item
// |title          = {{{{PAGENAME}}}}
// |image          = {{{{PAGENAME}}}}.png
// |cooldown       = ..
// |ammo           = ..
// |effects        = ..
// |cost           = ..
// |type           = ..
// |size           = ..
// |starting_tier  = ..
// |collection     = ..
// }}}}"#
//         )
//     }
// }

fn main() {
    let fang = CardFang::new();
}
