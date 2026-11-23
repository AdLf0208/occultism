// Pure Occultism rules (standalone crate lib, injected as
// `crate::game::occultism`). No game access lives here: the spirit-fire recipe
// table as it is matched, the lighters that may strike datura alight, the
// chalk-to-colour table, the two pieces of geometry the click needs, and the
// datura crop's two rules (its ripe harvest and its seed source).
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

/// The crop block the jar registers as `occultism:datura`: a `CropBlock`
/// (`ReplantableCropsBlock`) on the wheat template, with no block item.
pub const DATURA_CROP: &str = "occultism:datura";
/// The crop's seed item, upstream's `ItemNameBlockItem` pointing at the block.
pub const DATURA_SEEDS: &str = "occultism:datura_seeds";
/// The crop's `age` state property, inherited from the wheat template.
pub const AGE: &str = "age";
/// The age a harvest leaves behind (`state.setValue(CropBlock.AGE, 0)`).
pub const HARVEST_AGE: &str = "0";

/// The blocks the jar's two loot modifiers turn into a seed source: `short_grass`
/// and `tall_grass` (`data/occultism/loot_modifiers/datura_seed_from_*.json`).
const SEED_SOURCES: [&str; 2] = ["minecraft:short_grass", "minecraft:tall_grass"];

/// The chance a hand-broken seed source yields one seed: the modifiers'
/// `minecraft:random_chance` of 0.02.
pub const SEED_CHANCE: f32 = 0.02;

/// The shears the modifiers exclude (`#c:tools/shear`): the pin has no item-tag
/// read, and vanilla's shears is the tag's only member at 26.3.
const SHEARS: &str = "minecraft:shears";

/// The age a right click leaves a ripe crop at, or none when the clicked block is
/// not a ripe datura (`ReplantableCropsBlock.useItemOn` then `IReplantableCrops`:
/// `state.getValue(CropBlock.AGE) >= 7`).
pub fn harvest_age(block: &str, age: &str) -> Option<&'static str> {
    (block == DATURA_CROP && age.parse::<u8>().is_ok_and(|age| age >= 7)).then_some(HARVEST_AGE)
}

/// Whether a break of `block` holding `tool` can yield a datura seed: one of the
/// jar's two seed sources, broken by anything but shears.
pub fn seed_source(block: &str, tool: Option<&str>) -> bool {
    SEED_SOURCES.contains(&block) && tool != Some(SHEARS)
}

/// Whether one uniform draw in `[0, 1)` yields a seed
/// (`LootItemRandomChanceCondition`: `random.nextFloat() < 0.02`).
pub fn seed_rolls(draw: f32) -> bool {
    draw < SEED_CHANCE
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

/// The hand a click arrived on.
///
/// Horizon calls `right_click_block` once per hand (the hook's own doc says
/// "once per hand"), where vanilla calls the block's `useItemOn` for the main
/// hand and only tries the off hand when the main hand's action did not consume
/// the click. The crop harvest **changes the block**, so it may run once per
/// click: the off hand's turn would harvest the crop the main hand just
/// replanted, because the hand pair is dispatched from one click. A mirror of the
/// SDK's `Hand`, so this module stays free of `mod_api` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickHand {
    /// The main hand: the click vanilla performs.
    Main,
    /// The off hand: reached only when the main hand's action failed.
    Off,
}

impl ClickHand {
    /// Whether a click on this hand may harvest the crop: the main hand only.
    pub fn may_harvest(self) -> bool {
        matches!(self, Self::Main)
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
    fn only_the_main_hand_harvests_a_crop() {
        // Horizon calls the hook once per hand; vanilla calls `useItemOn` for the
        // main hand and only tries the off hand when the main hand failed. The
        // harvest writes the block, so a second hand's turn harvests the crop the
        // first just replanted: measured in game as 2 fruit and 6 seeds for one
        // click (2026-11-21, phase L).
        assert!(ClickHand::Main.may_harvest(), "the main hand harvests");
        assert!(
            !ClickHand::Off.may_harvest(),
            "the off hand's turn must not harvest the crop the main hand replanted"
        );
    }

    #[test]
    fn a_ripe_datura_is_harvested_and_a_young_one_is_not() {
        // `ReplantableCropsBlock.useItemOn` -> `IReplantableCrops.onHarvest`: only a
        // crop at its last age is harvested; a young one falls through to the game.
        assert_eq!(harvest_age(DATURA_CROP, "7"), Some(HARVEST_AGE));
        for young in ["0", "1", "3", "6"] {
            assert_eq!(harvest_age(DATURA_CROP, young), None, "age {young}");
        }
        // Another block's `age` property is not this crop's.
        assert_eq!(harvest_age("minecraft:wheat", "7"), None);
        assert_eq!(harvest_age("occultism:otherflower", "7"), None);
        // A state the game cannot parse is not ripe.
        assert_eq!(harvest_age(DATURA_CROP, "ripe"), None);
    }

    #[test]
    fn the_harvest_replants_the_crop_at_age_zero() {
        // `level.setBlockAndUpdate(pos, state.setValue(CropBlock.AGE, 0))`.
        assert_eq!(harvest_age(DATURA_CROP, "7"), Some("0"));
        assert_eq!(HARVEST_AGE, "0");
    }

    #[test]
    fn a_hand_break_of_grass_is_a_seed_source() {
        // The jar's two loot modifiers: `short_grass` and `tall_grass`, each at a
        // 0.02 chance, each inverted on `#c:tools/shear`.
        for source in ["minecraft:short_grass", "minecraft:tall_grass"] {
            assert!(seed_source(source, None), "{source} by hand");
            assert!(
                seed_source(source, Some("minecraft:flint_and_steel")),
                "{source} with a lighter"
            );
            assert!(
                !seed_source(source, Some("minecraft:shears")),
                "{source} sheared: the modifier is inverted on shears"
            );
        }
        // Not a seed source: stone, a flower, the crop itself.
        for other in [
            "minecraft:stone",
            "minecraft:grass_block",
            "occultism:datura",
            "minecraft:fern",
        ] {
            assert!(!seed_source(other, None), "{other}");
        }
    }

    #[test]
    fn the_seed_roll_is_upstreams_two_percent() {
        // `LootItemRandomChanceCondition`: `random.nextFloat() < 0.02`.
        assert_eq!(SEED_CHANCE, 0.02);
        for inside in [0.0, 0.0001, 0.019_999] {
            assert!(seed_rolls(inside), "draw {inside} is below the chance");
        }
        for outside in [0.02, 0.5, 0.999_999] {
            assert!(!seed_rolls(outside), "draw {outside} is not below it");
        }
    }

    /// The crafting recipes this port carries, as the package ships them. The
    /// 26.3 loader reads an ingredient as a bare `"id"`, a `"#tag"` or a list of
    /// those strings; a 1.21.1 `{"item": …}` / `{"tag": …}` object is dropped
    /// without a word (the `compactmachines` / `enderio` finding, 2026-11-18),
    /// and a list of such objects is dropped with it.
    const CRAFTING: [(&str, &str); 2] = [
        (
            "demons_dream_essence_from_fruit_or_seed",
            include_str!(
                "../data/occultism/recipe/crafting/demons_dream_essence_from_fruit_or_seed.json"
            ),
        ),
        (
            "book_of_binding_empty",
            include_str!("../data/occultism/recipe/crafting/book_of_binding_empty.json"),
        ),
    ];

    #[test]
    fn the_carried_crafting_recipes_are_in_the_26_3_dialect() {
        // The defect class this pins: the jar's own files are 1.21.1 objects, and
        // carrying them byte for byte makes the game register no recipe at all —
        // the grid then yields nothing, silently.
        for (name, text) in CRAFTING {
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
        }
    }

    #[test]
    fn the_carried_crafting_recipes_name_their_declared_references() {
        // The two recipes whose result and every ingredient this package (or
        // vanilla) already provides. Their references are the slice's contract.
        let (_, essence) = CRAFTING[0];
        for reference in [
            "\"#c:crops/datura\"",
            "\"#c:seeds/datura\"",
            "\"occultism:demons_dream_essence\"",
        ] {
            assert!(
                essence.contains(reference),
                "the essence recipe lost {reference}: {essence}"
            );
        }
        assert!(
            essence.contains("\"minecraft:crafting_shaped\""),
            "the essence recipe is the jar's shaped 3x3: {essence}"
        );
        let (_, book) = CRAFTING[1];
        for reference in [
            "\"occultism:awakened_feather\"",
            "\"occultism:purified_ink\"",
            "\"occultism:taboo_book\"",
            "\"occultism:book_of_binding_empty\"",
        ] {
            assert!(
                book.contains(reference),
                "the book recipe lost {reference}: {book}"
            );
        }
        assert!(
            book.contains("\"minecraft:crafting_shapeless\""),
            "the book recipe is the jar's shapeless one: {book}"
        );
    }

    /// The six `otherstone`/`otherrock` building blocks this slice carries, with
    /// the vanilla template each copies and the count its jar recipe makes.
    const BUILDING: [(&str, &str, u32); 6] = [
        ("otherstone_slab", "minecraft:stone_slab", 6),
        ("otherstone_stairs", "minecraft:stone_stairs", 4),
        ("otherstone_wall", "minecraft:cobblestone_wall", 6),
        ("otherrock_slab", "minecraft:stone_slab", 6),
        ("otherrock_stairs", "minecraft:stone_stairs", 4),
        ("otherrock_wall", "minecraft:cobblestone_wall", 6),
    ];

    /// The port's own content tree, as `cargo test` sees it (the package is the
    /// unit under test: a missing file is a data defect, not a build one).
    fn content(relative: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    #[test]
    fn the_building_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's six `building_blocks/crafting` recipes: three (or six) of the
        // plain block in the middle row make six slabs, the jar's stair step makes
        // four stairs, and two rows of three make six walls. The 26.3 loader reads
        // an ingredient as a bare `"id"`; the jar's `{"item": …}` object is
        // dropped without a word, and the grid then yields nothing (the 0.2.5
        // finding, pinned there for the two carried table recipes).
        for (name, _, count) in BUILDING {
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name} does not make itself: {text}"
            );
            assert!(
                text.contains(&format!("\"count\": {count}")),
                "{name} does not make {count}: {text}"
            );
        }
        // The jar's own patterns, cell by cell.
        let slab = content("occultism/recipe/crafting/otherstone_slab.json");
        assert!(
            slab.contains("\"ooo\""),
            "the slab recipe is one row of three: {slab}"
        );
        let stairs = content("occultism/recipe/crafting/otherrock_stairs.json");
        for row in ["\"#  \"", "\"## \"", "\"###\""] {
            assert!(
                stairs.contains(row),
                "the stair recipe lost {row}: {stairs}"
            );
        }
        let wall = content("occultism/recipe/crafting/otherrock_wall.json");
        assert_eq!(
            wall.matches("\"ooo\"").count(),
            2,
            "the wall recipe is two rows of three: {wall}"
        );
        // Every ingredient is the plain block the slab/stairs/wall is made of.
        for (name, _, _) in BUILDING {
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            let block = if name.starts_with("otherstone") {
                "occultism:otherstone"
            } else {
                "occultism:otherrock"
            };
            assert!(
                text.contains(&format!("\"{block}\"")),
                "{name} is not made of {block}: {text}"
            );
        }
    }

    #[test]
    fn every_building_block_has_a_26_3_loot_table_that_drops_itself() {
        // The rule the jar breaks: `data/occultism/loot_table/blocks/otherrock_slab.json`
        // **does not exist** in Occultism 1.225.0 (the jar carries the other five
        // building tables and not that one), so the block drops nothing at all.
        // The port ships it; this test is the fail-capable pin.
        for (name, _, _) in BUILDING {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{name}'s loot table uses the 1.21.1 {legacy} shape, which the 26.3 \
                     loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                text.contains(&format!("\"name\": \"occultism:{name}\"")),
                "{name}'s loot table does not drop itself: {text}"
            );
        }
        // A slab's table is the jar's: one when single, two when double. The
        // branch is the 26.3 `match_block` condition on the `type=double` state.
        for name in ["otherstone_slab", "otherrock_slab"] {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            assert!(
                text.contains("\"minecraft:match_block\"")
                    && text.contains("\"type\": \"double\"")
                    && text.contains("\"count\": 2"),
                "{name}'s double-slab branch is missing: {text}"
            );
        }
    }

    #[test]
    fn the_building_blocks_copy_the_upstream_stone_templates() {
        // Upstream (`OccultismBlocks`): the slabs are `ofFullCopy(OTHERSTONE)`
        // (hardness 1.5, resistance 30, `requiresCorrectToolForDrops`); the stairs
        // and walls are `Properties.of().strength(1.5f).requiresCorrectToolForDrops()`
        // (hardness and resistance 1.5). Every one of them needs a pickaxe, so a
        // bare-hands break rolls nothing.
        for (name, template, _) in BUILDING {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(&format!("\"template\": \"{template}\"")),
                "{name} does not copy {template}: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": true"),
                "{name} must be gated on a pickaxe as upstream gates it: {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{name} is not hardness 1.5: {text}"
            );
            let resistance = if name.contains("_slab") { 30.0 } else { 1.5 };
            assert!(
                text.contains(&format!("\"explosion_resistance\": {resistance}")),
                "{name} is not resistance {resistance}: {text}"
            );
        }
    }

    #[test]
    fn every_building_block_has_its_state_model_and_name() {
        // A block with no blockstate renders as the missing model, an item with no
        // model is a purple-black cube, and an id absent from `mineable/pickaxe`
        // never drops to a pickaxe; a wall outside `minecraft:walls` does not
        // connect to its neighbours. The tags are also how vanilla recipes find
        // the blocks.
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let walls = content("minecraft/tags/block/walls.json");
        let slabs = content("minecraft/tags/block/slabs.json");
        let stairs = content("minecraft/tags/block/stairs.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        for (name, _, _) in BUILDING {
            let id = format!("occultism:{name}");
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe"
            );
            let tag = if name.ends_with("_wall") {
                &walls
            } else if name.ends_with("_slab") {
                &slabs
            } else {
                &stairs
            };
            assert!(
                tag.contains(&format!("\"{id}\"")),
                "{id} is missing from its tag"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/blockstates/{name}.json"))
                    .is_file(),
                "{id} has no blockstate"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/models/item/{name}.json"))
                    .is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
        }
    }

    /// The twelve blocks of the bricks tier this slice carries, with the vanilla
    /// template each copies and the count its jar recipe makes. The two cracked
    /// blocks make nothing in a grid: their recipe is a furnace one.
    const BRICKS: [(&str, &str, u32); 12] = [
        ("otherstone_bricks", "minecraft:stone", 4),
        ("otherstone_bricks_slab", "minecraft:stone_slab", 6),
        ("otherstone_bricks_stairs", "minecraft:stone_stairs", 4),
        ("otherstone_bricks_wall", "minecraft:cobblestone_wall", 6),
        ("chiseled_otherstone_bricks", "minecraft:stone", 1),
        ("cracked_otherstone_bricks", "minecraft:stone", 0),
        ("otherrock_bricks", "minecraft:stone", 4),
        ("otherrock_bricks_slab", "minecraft:stone_slab", 6),
        ("otherrock_bricks_stairs", "minecraft:stone_stairs", 4),
        ("otherrock_bricks_wall", "minecraft:cobblestone_wall", 6),
        ("chiseled_otherrock_bricks", "minecraft:stone", 1),
        ("cracked_otherrock_bricks", "minecraft:stone", 0),
    ];

    #[test]
    fn the_brick_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's ten bricks-tier grids (`building_blocks/crafting`): a 2x2 of
        // the plain stone makes four bricks, the slabs and the walls come out in
        // sixes, the stair step in fours, and the chiseled pair is two of the
        // brick **slab** stacked (upstream's own recipe, not a bricks 2x2). The
        // 26.3 loader reads an ingredient as a bare `"id"`; the jar's
        // `{"item": …}` object is dropped without a word (the 0.2.5/0.2.6
        // finding), so a byte-for-byte carry yields nothing.
        for (name, _, count) in BRICKS {
            if count == 0 {
                continue; // cracked: smelted, tested below
            }
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name} does not make itself: {text}"
            );
            assert!(
                text.contains(&format!("\"count\": {count}")),
                "{name} does not make {count}: {text}"
            );
        }
        // The ingredient each recipe is made of, as the jar declares it: the
        // bricks are the only ones made of the raw stone; their slabs, stairs
        // and walls are made of the bricks; the chiseled pair of the brick slab.
        for (stone, bricks) in [
            ("otherstone", "otherstone_bricks"),
            ("otherrock", "otherrock_bricks"),
        ] {
            let raw = format!("occultism:{stone}");
            let made = format!("occultism:{bricks}");
            let bricks_recipe = content(&format!("occultism/recipe/crafting/{bricks}.json"));
            assert!(
                bricks_recipe.contains(&format!("\"{raw}\"")),
                "{bricks} is not made of {raw}: {bricks_recipe}"
            );
            assert_eq!(
                bricks_recipe.matches("\"oo\"").count(),
                2,
                "the bricks recipe is the jar's 2x2: {bricks_recipe}"
            );
            let slab = content(&format!("occultism/recipe/crafting/{bricks}_slab.json"));
            assert!(
                slab.contains(&format!("\"{made}\"")) && slab.contains("\"ooo\""),
                "the brick slab recipe is one row of three bricks: {slab}"
            );
            let stairs = content(&format!("occultism/recipe/crafting/{bricks}_stairs.json"));
            for row in ["\"#  \"", "\"## \"", "\"###\""] {
                assert!(
                    stairs.contains(row),
                    "the brick stair recipe lost {row}: {stairs}"
                );
            }
            assert!(
                stairs.contains(&format!("\"{made}\"")),
                "the brick stairs are not made of {made}: {stairs}"
            );
            let wall = content(&format!("occultism/recipe/crafting/{bricks}_wall.json"));
            assert_eq!(
                wall.matches("\"ooo\"").count(),
                2,
                "the brick wall recipe is two rows of three bricks: {wall}"
            );
            let chiseled = content(&format!("occultism/recipe/crafting/chiseled_{bricks}.json"));
            assert!(
                chiseled.contains(&format!("\"occultism:{bricks}_slab\"")),
                "the chiseled recipe is not two brick slabs: {chiseled}"
            );
            assert!(
                chiseled.contains("\"pattern\": [\n    \"o\",\n    \"o\"\n  ]"),
                "the chiseled recipe is two stacked slabs: {chiseled}"
            );
        }
    }

    #[test]
    fn the_cracked_bricks_are_smelted_from_the_bricks() {
        // The jar's only recipe for the cracked pair: a furnace one, 200 ticks,
        // 0.3 experience, from the plain bricks. 26.3 reads the ingredient as a
        // bare string and the result as `{id, count}`.
        for stone in ["otherstone", "otherrock"] {
            let bricks = format!("{stone}_bricks");
            let cracked = format!("cracked_{bricks}");
            let text = content(&format!("occultism/recipe/smelting/{cracked}.json"));
            assert!(
                text.contains("\"type\": \"minecraft:smelting\""),
                "{cracked} is not a furnace recipe: {text}"
            );
            assert!(
                !text.contains("\"item\":") && !text.contains("\"tag\":"),
                "{cracked} carries a 1.21.1 object-form ingredient: {text}"
            );
            assert!(
                text.contains(&format!("\"ingredient\": \"occultism:{bricks}\"")),
                "{cracked} is not smelted from {bricks}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{cracked}\"")),
                "{cracked} does not smelt into itself: {text}"
            );
            assert!(
                text.contains("\"cookingtime\": 200") && text.contains("\"experience\": 0.3"),
                "{cracked} lost the jar's 200 ticks / 0.3 xp: {text}"
            );
        }
    }

    #[test]
    fn every_brick_block_has_a_26_3_loot_table_that_drops_itself() {
        // The rule the jar breaks twice: `otherrock_slab.json` (0.2.6) and
        // `otherrock_bricks_slab.json` (this slice) are the only slab tables the
        // jar does not ship, although both blocks are registered
        // `LootTableType.CUSTOM`, so nothing generates a default. A missing
        // table means the block drops nothing at all; the port ships it.
        for (name, _, _) in BRICKS {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{name}'s loot table uses the 1.21.1 {legacy} shape, which the 26.3 \
                     loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                text.contains(&format!("\"name\": \"occultism:{name}\"")),
                "{name}'s loot table does not drop itself: {text}"
            );
        }
        // A brick slab's table is the jar's: one when single, two when double.
        for name in ["otherstone_bricks_slab", "otherrock_bricks_slab"] {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            assert!(
                text.contains("\"minecraft:match_block\"")
                    && text.contains("\"type\": \"double\"")
                    && text.contains("\"count\": 2"),
                "{name}'s double-slab branch is missing: {text}"
            );
        }
    }

    #[test]
    fn the_brick_blocks_copy_the_upstream_stone_templates() {
        // Upstream (`OccultismBlocks`): the bricks, chiseled and cracked blocks
        // are `ofFullCopy(OTHERSTONE)` (hardness 1.5, resistance 30), the brick
        // slab `ofFullCopy(OTHERSTONE_BRICKS)` (the same 30), the brick stairs
        // and wall `of().strength(1.5f).requiresCorrectToolForDrops()` (1.5).
        // Every one of them needs a pickaxe, so a bare hand rolls nothing.
        for (name, template, _) in BRICKS {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(&format!("\"template\": \"{template}\"")),
                "{name} does not copy {template}: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": true"),
                "{name} must be gated on a pickaxe as upstream gates it: {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{name} is not hardness 1.5: {text}"
            );
            let resistance = if name.ends_with("_stairs") || name.ends_with("_wall") {
                1.5
            } else {
                30.0
            };
            assert!(
                text.contains(&format!("\"explosion_resistance\": {resistance}")),
                "{name} is not resistance {resistance}: {text}"
            );
        }
    }

    #[test]
    fn every_brick_block_has_its_state_model_and_name() {
        // A block with no blockstate renders as the missing model, an item with
        // no model is a purple-black cube, and an id absent from
        // `mineable/pickaxe` never drops to a pickaxe; a wall outside
        // `minecraft:walls` does not connect to its neighbours.
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let walls = content("minecraft/tags/block/walls.json");
        let slabs = content("minecraft/tags/block/slabs.json");
        let stairs = content("minecraft/tags/block/stairs.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        for (name, _, _) in BRICKS {
            let id = format!("occultism:{name}");
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe"
            );
            let tag = if name.ends_with("_wall") {
                &walls
            } else if name.ends_with("_slab") {
                &slabs
            } else if name.ends_with("_stairs") {
                &stairs
            } else {
                // The plain, chiseled and cracked blocks are in no shape tag.
                &pickaxe
            };
            assert!(
                tag.contains(&format!("\"{id}\"")),
                "{id} is missing from its tag"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/blockstates/{name}.json"))
                    .is_file(),
                "{id} has no blockstate"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/models/item/{name}.json"))
                    .is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
        }
    }

    /// The eight blocks of the polished tier this slice carries, with the vanilla
    /// template each copies and the count its jar recipe makes. The two plain
    /// blocks make nothing in a grid: their recipe is a furnace one.
    const POLISHED: [(&str, &str, u32); 8] = [
        ("polished_otherstone", "minecraft:stone", 0),
        ("polished_otherstone_slab", "minecraft:stone_slab", 6),
        ("polished_otherstone_stairs", "minecraft:stone_stairs", 4),
        ("polished_otherstone_wall", "minecraft:cobblestone_wall", 6),
        ("polished_otherrock", "minecraft:stone", 0),
        ("polished_otherrock_slab", "minecraft:stone_slab", 6),
        ("polished_otherrock_stairs", "minecraft:stone_stairs", 4),
        ("polished_otherrock_wall", "minecraft:cobblestone_wall", 6),
    ];

    #[test]
    fn the_polished_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's six polished-tier grids (`building_blocks/crafting`): three
        // polished blocks in the middle row make six slabs, the stair step makes
        // four stairs and two rows of three make six walls. The 26.3 loader reads
        // an ingredient as a bare `"id"`; the jar's `{"item": …}` object is
        // dropped without a word (the 0.2.5/0.2.6/0.2.7 finding), so a
        // byte-for-byte carry yields nothing.
        for (name, _, count) in POLISHED {
            if count == 0 {
                continue; // the plain pair: smelted, tested below
            }
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name} does not make itself: {text}"
            );
            assert!(
                text.contains(&format!("\"count\": {count}")),
                "{name} does not make {count}: {text}"
            );
        }
        // Each shape is made of the **polished** block, never of the raw stone:
        // the raw id, quoted, would be the un-polished recipe's ingredient.
        for stone in ["otherstone", "otherrock"] {
            let polished = format!("occultism:polished_{stone}");
            let raw = format!("\"occultism:{stone}\"");
            let slab = content(&format!(
                "occultism/recipe/crafting/polished_{stone}_slab.json"
            ));
            assert!(
                slab.contains(&format!("\"{polished}\"")) && slab.contains("\"ooo\""),
                "the polished slab recipe is one row of three polished blocks: {slab}"
            );
            assert!(
                !slab.contains(&raw),
                "the polished slab recipe names the raw stone: {slab}"
            );
            let stairs = content(&format!(
                "occultism/recipe/crafting/polished_{stone}_stairs.json"
            ));
            for row in ["\"#  \"", "\"## \"", "\"###\""] {
                assert!(
                    stairs.contains(row),
                    "the polished stair recipe lost {row}: {stairs}"
                );
            }
            assert!(
                stairs.contains(&format!("\"{polished}\"")),
                "the polished stairs are not made of {polished}: {stairs}"
            );
            let wall = content(&format!(
                "occultism/recipe/crafting/polished_{stone}_wall.json"
            ));
            assert_eq!(
                wall.matches("\"ooo\"").count(),
                2,
                "the polished wall recipe is two rows of three: {wall}"
            );
            assert!(
                wall.contains(&format!("\"{polished}\"")),
                "the polished wall is not made of {polished}: {wall}"
            );
        }
    }

    #[test]
    fn the_polished_pair_is_smelted_from_the_raw_stones() {
        // The jar's only recipe for the plain polished pair: a furnace one, 200
        // ticks, 0.5 experience, from the raw stone (0.3 for the cracked bricks
        // is the neighbouring tier's figure — this is its own). 26.3 reads the
        // ingredient as a bare string and the result as `{id, count}`.
        for stone in ["otherstone", "otherrock"] {
            let polished = format!("polished_{stone}");
            let text = content(&format!("occultism/recipe/smelting/{polished}.json"));
            assert!(
                text.contains("\"type\": \"minecraft:smelting\""),
                "{polished} is not a furnace recipe: {text}"
            );
            assert!(
                !text.contains("\"item\":") && !text.contains("\"tag\":"),
                "{polished} carries a 1.21.1 object-form ingredient: {text}"
            );
            assert!(
                text.contains(&format!("\"ingredient\": \"occultism:{stone}\"")),
                "{polished} is not smelted from {stone}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{polished}\"")),
                "{polished} does not smelt into itself: {text}"
            );
            assert!(
                text.contains("\"cookingtime\": 200") && text.contains("\"experience\": 0.5"),
                "{polished} lost the jar's 200 ticks / 0.5 xp: {text}"
            );
        }
    }

    #[test]
    fn every_polished_block_has_a_26_3_loot_table_that_drops_itself() {
        // The rule the jar breaks a third time: `polished_otherrock_slab.json` is
        // the only polished table the jar does not ship, although the block is
        // registered `LootTableType.CUSTOM`, so nothing generates a default. A
        // missing table means the block drops nothing at all; the port ships it.
        for (name, _, _) in POLISHED {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{name}'s loot table uses the 1.21.1 {legacy} shape, which the 26.3 \
                     loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                text.contains(&format!("\"name\": \"occultism:{name}\"")),
                "{name}'s loot table does not drop itself: {text}"
            );
        }
        // A polished slab's table is the jar's: one when single, two when double.
        for name in ["polished_otherstone_slab", "polished_otherrock_slab"] {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            assert!(
                text.contains("\"minecraft:match_block\"")
                    && text.contains("\"type\": \"double\"")
                    && text.contains("\"count\": 2"),
                "{name}'s double-slab branch is missing: {text}"
            );
        }
    }

    #[test]
    fn the_polished_blocks_copy_the_upstream_stone_templates() {
        // Upstream (`OccultismBlocks.java:181-233`): the plain pair and the slabs
        // are `ofFullCopy(OTHERSTONE)`/`ofFullCopy(POLISHED_*)` (hardness 1.5,
        // resistance 30), the stairs and walls
        // `of().strength(1.5f).requiresCorrectToolForDrops()` (1.5). Every one of
        // them needs a pickaxe, so a bare hand rolls nothing.
        for (name, template, _) in POLISHED {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(&format!("\"template\": \"{template}\"")),
                "{name} does not copy {template}: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": true"),
                "{name} must be gated on a pickaxe as upstream gates it: {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{name} is not hardness 1.5: {text}"
            );
            let resistance = if name.ends_with("_stairs") || name.ends_with("_wall") {
                1.5
            } else {
                30.0
            };
            assert!(
                text.contains(&format!("\"explosion_resistance\": {resistance}")),
                "{name} is not resistance {resistance}: {text}"
            );
        }
    }

    #[test]
    fn every_polished_block_has_its_state_model_and_name() {
        // A block with no blockstate renders as the missing model, an item with
        // no model is a purple-black cube, and an id absent from
        // `mineable/pickaxe` never drops to a pickaxe; a wall outside
        // `minecraft:walls` does not connect to its neighbours.
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let walls = content("minecraft/tags/block/walls.json");
        let slabs = content("minecraft/tags/block/slabs.json");
        let stairs = content("minecraft/tags/block/stairs.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        for (name, _, _) in POLISHED {
            let id = format!("occultism:{name}");
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe"
            );
            let tag = if name.ends_with("_wall") {
                &walls
            } else if name.ends_with("_slab") {
                &slabs
            } else if name.ends_with("_stairs") {
                &stairs
            } else {
                // The plain pair is in no shape tag.
                &pickaxe
            };
            assert!(
                tag.contains(&format!("\"{id}\"")),
                "{id} is missing from its tag"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/blockstates/{name}.json"))
                    .is_file(),
                "{id} has no blockstate"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/models/item/{name}.json"))
                    .is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
        }
    }

    /// The eight `othercobblestone`/`othercobblerock` blocks this slice carries,
    /// with the vanilla template each copies and the count its jar recipe makes.
    /// The two plain blocks make nothing in a grid: upstream has no recipe for
    /// them — a player gets them by breaking `otherstone`/`otherrock` without
    /// silk touch (the table tested below), and smelts them back into the stone.
    const COBBLE: [(&str, &str, u32); 8] = [
        ("othercobblestone", "minecraft:stone", 0),
        ("othercobblestone_slab", "minecraft:stone_slab", 6),
        ("othercobblestone_stairs", "minecraft:stone_stairs", 4),
        ("othercobblestone_wall", "minecraft:cobblestone_wall", 6),
        ("othercobblerock", "minecraft:stone", 0),
        ("othercobblerock_slab", "minecraft:stone_slab", 6),
        ("othercobblerock_stairs", "minecraft:stone_stairs", 4),
        ("othercobblerock_wall", "minecraft:cobblestone_wall", 6),
    ];

    #[test]
    fn the_cobble_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's six cobble-tier grids (`building_blocks/crafting`): three
        // cobble blocks in the middle row make six slabs, the stair step makes
        // four stairs and two rows of three make six walls. The 26.3 loader reads
        // an ingredient as a bare `"id"`; the jar's `{"item": …}` object is
        // dropped without a word (the 0.2.5–0.2.8 finding), so a byte-for-byte
        // carry yields nothing.
        for (name, _, count) in COBBLE {
            if count == 0 {
                continue; // the plain pair: no grid upstream
            }
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name} does not make itself: {text}"
            );
            assert!(
                text.contains(&format!("\"count\": {count}")),
                "{name} does not make {count}: {text}"
            );
        }
        // Each shape is made of the **cobble** block, never of the smooth stone:
        // a copy of the previous tier's recipe would name the wrong ingredient.
        for stone in ["othercobblestone", "othercobblerock"] {
            let smooth = format!("\"occultism:{}\"", stone.replace("cobble", ""));
            for suffix in ["_slab", "_stairs", "_wall"] {
                let text = content(&format!("occultism/recipe/crafting/{stone}{suffix}.json"));
                assert!(
                    text.contains(&format!("\"occultism:{stone}\"")),
                    "{stone}{suffix} is not made of {stone}: {text}"
                );
                assert!(
                    !text.contains(&smooth),
                    "{stone}{suffix} names the smooth stone: {text}"
                );
            }
            let slab = content(&format!("occultism/recipe/crafting/{stone}_slab.json"));
            assert!(
                slab.contains("\"ooo\""),
                "the {stone} slab recipe is one row of three: {slab}"
            );
            let stairs = content(&format!("occultism/recipe/crafting/{stone}_stairs.json"));
            for row in ["\"#  \"", "\"## \"", "\"###\""] {
                assert!(
                    stairs.contains(row),
                    "the {stone} stair recipe lost {row}: {stairs}"
                );
            }
            let wall = content(&format!("occultism/recipe/crafting/{stone}_wall.json"));
            assert_eq!(
                wall.matches("\"ooo\"").count(),
                2,
                "the {stone} wall recipe is two rows of three: {wall}"
            );
        }
    }

    #[test]
    fn every_cobble_block_has_a_26_3_loot_table_that_drops_itself() {
        // The rule the jar breaks a fourth time: `othercobblerock_slab.json` is
        // absent from Occultism 1.225.0 although the block is registered
        // `LootTableType.CUSTOM`, so nothing generates a default and the block
        // would drop nothing at all. The jar ships every other cobble table
        // (`othercobblestone_slab` included) and all four of the otherrock
        // family's slabs are the ones it omits. The port ships the missing one.
        for (name, _, _) in COBBLE {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{name}'s loot table uses the 1.21.1 {legacy} shape, which the 26.3 \
                     loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                text.contains(&format!("\"name\": \"occultism:{name}\"")),
                "{name}'s loot table does not drop itself: {text}"
            );
        }
        // A cobble slab's table is the jar's: one when single, two when double.
        for name in ["othercobblestone_slab", "othercobblerock_slab"] {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            assert!(
                text.contains("\"minecraft:match_block\"")
                    && text.contains("\"type\": \"double\"")
                    && text.contains("\"count\": 2"),
                "{name}'s double-slab branch is missing: {text}"
            );
        }
    }

    #[test]
    fn the_raw_stones_drop_the_cobble_tier_without_silk_touch() {
        // Upstream's `otherstone`/`otherrock` table (the jar's own file, carried
        // here): a silk-touch pickaxe drops the stone itself and any other break
        // the cobble tier. 0.2.4 shipped a simplified table (the block always
        // dropped itself) because this tier was not carried; now it is, and the
        // deviation's reason is gone. The engine reads `match_tool` with the
        // `minecraft:enchantments` predicate (`horizon-items/src/items.rs:1508`),
        // which is the branch's proof in game. The jar's condition objects spell
        // their kind `"condition": "minecraft:…"`; the 26.3 loader reads `"type"`
        // and drops the other silently (an unknown kind is false, so a verbatim
        // carry would drop nothing at all), exactly the ingredient-shape trap.
        for (stone, cobble) in [
            ("otherstone", "othercobblestone"),
            ("otherrock", "othercobblerock"),
        ] {
            let text = content(&format!("occultism/loot_table/blocks/{stone}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{stone}'s loot table uses the 1.21.1 {legacy} shape: {text}"
                );
            }
            assert!(
                !text.contains("\"condition\": \"minecraft:"),
                "{stone} carries the 1.21.1 string-form condition kind, which the \
                 26.3 loader does not read (it reads `type`): {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:alternatives\""),
                "{stone} has no silk-touch alternative: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:match_tool\"")
                    && text.contains("\"minecraft:silk_touch\""),
                "{stone} does not keep itself for a silk-touch tool: {text}"
            );
            assert!(
                text.contains(&format!("\"name\": \"occultism:{stone}\"")),
                "{stone} does not drop itself under silk touch: {text}"
            );
            assert!(
                text.contains(&format!("\"name\": \"occultism:{cobble}\"")),
                "{stone} does not drop {cobble} without silk touch: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:survives_explosion\""),
                "{stone} lost the jar's explosion guard: {text}"
            );
        }
    }

    #[test]
    fn the_cobble_blocks_copy_the_upstream_stone_templates() {
        // Upstream (`OccultismBlocks.java`): `OTHERCOBBLESTONE`/`OTHERCOBBLEROCK`
        // are `of().mapColor(STONE).sound(STONE).strength(1.5f, 30)
        // .requiresCorrectToolForDrops()`, their slabs `ofFullCopy(OTHERSTONE)` /
        // `ofFullCopy(OTHERROCK)` (1.5 / 30), and the stairs and walls
        // `of().strength(1.5f).requiresCorrectToolForDrops()` (1.5 / 1.5). Every
        // one needs a pickaxe, so a bare hand rolls nothing.
        for (name, template, _) in COBBLE {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(&format!("\"template\": \"{template}\"")),
                "{name} does not copy {template}: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": true"),
                "{name} must be gated on a pickaxe as upstream gates it: {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{name} is not hardness 1.5: {text}"
            );
            let resistance = if name.ends_with("_stairs") || name.ends_with("_wall") {
                1.5
            } else {
                30.0
            };
            assert!(
                text.contains(&format!("\"explosion_resistance\": {resistance}")),
                "{name} is not resistance {resistance}: {text}"
            );
        }
    }

    #[test]
    fn every_cobble_block_has_its_state_model_and_name() {
        // A block with no blockstate renders as the missing model, an item with
        // no model is a purple-black cube, an id absent from
        // `mineable/pickaxe` never drops to a pickaxe, and a wall outside
        // `minecraft:walls` does not connect to its neighbours.
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let walls = content("minecraft/tags/block/walls.json");
        let slabs = content("minecraft/tags/block/slabs.json");
        let stairs = content("minecraft/tags/block/stairs.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        for (name, _, _) in COBBLE {
            let id = format!("occultism:{name}");
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe"
            );
            let tag = if name.ends_with("_wall") {
                &walls
            } else if name.ends_with("_slab") {
                &slabs
            } else if name.ends_with("_stairs") {
                &stairs
            } else {
                // The plain pair is in no shape tag.
                &pickaxe
            };
            assert!(
                tag.contains(&format!("\"{id}\"")),
                "{id} is missing from its tag"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/blockstates/{name}.json"))
                    .is_file(),
                "{id} has no blockstate"
            );
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("assets/occultism/models/item/{name}.json"))
                    .is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
        }
    }

    /// The four `otherstone`/`otherrock` redstone blocks this slice carries, with
    /// the vanilla template each copies. Upstream (`OccultismBlocks.java`):
    /// `otherstone_button`/`otherrock_button` are
    /// `new ButtonBlock(BlockSetType.STONE, 30, Properties.of().noCollission()
    /// .strength(0.5F).pushReaction(DESTROY))`, and the two pressure plates are
    /// `new PressurePlateBlock(BlockSetType.STONE, Properties.ofFullCopy(
    /// Blocks.STONE_PRESSURE_PLATE))`. `Blocks.STONE_BUTTON` and
    /// `Blocks.STONE_PRESSURE_PLATE` are the same two properties at the pin
    /// (`gameplay-facts.json`: hardness 0.5, `requires_tool` **false**), so the
    /// templates are the exact vanilla pair and neither block is
    /// `requiresCorrectToolForDrops` — a bare hand drops one.
    const REDSTONE: [(&str, &str); 4] = [
        ("otherstone_button", "minecraft:stone_button"),
        (
            "otherstone_pressure_plate",
            "minecraft:stone_pressure_plate",
        ),
        ("otherrock_button", "minecraft:stone_button"),
        ("otherrock_pressure_plate", "minecraft:stone_pressure_plate"),
    ];

    /// The stone each redstone block's jar grid takes (`otherstone` for the two
    /// `otherstone` blocks, `otherrock` for the two `otherrock` ones).
    fn redstone_stone(name: &str) -> &'static str {
        if name.starts_with("otherrock") {
            "otherrock"
        } else {
            "otherstone"
        }
    }

    #[test]
    fn the_redstone_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's four grids (`recipe/crafting/{otherstone,otherrock}_{button,
        // pressure_plate}.json`): one stone makes one button (shapeless), two
        // stones in one row make one pressure plate (shaped `oo`). The 26.3
        // loader reads an ingredient as a bare `"id"`; the jar's `{"item": …}`
        // object is dropped without a word and the grid then yields nothing.
        for (name, _) in REDSTONE {
            let stone = redstone_stone(name);
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{name} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name} does not make itself: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{name} does not make one: {text}"
            );
            assert!(
                text.contains(&format!("\"occultism:{stone}\"")),
                "{name} is not made of occultism:{stone}: {text}"
            );
            // The cobble tier is a different block: a copy of the 0.2.9 recipe
            // would name it and craft nothing a player could get.
            let cobble = match stone {
                "otherrock" => "occultism:othercobblerock",
                _ => "occultism:othercobblestone",
            };
            assert!(
                !text.contains(cobble),
                "{name} names the cobble block {cobble}: {text}"
            );
            if name.ends_with("_button") {
                assert!(
                    text.contains("\"minecraft:crafting_shapeless\""),
                    "{name} is the jar's shapeless one-stone grid: {text}"
                );
            } else {
                assert!(
                    text.contains("\"minecraft:crafting_shaped\""),
                    "{name} is the jar's shaped `oo` grid: {text}"
                );
                assert_eq!(
                    text.matches("\"oo\"").count(),
                    1,
                    "the {name} recipe is one row of two: {text}"
                );
            }
        }
    }

    #[test]
    fn every_redstone_block_has_a_26_3_loot_table_that_drops_itself() {
        // The jar ships all four tables for this family (unlike the four
        // `otherrock`-family slabs the earlier slices had to add), but they spell
        // their guard `"condition": "minecraft:survives_explosion"` — the 1.21.1
        // string form, which the 26.3 loader reads as an unknown kind and so as
        // false: a verbatim carry drops **nothing at all**. The port's tables use
        // the `explosion_decay` modifier, as every carried table does.
        for (name, _) in REDSTONE {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{name}'s loot table uses the 1.21.1 {legacy} shape, which the 26.3 \
                     loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                !text.contains("\"condition\": \"minecraft:"),
                "{name} carries the 1.21.1 string-form condition kind, which the 26.3 \
                 loader does not read (it reads `type`): {text}"
            );
            assert!(
                text.contains(&format!("\"name\": \"occultism:{name}\"")),
                "{name}'s loot table does not drop itself: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:survives_explosion\""),
                "{name} lost the jar's survives_explosion guard: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:block\""),
                "{name}'s loot table is not a block table: {text}"
            );
        }
    }

    #[test]
    fn the_redstone_blocks_copy_the_upstream_templates() {
        // Upstream's four declarations: both buttons are `Properties.of()
        // .noCollission().strength(0.5F)` (hardness 0.5, resistance 0.5, map
        // colour NONE because `of()` leaves it unset) and both plates are a full
        // copy of `Blocks.STONE_PRESSURE_PLATE` (1.21.1: hardness 0.5, resistance
        // 0.5, map colour STONE). None calls `requiresCorrectToolForDrops`, so
        // `requires_tool` is false: a bare hand drops one. The brief's trap line
        // claims hand 0 / pickaxe 1 for this family; the artefact says otherwise
        // and the artefact wins (recorded in notes.md).
        for (name, template) in REDSTONE {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(&format!("\"template\": \"{template}\"")),
                "{name} does not copy {template}: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": false"),
                "{name} must not gate its drops on a pickaxe (upstream has no \
                 requiresCorrectToolForDrops): {text}"
            );
            assert!(
                text.contains("\"hardness\": 0.5"),
                "{name} is not hardness 0.5: {text}"
            );
            assert!(
                text.contains("\"explosion_resistance\": 0.5"),
                "{name} is not resistance 0.5: {text}"
            );
            let colour = if name.ends_with("_pressure_plate") {
                11 // MapColor.STONE, the full copy of Blocks.STONE_PRESSURE_PLATE
            } else {
                0 // MapColor.NONE, `Properties.of()` for the buttons
            };
            assert!(
                text.contains(&format!("\"map_color\": {colour}")),
                "{name} is not map colour {colour}: {text}"
            );
        }
    }

    #[test]
    fn every_redstone_block_has_its_state_model_and_name() {
        // The jar's tags: the two plates are in `mineable/pickaxe` (the buttons
        // are not — vanilla's stone button is not either), the buttons are in
        // `stone_buttons` (block and item) and the plates in
        // `stone_pressure_plates`. A button needs its up and pressed model, a
        // plate its up and down model, or the pressed state renders as the
        // missing model.
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let buttons = content("minecraft/tags/block/stone_buttons.json");
        let plates = content("minecraft/tags/block/stone_pressure_plates.json");
        let button_items = content("minecraft/tags/item/stone_buttons.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/occultism");
        for (name, _) in REDSTONE {
            let id = format!("occultism:{name}");
            if name.ends_with("_pressure_plate") {
                assert!(
                    pickaxe.contains(&format!("\"{id}\"")),
                    "{id} is not in mineable/pickaxe"
                );
                assert!(
                    plates.contains(&format!("\"{id}\"")),
                    "{id} is not in stone_pressure_plates"
                );
                for model in ["", "_down"] {
                    assert!(
                        assets
                            .join(format!("models/block/{name}{model}.json"))
                            .is_file(),
                        "{id} has no block model {model}"
                    );
                }
            } else {
                assert!(
                    !pickaxe.contains(&format!("\"{id}\"")),
                    "{id} must not be in mineable/pickaxe (the jar's tag omits buttons)"
                );
                assert!(
                    buttons.contains(&format!("\"{id}\"")),
                    "{id} is not in stone_buttons"
                );
                assert!(
                    button_items.contains(&format!("\"{id}\"")),
                    "{id} is not in the item tag stone_buttons"
                );
                for model in ["", "_pressed"] {
                    assert!(
                        assets
                            .join(format!("models/block/{name}{model}.json"))
                            .is_file(),
                        "{id} has no block model {model}"
                    );
                }
            }
            assert!(
                assets.join(format!("blockstates/{name}.json")).is_file(),
                "{id} has no blockstate"
            );
            assert!(
                assets.join(format!("models/item/{name}.json")).is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
        }
    }

    #[test]
    fn the_cobble_smelts_back_into_the_stone() {
        // The jar's reverse pair (`recipe/smelting/{otherstone,otherrock}`): the
        // cobble block smelts back into the stone, 200 ticks, 0.5 experience.
        // Both endpoints are carried, so the pair completes the tier's loop; the
        // 26.3 loader drops the jar's `{"item": …}` ingredient object.
        for (stone, cobble) in [
            ("otherstone", "othercobblestone"),
            ("otherrock", "othercobblerock"),
        ] {
            let text = content(&format!("occultism/recipe/smelting/{stone}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{stone}'s smelting recipe carries the 1.21.1 object-form ingredient \
                     {object_form}: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:smelting\""),
                "{stone} is not a smelting recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"occultism:{cobble}\"")),
                "{stone} is not smelted from {cobble}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{stone}\"")),
                "{stone}'s smelting recipe does not make it: {text}"
            );
            assert!(
                text.contains("\"cookingtime\": 200") && text.contains("\"experience\": 0.5"),
                "{stone} is not the jar's 200 ticks / 0.5 xp: {text}"
            );
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
