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
