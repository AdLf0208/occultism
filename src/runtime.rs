// Pure Occultism slice-1 rules (standalone crate lib, injected as
// `crate::game::occultism`). No game access lives here: the spirit-fire recipe
// table as it is matched, the lighters that may strike datura alight, the
// chalk-to-colour table, and the two pieces of geometry the click needs.
//
// Upstream (`SpiritFireBlock`, `SpiritFireRecipe`, `PlayerEventHandler.spiritFire`,
// read from Occultism 1.225.0 / tag `release/v1.21.1-1.225.0`):
//   * an item entity inside the fire whose stack matches a `occultism:spirit_fire`
//     recipe is replaced by the recipe's result with
//     `result.setCount(input.getCount())` — the result's own count is ignored;
//   * flint and steel (or a fire charge) struck on a block while a
//     `#occultism:start_spiritfire` item entity (datura) lies within
//     `AABB(-1,-1,-1,1,1,1)` of the block's centre consumes the datura and puts
//     the fire at the clicked block when it can be replaced, else on the clicked
//     face's neighbour;
//   * a chalk right-clicked on the fire sets the fire's `color` state.

/// The mod's own block id, as the carried content declares it.
pub const SPIRIT_FIRE: &str = "occultism:spirit_fire";
/// The one item the jar's `#occultism:start_spiritfire` tag holds.
pub const DATURA: &str = "occultism:datura";
/// The recipe type the carried `data/occultism/recipe/spirit_fire/` files name.
pub const RECIPE_TYPE: &str = "occultism:spirit_fire";

/// Items that may strike the datura alight, by id. Upstream also accepts any
/// item with NeoForge's `light_fire`/`light_campfire` ability; the pin has no
/// item-ability read, so a modded lighter is a recorded gap.
const LIGHTERS: [&str; 2] = ["minecraft:flint_and_steel", "minecraft:fire_charge"];

/// One `occultism:spirit_fire` recipe after its ingredient's tags were expanded:
/// the item ids it accepts and the item id it yields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub inputs: Vec<String>,
    pub result: String,
}

/// What one conversion yields. Its count is always the converted stack's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversion {
    pub result: String,
    pub count: u32,
}

/// The loaded recipe table, in recipe-load order (upstream's `getRecipeFor`
/// takes the first match).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    rules: Vec<Rule>,
}

impl Table {
    /// Keeps the rules that can match: one with no resolved input is a tag with
    /// no known member and would otherwise match every item.
    pub fn new(rules: Vec<Rule>) -> Self {
        let rules = rules
            .into_iter()
            .filter(|rule| !rule.inputs.is_empty() && !rule.result.is_empty())
            .map(|mut rule| {
                let mut seen = Vec::new();
                rule.inputs.retain(|input| {
                    let fresh = !seen.contains(input);
                    if fresh {
                        seen.push(input.clone());
                    }
                    fresh
                });
                rule
            })
            .collect();
        Self { rules }
    }

    /// The rules that were kept, in load order.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// The distinct item ids any rule accepts (`Ingredient` members).
    pub fn inputs(&self) -> usize {
        let mut seen: Vec<&str> = Vec::new();
        for rule in &self.rules {
            for input in &rule.inputs {
                if !seen.contains(&input.as_str()) {
                    seen.push(input);
                }
            }
        }
        seen.len()
    }

    /// Whether any rule accepts `item`.
    pub fn accepts(&self, item: &str) -> bool {
        self.matching(item).is_some()
    }

    /// The conversion for `count` of `item`, or none when no rule accepts it.
    /// The result keeps the stack's count, as upstream's `assemble` sets it.
    pub fn convert(&self, item: &str, count: u32) -> Option<Conversion> {
        if count == 0 {
            return None;
        }
        let rule = self.matching(item)?;
        Some(Conversion {
            result: rule.result.clone(),
            count,
        })
    }

    /// The first rule whose ingredient accepts `item` (`getRecipeFor`'s order).
    fn matching(&self, item: &str) -> Option<&Rule> {
        self.rules
            .iter()
            .find(|rule| rule.inputs.iter().any(|input| input == item))
    }
}

/// One ingredient as a 1.21.1 recipe file names it (`{"item": …}` or
/// `{"tag": …}`), reduced to the reference `native_recipe_ingredient` takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientRef {
    /// A single item id.
    ItemId(String),
    /// A tag, held without its `#`.
    Tag(String),
}

impl IngredientRef {
    /// An item ingredient, `{"item": "minecraft:diorite"}`.
    pub fn item(id: &str) -> Self {
        Self::ItemId(id.to_owned())
    }

    /// A tag ingredient, `{"tag": "c:feathers"}`; a leading `#` is not repeated.
    pub fn tag(tag: &str) -> Self {
        Self::Tag(tag.trim_start_matches('#').to_owned())
    }

    /// The reference the game's `native_recipe_ingredient` parses: a bare id, or
    /// `#namespace:path` for a tag.
    pub fn reference(&self) -> String {
        match self {
            Self::ItemId(id) => id.clone(),
            Self::Tag(tag) => format!("#{tag}"),
        }
    }
}

/// Whether `item` is a lighter that may strike datura alight.
pub fn starts_fire(item: &str) -> bool {
    LIGHTERS.contains(&item)
}

/// The `color` state value a chalk item sets, or none for anything else.
/// Upstream's `useItemOn` chain, item by item; `chalk_gold` is the yellow one.
pub fn chalk_color(item: &str) -> Option<&'static str> {
    const CHALKS: [(&str, &str); 18] = [
        ("occultism:chalk_white", "white"),
        ("occultism:chalk_light_gray", "light_gray"),
        ("occultism:chalk_gray", "gray"),
        ("occultism:chalk_black", "black"),
        ("occultism:chalk_brown", "brown"),
        ("occultism:chalk_red", "red"),
        ("occultism:chalk_orange", "orange"),
        ("occultism:chalk_gold", "yellow"),
        ("occultism:chalk_lime", "lime"),
        ("occultism:chalk_green", "green"),
        ("occultism:chalk_cyan", "cyan"),
        ("occultism:chalk_light_blue", "light_blue"),
        ("occultism:chalk_blue", "blue"),
        ("occultism:chalk_purple", "purple"),
        ("occultism:chalk_magenta", "magenta"),
        ("occultism:chalk_pink", "pink"),
        ("occultism:chalk_rainbow", "rainbow"),
        ("occultism:chalk_void", "void"),
    ];
    CHALKS
        .iter()
        .find_map(|(id, colour)| (*id == item).then_some(*colour))
}

/// Where the fire goes: the clicked block itself when it can be replaced,
/// otherwise the block on the clicked face.
pub fn fire_target(pos: [i32; 3], face: [i32; 3], clicked_replaceable: bool) -> [i32; 3] {
    if clicked_replaceable {
        return pos;
    }
    [
        pos[0].saturating_add(face[0]),
        pos[1].saturating_add(face[1]),
        pos[2].saturating_add(face[2]),
    ]
}

/// Upstream's datura search box: `AABB(-1,-1,-1,1,1,1)` around the clicked
/// block's centre, as (min, max) corners.
pub fn datura_box(pos: [i32; 3]) -> ([f64; 3], [f64; 3]) {
    let centre = [
        f64::from(pos[0]) + 0.5,
        f64::from(pos[1]) + 0.5,
        f64::from(pos[2]) + 0.5,
    ];
    (
        [centre[0] - 1.0, centre[1] - 1.0, centre[2] - 1.0],
        [centre[0] + 1.0, centre[1] + 1.0, centre[2] + 1.0],
    )
}

/// How many recent conversions the readout keeps. The state is diagnostic
/// only: it is bounded so a long-lived server cannot grow it per fire.
pub const RECENT_LIMIT: usize = 256;

/// The operator level `/occultism info` requires. The readout names the block at
/// coordinates the caller chooses, so it is a gamemaster tool (level 2), as
/// `/data` is — not a player command.
pub const INFO_LEVEL: u8 = 2;

/// The refusal a source below [`INFO_LEVEL`] gets. It names no block.
pub const INFO_DENIED: &str =
    "occultism info: operator permission is required to read a spirit fire";

/// The answer for a position that holds no spirit fire. It names no block: the
/// readout must not double as a world probe at arbitrary coordinates.
pub const NOT_A_FIRE: &str = "not a spirit fire";

/// Whether a command source at `level` may read a fire. The usage is registered
/// at [`INFO_LEVEL`] *and* the body asks again, so a source that reaches the
/// body by another route (a function, a future dispatcher change) is refused
/// too.
pub fn info_permitted(level: u8) -> bool {
    level >= INFO_LEVEL
}

/// The readout for `pos`: a spirit fire's state, or [`NOT_A_FIRE`] for any other
/// block — which is why the answer can never name the block it found.
pub fn info_line(
    pos: [i32; 3],
    block: &str,
    colour: &str,
    last: &str,
    rules: usize,
    files: usize,
) -> String {
    if block != SPIRIT_FIRE {
        return NOT_A_FIRE.to_owned();
    }
    format!(
        "Spirit Fire at {pos:?}: block={block}, colour={colour}, last={last}, rules={rules}/{files}"
    )
}

/// A player's game mode, as the click guard needs it.
///
/// A mirror of the game's four modes rather than the SDK's `GameMode`: this
/// module carries no `mod_api` type, so the glue maps one to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Survival.
    Survival,
    /// Creative.
    Creative,
    /// Adventure.
    Adventure,
    /// Spectator: Horizon calls `right_click_block` for this one too.
    Spectator,
}

impl Mode {
    /// Whether a click from a player in this mode may change the world. A
    /// spectator changes nothing and loses nothing.
    pub fn may_change(self) -> bool {
        !matches!(self, Self::Spectator)
    }
}

/// One conversion, keyed by its world, dimension and position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentConversion {
    /// The open of the world the fire was in.
    pub world: u64,
    /// The dimension's id (`minecraft:overworld`).
    pub dimension: String,
    /// The fire's position.
    pub pos: [i32; 3],
    /// The item the conversion produced.
    pub result: String,
    /// How many of it.
    pub count: u32,
}

/// The bounded ring of recent conversions [`RecentConversion`] describes.
/// A fire in another dimension, or in an earlier open of the world at the same
/// coordinates, is not the same fire.
#[derive(Debug, Default)]
pub struct Recent {
    entries: std::collections::VecDeque<RecentConversion>,
}

impl Recent {
    /// Remember one conversion, dropping the oldest once [`RECENT_LIMIT`] is
    /// reached.
    pub fn record(&mut self, conversion: RecentConversion) {
        self.entries.push_back(conversion);
        while self.entries.len() > RECENT_LIMIT {
            self.entries.pop_front();
        }
    }

    /// The newest conversion at `pos` in `dimension` of `world`, if this ring
    /// still holds one.
    #[must_use]
    pub fn last(&self, world: u64, dimension: &str, pos: [i32; 3]) -> Option<&RecentConversion> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.world == world && entry.dimension == dimension && entry.pos == pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(inputs: &[&str], result: &str) -> Rule {
        Rule {
            inputs: inputs.iter().map(|s| (*s).to_owned()).collect(),
            result: result.to_owned(),
        }
    }

    fn feather_table() -> Table {
        Table::new(vec![rule(
            &["minecraft:feather"],
            "occultism:awakened_feather",
        )])
    }

    #[test]
    fn converts_whole_stacks() {
        let table = feather_table();
        for count in [1, 4, 64] {
            assert_eq!(
                table.convert("minecraft:feather", count),
                Some(Conversion {
                    result: "occultism:awakened_feather".to_owned(),
                    count,
                }),
                "count {count}"
            );
        }
    }

    #[test]
    fn unknown_item_does_not_convert() {
        assert_eq!(feather_table().convert("minecraft:dirt", 1), None);
    }

    #[test]
    fn zero_count_is_not_a_conversion() {
        assert_eq!(feather_table().convert("minecraft:feather", 0), None);
    }

    #[test]
    fn empty_ingredient_is_not_a_rule() {
        let table = Table::new(vec![rule(&[], "occultism:otherworld_ashes")]);
        assert_eq!(table.rules().len(), 0);
        assert!(!table.accepts("minecraft:oak_log"));
        assert_eq!(table.convert("minecraft:oak_log", 1), None);
    }

    #[test]
    fn first_rule_wins() {
        let table = Table::new(vec![
            rule(&["minecraft:book"], "occultism:taboo_book"),
            rule(&["minecraft:book"], "occultism:book_of_binding_empty"),
        ]);
        assert_eq!(
            table.convert("minecraft:book", 1).map(|c| c.result),
            Some("occultism:taboo_book".to_owned())
        );
    }

    #[test]
    fn matches_by_exact_id() {
        let table = Table::new(vec![rule(
            &["occultism:chalk_white_impure"],
            "occultism:chalk_white",
        )]);
        assert!(table.accepts("occultism:chalk_white_impure"));
        assert!(!table.accepts("occultism:chalk_white"));
        assert!(!table.accepts("occultism:chalk_white_impure_extra"));
    }

    #[test]
    fn accepts_every_tag_member() {
        let table = Table::new(vec![rule(
            &[
                "minecraft:writable_book",
                "minecraft:book",
                "occultism:taboo_book",
            ],
            "occultism:book_of_binding_empty",
        )]);
        for item in [
            "minecraft:writable_book",
            "minecraft:book",
            "occultism:taboo_book",
        ] {
            assert!(table.accepts(item), "{item}");
        }
        assert_eq!(table.inputs(), 3);
    }

    #[test]
    fn a_block_result_is_an_item_id() {
        let table = Table::new(vec![rule(&["minecraft:andesite"], "occultism:otherstone")]);
        assert_eq!(
            table.convert("minecraft:andesite", 2),
            Some(Conversion {
                result: "occultism:otherstone".to_owned(),
                count: 2,
            })
        );
    }

    #[test]
    fn an_item_ingredient_is_a_bare_id() {
        assert_eq!(
            IngredientRef::item("minecraft:diorite").reference(),
            "minecraft:diorite"
        );
    }

    #[test]
    fn a_tag_ingredient_is_hash_prefixed() {
        assert_eq!(IngredientRef::tag("c:feathers").reference(), "#c:feathers");
    }

    #[test]
    fn a_tag_keeps_one_hash() {
        assert_eq!(
            IngredientRef::tag("#occultism:books/books_for_empty").reference(),
            "#occultism:books/books_for_empty"
        );
    }

    #[test]
    fn lighters_are_flint_and_steel_and_fire_charge() {
        assert!(starts_fire("minecraft:flint_and_steel"));
        assert!(starts_fire("minecraft:fire_charge"));
        assert!(!starts_fire("minecraft:flint"));
        assert!(!starts_fire(DATURA));
        assert!(!starts_fire("occultism:chalk_white"));
    }

    #[test]
    fn chalk_items_name_their_colour() {
        let cases = [
            ("chalk_white", "white"),
            ("chalk_light_gray", "light_gray"),
            ("chalk_gray", "gray"),
            ("chalk_black", "black"),
            ("chalk_brown", "brown"),
            ("chalk_red", "red"),
            ("chalk_orange", "orange"),
            ("chalk_gold", "yellow"),
            ("chalk_lime", "lime"),
            ("chalk_green", "green"),
            ("chalk_cyan", "cyan"),
            ("chalk_light_blue", "light_blue"),
            ("chalk_blue", "blue"),
            ("chalk_purple", "purple"),
            ("chalk_magenta", "magenta"),
            ("chalk_pink", "pink"),
            ("chalk_rainbow", "rainbow"),
            ("chalk_void", "void"),
        ];
        for (name, colour) in cases {
            assert_eq!(
                chalk_color(&format!("occultism:{name}")),
                Some(colour),
                "{name}"
            );
        }
    }

    #[test]
    fn chalk_gold_is_yellow() {
        assert_eq!(chalk_color("occultism:chalk_gold"), Some("yellow"));
    }

    #[test]
    fn impure_chalk_is_not_chalk_paint() {
        assert_eq!(chalk_color("occultism:chalk_white_impure"), None);
        assert_eq!(chalk_color("minecraft:bone_meal"), None);
    }

    #[test]
    fn fire_target_stays_on_a_replaceable_block() {
        assert_eq!(fire_target([10, 64, -3], [0, 1, 0], true), [10, 64, -3]);
    }

    #[test]
    fn fire_target_moves_onto_the_clicked_face() {
        assert_eq!(fire_target([10, 64, -3], [0, 1, 0], false), [10, 65, -3]);
        assert_eq!(fire_target([10, 64, -3], [-1, 0, 0], false), [9, 64, -3]);
        assert_eq!(fire_target([10, 64, -3], [0, 0, 1], false), [10, 64, -2]);
    }

    #[test]
    fn datura_box_matches_upstream() {
        assert_eq!(
            datura_box([10, 64, -3]),
            ([9.5, 63.5, -3.5], [11.5, 65.5, -1.5])
        );
    }

    #[test]
    fn datura_box_excludes_two_blocks_away() {
        let (min, max) = datura_box([0, 0, 0]);
        // An entity two blocks east of the block's centre is outside.
        assert!(2.5 > max[0]);
        assert!(-2.5 < min[0]);
    }

    fn conversion(world: u64, dimension: &str, pos: [i32; 3], result: &str) -> RecentConversion {
        RecentConversion {
            world,
            dimension: dimension.to_owned(),
            pos,
            result: result.to_owned(),
            count: 1,
        }
    }

    #[test]
    fn recent_conversions_are_bounded() {
        let mut recent = Recent::default();
        for index in 0..(RECENT_LIMIT as i32 + 10) {
            recent.record(conversion(1, "minecraft:overworld", [index, 64, 0], "a"));
        }
        assert_eq!(recent.entries.len(), RECENT_LIMIT);
        assert!(
            recent.last(1, "minecraft:overworld", [0, 64, 0]).is_none(),
            "the oldest conversion was dropped"
        );
        assert!(
            recent
                .last(1, "minecraft:overworld", [RECENT_LIMIT as i32 + 9, 64, 0])
                .is_some(),
            "the newest conversion is kept"
        );
    }

    #[test]
    fn the_info_readout_needs_an_operator() {
        // `/occultism info` names the block at coordinates the caller chooses,
        // so a player below level 2 must be refused (the reviewer's finding: a
        // non-operator could find hidden ores or containers).
        assert!(!info_permitted(0), "a plain player");
        assert!(!info_permitted(1), "a moderator");
        assert!(info_permitted(INFO_LEVEL), "a gamemaster");
        assert!(info_permitted(4), "the console");
    }

    #[test]
    fn a_non_fire_readout_names_no_block() {
        // A position holding no spirit fire must not echo what is there: that
        // would turn the readout into a world probe even for an operator.
        let line = info_line([10, 64, 0], "minecraft:diamond_ore", "none", "none", 26, 27);
        assert_eq!(line, NOT_A_FIRE);
        assert!(
            !line.contains(':'),
            "the answer must not carry a block id: {line}"
        );
        // A fire's own line is unchanged, so the existing e2e assertions hold.
        assert_eq!(
            info_line(
                [10, 64, 0],
                SPIRIT_FIRE,
                "yellow",
                "occultism:awakened_feather x3",
                26,
                27
            ),
            "Spirit Fire at [10, 64, 0]: block=occultism:spirit_fire, colour=yellow, \
             last=occultism:awakened_feather x3, rules=26/27"
        );
    }

    #[test]
    fn a_spectator_click_changes_nothing() {
        // Horizon calls `right_click_block` for a spectator too (the hook's own
        // doc says so), so the click path must ask before it writes anything.
        assert!(
            !Mode::Spectator.may_change(),
            "a spectator must not recolour, light or consume anything"
        );
        for mode in [Mode::Survival, Mode::Creative, Mode::Adventure] {
            assert!(mode.may_change(), "{mode:?} may still use the loop");
        }
    }

    #[test]
    fn a_conversion_is_scoped_to_its_world_and_dimension() {
        let mut recent = Recent::default();
        recent.record(conversion(1, "minecraft:overworld", [0, 64, 0], "old"));
        recent.record(conversion(2, "minecraft:overworld", [0, 64, 0], "new"));
        recent.record(conversion(2, "minecraft:the_nether", [0, 64, 0], "nether"));
        assert_eq!(
            recent
                .last(1, "minecraft:overworld", [0, 64, 0])
                .unwrap()
                .result,
            "old"
        );
        assert_eq!(
            recent
                .last(2, "minecraft:overworld", [0, 64, 0])
                .unwrap()
                .result,
            "new"
        );
        assert_eq!(
            recent
                .last(2, "minecraft:the_nether", [0, 64, 0])
                .unwrap()
                .result,
            "nether"
        );
        assert!(recent.last(2, "minecraft:overworld", [1, 64, 0]).is_none());
    }
}
