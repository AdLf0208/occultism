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

/// The six sacrificial bowls this slice carries. Upstream registers ten
/// (`OccultismBlocks.java:462-515`): the four `SacrificialBowlBlock`s and the two
/// `GoldenSacrificialBowlBlock`s, plus the silver pair (whose crafting grid needs
/// `#c:ingots/silver`, which this package has no member for) and the two iesnium
/// bowls (pentacle-ritual results, `RitualRecipes.craft_iesnium_sacrificial_bowl`).
/// The four uncarriable ones are recorded as gaps, not shipped as creative-only
/// blocks.
pub const BOWLS: [&str; 6] = [
    "occultism:sacrificial_bowl",
    "occultism:dark_sacrificial_bowl",
    "occultism:copper_sacrificial_bowl",
    "occultism:dark_copper_sacrificial_bowl",
    "occultism:golden_sacrificial_bowl",
    "occultism:dark_golden_sacrificial_bowl",
];

/// The two golden bowls: upstream's `GoldenSacrificialBowlBlock` is a plain
/// `Block`, not a `SacrificialBowlBlock`, and its block entity overrides the
/// contents change only to start a ritual — the spirit-fire conversion is skipped
/// by an `instanceof GoldenSacrificialBowlBlockEntity` check.
pub const GOLDEN_BOWLS: [&str; 2] = [
    "occultism:golden_sacrificial_bowl",
    "occultism:dark_golden_sacrificial_bowl",
];

/// The bowl's one slot holds one item:
/// `SacrificialBowlBlockEntity.itemStackHandler.getSlotLimit → 1`.
pub const BOWL_SLOT_LIMIT: u32 = 1;

/// The most `extractItem(0, 64, false)` takes out of the bowl
/// (`SacrificialBowlBlock.useItemOn`), which is the whole slot unless a hopper put
/// more in — the engine's declared container has no per-slot cap.
pub const BOWL_EXTRACT_LIMIT: u32 = 64;

/// The one item the jar excludes from the bowl's over-fire conversion
/// (`!recipeInput.item().is(OccultismBlocks.OTHERFLOWER.asItem())`): the flower's
/// own spirit-fire recipe would otherwise burn it in a bowl.
pub const OTHERFLOWER: &str = "occultism:otherflower";

/// Whether `block` is one of the six bowls this slice carries.
pub fn is_bowl(block: &str) -> bool {
    BOWLS.contains(&block)
}

/// Whether `block` is one of the two golden bowls.
pub fn is_golden_bowl(block: &str) -> bool {
    GOLDEN_BOWLS.contains(&block)
}

/// Whether `block` is a bowl whose contents its block entity converts while it
/// stands over a spirit fire: every bowl but the golden pair
/// (`SacrificialBowlBlockEntity.onContentsChanged` skips `GoldenSacrificialBowlBlockEntity`).
pub fn converts_in_bowl(block: &str) -> bool {
    is_bowl(block) && !is_golden_bowl(block)
}

/// Whether `item` may be converted by a bowl: anything but the flower the jar
/// excludes.
pub fn bowl_converts_item(item: &str) -> bool {
    item != OTHERFLOWER
}

/// What one right click puts in an empty bowl and what it leaves in the hand:
/// `insertItem(0, held, false)` with a slot limit of one. `None` when the hand
/// holds nothing.
pub fn bowl_insert(held: u32) -> Option<(u32, u32)> {
    (held > 0).then(|| (BOWL_SLOT_LIMIT.min(held), held - BOWL_SLOT_LIMIT.min(held)))
}

/// How much a take-out click removes from the bowl's slot:
/// `extractItem(0, 64, false)`.
pub fn bowl_take(stored: u32) -> u32 {
    stored.min(BOWL_EXTRACT_LIMIT)
}

/// What a plain bowl turns a stored stack into while it stands over a spirit
/// fire: upstream's `onContentsChanged` runs the stack through the loaded
/// `occultism:spirit_fire` table, unless the bowl is golden or the item is the
/// flower the jar excludes. The bowl's slot holds one item, so the count is one.
pub fn bowl_conversion(table: &Table, block: &str, item: &str) -> Option<Conversion> {
    if !converts_in_bowl(block) || !bowl_converts_item(item) {
        return None;
    }
    table.convert(item, 1)
}

/// The `/occultism info` answer for a bowl: the stack its block entity holds, as
/// read through the block entity's own slot.
pub fn bowl_line(pos: [i32; 3], stored: Option<(&str, u32)>) -> String {
    match stored {
        None => format!("Bowl at {pos:?}: stored=empty"),
        Some((item, count)) => format!("Bowl at {pos:?}: stored={item} x{count}"),
    }
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

    /// Whether a click on this hand may fill or empty a sacrificial bowl: the
    /// main hand only. Horizon dispatches a click's block Part once **per hand**
    /// (measured in game: the main hand filled the bowl and the off hand emptied
    /// it from the same click, 2026-11-23), where vanilla reaches the off hand
    /// only after a main-hand failure. Upstream refuses the off hand for its own
    /// golden-bowl activation for the same reason.
    pub fn may_use_bowl(self) -> bool {
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

    /// The two `otherstone`/`otherrock` frame **items** this slice carries. They
    /// are `OccultismItems`, not blocks (`OccultismItems.java:407-410`): no
    /// blockstate, no loot table and no block item, only an item model and a name.
    const FRAMES: [&str; 2] = ["otherstone_frame", "otherrock_frame"];

    /// The two pedestals this slice carries. Upstream registers them as plain
    /// `NonPathfindableBlock`s (`OccultismBlocks.java:535-541`), and the
    /// "silver"/otherrock one keeps the jar's old id `otherstone_pedestal_silver`
    /// (the `TODO: change id in next major version` beside it).
    const PEDESTALS: [&str; 2] = ["otherstone_pedestal", "otherstone_pedestal_silver"];

    /// The stone each pedestal's jar grid takes: the `otherrock` pedestal (whose
    /// block id is `otherstone_pedestal_silver`) is made of `otherrock`.
    fn pedestal_stone(name: &str) -> &'static str {
        if name == "otherstone_pedestal_silver" {
            "otherrock"
        } else {
            "otherstone"
        }
    }

    #[test]
    fn the_frame_and_pedestal_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's four grids (`recipe/crafting/{otherstone_frame,
        // otherrock_frame,otherstone_pedestal,otherrock_pedestal}.json`): eight
        // stones in a ring make one frame (the middle empty), five slabs + one
        // stone make one pedestal. The 26.3 loader reads an ingredient as a bare
        // `"id"`; the jar's `{"item": …}` object is dropped without a word and
        // the grid then yields nothing. The otherrock pedestal's **file** is
        // `otherrock_pedestal.json` while its result is
        // `occultism:otherstone_pedestal_silver` — the jar's own naming, kept.
        for frame in FRAMES {
            let stone = if frame == "otherrock_frame" {
                "otherrock"
            } else {
                "otherstone"
            };
            let text = content(&format!("occultism/recipe/crafting/{frame}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{frame} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{frame} is the jar's shaped ring: {text}"
            );
            for row in ["\"ooo\"", "\"o o\"", "\"ooo\""] {
                assert!(
                    text.contains(row),
                    "{frame}'s ring still carries the row {row}: {text}"
                );
            }
            assert!(
                text.contains(&format!("\"occultism:{stone}\"")),
                "{frame} is not made of occultism:{stone}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{frame}\"")),
                "{frame}'s recipe does not make it: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{frame}'s recipe does not make one: {text}"
            );
        }
        for pedestal in PEDESTALS {
            let stone = pedestal_stone(pedestal);
            let file = if pedestal == "otherstone_pedestal_silver" {
                "otherrock_pedestal"
            } else {
                pedestal
            };
            let text = content(&format!("occultism/recipe/crafting/{file}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{file} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            for row in ["\"s s\"", "\" o \"", "\"sss\""] {
                assert!(
                    text.contains(row),
                    "{file}'s pattern still carries the row {row}: {text}"
                );
            }
            assert!(
                text.contains(&format!("\"occultism:{stone}_slab\"")),
                "{file} is not made of {stone} slabs: {text}"
            );
            assert!(
                text.contains(&format!("\"occultism:{stone}\"")),
                "{file} is not made of {stone}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{pedestal}\"")),
                "{file}'s recipe does not make {pedestal}: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{file}'s recipe does not make one: {text}"
            );
        }
    }

    #[test]
    fn every_pedestal_has_a_26_3_loot_table_that_drops_itself() {
        // The jar ships both pedestal tables; like every 1.21.1 table they spell
        // their guard `"condition": "minecraft:survives_explosion"`, the string
        // form the 26.3 loader reads as an unknown kind and so as false — a
        // verbatim carry drops nothing at all. The port's tables use the
        // `explosion_decay` modifier, as every carried table does. The two frames
        // are items and have no table (asserted below).
        for pedestal in PEDESTALS {
            let text = content(&format!("occultism/loot_table/blocks/{pedestal}.json"));
            for legacy in ["\"function\":", "\"functions\":"] {
                assert!(
                    !text.contains(legacy),
                    "{pedestal}'s loot table uses the 1.21.1 {legacy} shape, which the \
                     26.3 loader does not read (it reads `modifier`): {text}"
                );
            }
            assert!(
                !text.contains("\"condition\": \"minecraft:"),
                "{pedestal} carries the 1.21.1 string-form condition kind, which the \
                 26.3 loader does not read (it reads `type`): {text}"
            );
            assert!(
                text.contains(&format!("\"name\": \"occultism:{pedestal}\"")),
                "{pedestal}'s loot table does not drop itself: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:survives_explosion\""),
                "{pedestal} lost the jar's survives_explosion guard: {text}"
            );
            assert!(
                text.contains("\"type\": \"minecraft:block\""),
                "{pedestal}'s loot table is not a block table: {text}"
            );
        }
        for frame in FRAMES {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("data/occultism/loot_table/blocks/{frame}.json"));
            assert!(
                !path.is_file(),
                "{frame} is an item; an item needs no block loot table ({})",
                path.display()
            );
        }
    }

    #[test]
    fn the_pedestals_copy_the_upstream_non_pathfindable_shape() {
        // Upstream's `NonPathfindableBlock` (`NonPathfindableBlock.java`): a full
        // copy of `OTHERSTONE` (hardness 1.5, resistance 30, map colour STONE,
        // `requiresCorrectToolForDrops`), `noOcclusion()`, the four-box post shape
        // below and `isPathfindable → false`. The declaration carries the shape
        // and the occlusion; the pathfinder answer has no in-game probe at this
        // pin and is recorded as a gap in notes.md.
        let boxes = [
            "[0.0625, 0.0, 0.0625, 0.9375, 0.125, 0.9375]",
            "[0.125, 0.125, 0.125, 0.875, 0.1875, 0.875]",
            "[0.1875, 0.1875, 0.1875, 0.8125, 0.875, 0.8125]",
            "[0.125, 0.875, 0.125, 0.875, 1.0, 0.875]",
        ];
        for pedestal in PEDESTALS {
            let text = content(&format!("occultism/horizon/block/{pedestal}.json"));
            assert!(
                text.contains("\"template\": \"minecraft:stone\""),
                "{pedestal} does not copy minecraft:stone: {text}"
            );
            assert!(
                text.contains("\"requires_tool\": true"),
                "{pedestal} must gate its drops on a pickaxe (upstream is \
                 requiresCorrectToolForDrops): {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{pedestal} is not hardness 1.5: {text}"
            );
            assert!(
                text.contains("\"explosion_resistance\": 30.0"),
                "{pedestal} is not resistance 30: {text}"
            );
            assert!(
                text.contains("\"map_color\": 11"),
                "{pedestal} is not map colour 11 (MapColor.STONE): {text}"
            );
            assert!(
                text.contains("\"occludes\": false"),
                "{pedestal} must not occlude (upstream noOcclusion): {text}"
            );
            for shape in boxes {
                assert!(
                    text.contains(shape),
                    "{pedestal} lost the upstream box {shape}: {text}"
                );
            }
        }
    }

    #[test]
    fn every_frame_and_pedestal_has_its_model_name_and_tag() {
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        let tab = content("occultism/horizon/creative_tab/spirit_fire.json");
        let lang = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/occultism/lang/en_us.json"),
        )
        .expect("the package's lang file");
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/occultism");
        for pedestal in PEDESTALS {
            let id = format!("occultism:{pedestal}");
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe"
            );
            assert!(
                assets
                    .join(format!("blockstates/{pedestal}.json"))
                    .is_file(),
                "{id} has no blockstate"
            );
            assert!(
                assets
                    .join(format!("models/block/{pedestal}.json"))
                    .is_file(),
                "{id} has no block model"
            );
            assert!(
                assets
                    .join(format!("models/item/{pedestal}.json"))
                    .is_file(),
                "{id} has no item model"
            );
            assert!(
                lang.contains(&format!("\"block.occultism.{pedestal}\"")),
                "{id} has no block lang entry"
            );
            assert!(
                tab.contains(&format!("\"{id}\"")),
                "{id} is not in the creative tab"
            );
        }
        for frame in FRAMES {
            let id = format!("occultism:{frame}");
            // An item, not a block: its declaration is an item one and it has no
            // blockstate or block declaration.
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("data/occultism/horizon/item/{frame}.json"))
                    .is_file(),
                "{id} has no item declaration"
            );
            assert!(
                !assets.join(format!("blockstates/{frame}.json")).exists(),
                "{id} is an item; it must have no blockstate"
            );
            assert!(
                !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("data/occultism/horizon/block/{frame}.json"))
                    .exists(),
                "{id} is an item; it must have no block declaration"
            );
            assert!(
                assets.join(format!("models/item/{frame}.json")).is_file(),
                "{id} has no item model"
            );
            assert!(
                assets.join(format!("textures/item/{frame}.png")).is_file(),
                "{id} has no item texture"
            );
            assert!(
                lang.contains(&format!("\"item.occultism.{frame}\"")),
                "{id} has no item lang entry"
            );
            assert!(
                tab.contains(&format!("\"{id}\"")),
                "{id} is not in the creative tab"
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

    // ---- the sacrificial bowls (0.2.12) -----------------------------------

    /// The six bowls this slice carries, by bare name (the product constant above
    /// holds the full ids, which the readout and the click look up).
    const CARRIED_BOWLS: [&str; 6] = [
        "sacrificial_bowl",
        "dark_sacrificial_bowl",
        "copper_sacrificial_bowl",
        "dark_copper_sacrificial_bowl",
        "golden_sacrificial_bowl",
        "dark_golden_sacrificial_bowl",
    ];

    /// The package's own assets tree, as `cargo test` sees it.
    fn asset(relative: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/occultism")
            .join(relative);
        std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn asset_text(relative: &str) -> String {
        String::from_utf8(asset(relative)).expect("utf-8 asset")
    }

    #[test]
    fn the_six_bowl_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's `data/occultism/recipe/crafting/`: a 5-stone ring makes the plain
        // or the dark bowl (`o o`/`ooo`), eight gold ingots around the bowl make the
        // golden pair (`ggg`/`gbg`/`ggg`), and a shapeless bowl + copper ingot makes
        // the copper pair. The 26.3 loader reads an ingredient as a bare `"id"` /
        // `"#tag"`; the jar's `{"item": …}` object is dropped without a word and the
        // grid then yields nothing (the 0.2.5 finding, pinned there first).
        for name in CARRIED_BOWLS {
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
                "{name}'s recipe does not make it: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{name}'s recipe does not make one: {text}"
            );
        }
        for (name, stone) in [
            ("sacrificial_bowl", "otherstone"),
            ("dark_sacrificial_bowl", "otherrock"),
        ] {
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped ring: {text}"
            );
            for row in ["\"o o\"", "\"ooo\""] {
                assert!(
                    text.contains(row),
                    "{name}'s ring still carries the row {row}: {text}"
                );
            }
            assert!(
                text.contains(&format!("\"occultism:{stone}\"")),
                "{name} is not made of occultism:{stone}: {text}"
            );
        }
        for (name, bowl) in [
            ("golden_sacrificial_bowl", "sacrificial_bowl"),
            ("dark_golden_sacrificial_bowl", "dark_sacrificial_bowl"),
        ] {
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{name} is the jar's shaped golden ring: {text}"
            );
            for row in ["\"ggg\"", "\"gbg\""] {
                assert!(
                    text.contains(row),
                    "{name}'s ring still carries the row {row}: {text}"
                );
            }
            assert!(
                text.contains("\"#c:ingots/gold\""),
                "{name} is not made of the gold tag: {text}"
            );
            assert!(
                text.contains(&format!("\"occultism:{bowl}\"")),
                "{name} is not made around occultism:{bowl}: {text}"
            );
        }
        for (name, bowl) in [
            ("copper_sacrificial_bowl", "sacrificial_bowl"),
            ("dark_copper_sacrificial_bowl", "dark_sacrificial_bowl"),
        ] {
            let text = content(&format!("occultism/recipe/crafting/{name}.json"));
            assert!(
                text.contains("\"minecraft:crafting_shapeless\""),
                "{name} is the jar's shapeless recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"occultism:{bowl}\"")),
                "{name} is not made from occultism:{bowl}: {text}"
            );
            assert!(
                text.contains("\"#c:ingots/copper\""),
                "{name} is not made with the copper tag: {text}"
            );
        }
    }

    #[test]
    fn every_bowl_has_a_26_3_loot_table_that_drops_itself() {
        // The jar's `loot_table/blocks/<bowl>.json` in the 26.3 condition dialect:
        // the 1.21.1 plural `"conditions"` list evaluates false at this pin and the
        // block would drop nothing.
        for name in CARRIED_BOWLS {
            let text = content(&format!("occultism/loot_table/blocks/{name}.json"));
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
            assert!(
                !text.contains("\"conditions\": ["),
                "{name} carries the 1.21.1 plural condition list, read as false at 26.3: {text}"
            );
        }
    }

    #[test]
    fn the_bowls_copy_the_upstream_properties_and_shape() {
        // `OccultismBlocks.java:462-515`: every bowl is `Properties.of()`
        // `.mapColor(STONE).sound(STONE).strength(1.5f, 30).noOcclusion()`, and
        // **none** calls `requiresCorrectToolForDrops`, so `requires_tool` is
        // false: a bare hand drops exactly one. The brief asks for this line to be
        // measured, not inherited; `gameplay-facts.json` gives the `minecraft:stone`
        // template `requires_tool: true`, so the declaration must override it.
        for name in CARRIED_BOWLS {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains("\"requires_tool\": false"),
                "{name} must not gate its drops on a pickaxe (upstream has no \
                 requiresCorrectToolForDrops): {text}"
            );
            assert!(
                text.contains("\"hardness\": 1.5"),
                "{name} is not hardness 1.5: {text}"
            );
            assert!(
                text.contains("\"explosion_resistance\": 30.0"),
                "{name} is not resistance 30: {text}"
            );
            assert!(
                text.contains("\"map_color\": 11"),
                "{name} is not MapColor.STONE: {text}"
            );
            assert!(
                text.contains("\"occludes\": false"),
                "{name} is not noOcclusion: {text}"
            );
            assert!(
                text.contains("\"sounds\": \"minecraft:stone\""),
                "{name} does not use the stone sounds: {text}"
            );
            assert!(
                text.contains("\"template\": \"minecraft:stone\""),
                "{name} does not copy the stone template: {text}"
            );
        }
        // The golden bowls are `Block.box(3, 2, 3, 13, 8, 13)`, one box; the plain
        // bowls are a bowl (`DirectionalBlockShape`'s upright shape, which the jar's
        // model draws as a base plus a rim).
        let golden = "[0.1875, 0.125, 0.1875, 0.8125, 0.5, 0.8125]";
        let base = "[0.25, 0.0625, 0.25, 0.75, 0.3125, 0.75]";
        for name in ["golden_sacrificial_bowl", "dark_golden_sacrificial_bowl"] {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(golden),
                "{name} does not carry the jar's 3..13 / 2..8 box: {text}"
            );
            assert!(
                !text.contains(base),
                "{name} is a golden bowl, not the plain bowl's shape: {text}"
            );
        }
        for name in [
            "sacrificial_bowl",
            "dark_sacrificial_bowl",
            "copper_sacrificial_bowl",
            "dark_copper_sacrificial_bowl",
        ] {
            let text = content(&format!("occultism/horizon/block/{name}.json"));
            assert!(
                text.contains(golden) && text.contains(base),
                "{name} does not carry the bowl's base and rim: {text}"
            );
        }
    }

    #[test]
    fn every_bowl_has_its_state_model_name_and_tag() {
        // This port carries no `facing` state (the jar's `DirectionalBlock`
        // placement is a content-API gap, recorded in notes.md), so each blockstate
        // is one variant and the model is the jar's upright one.
        let lang = asset_text("lang/en_us.json");
        let tab = content("occultism/horizon/creative_tab/spirit_fire.json");
        let pickaxe = content("minecraft/tags/block/mineable/pickaxe.json");
        for name in CARRIED_BOWLS {
            let state = asset_text(&format!("blockstates/{name}.json"));
            assert!(
                state.contains(&format!("\"model\": \"occultism:block/{name}\"")),
                "{name}'s blockstate does not name its model: {state}"
            );
            assert!(
                !state.contains("facing="),
                "{name}'s blockstate still keys on a facing state this port does not carry: {state}"
            );
            let item_model = asset_text(&format!("models/item/{name}.json"));
            assert!(
                item_model.contains(&format!("\"occultism:block/{name}\"")),
                "{name}'s item model does not parent its block model: {item_model}"
            );
            let id = format!("occultism:{name}");
            assert!(
                lang.contains(&format!("\"block.occultism.{name}\"")),
                "{id} has no lang entry"
            );
            assert!(
                tab.contains(&format!("\"{id}\"")),
                "{id} is not in the creative tab"
            );
            assert!(
                pickaxe.contains(&format!("\"{id}\"")),
                "{id} is not in mineable/pickaxe (the jar's datagen output)"
            );
        }
        // The six textures the models name, each a real PNG.
        for texture in [
            "sacrificial_bowl",
            "dark_sacrificial_bowl",
            "sacrificial_bowl_golden",
            "dark_sacrificial_bowl_golden",
            "sacrificial_bowl_copper",
            "dark_sacrificial_bowl_copper",
        ] {
            let bytes = asset(&format!("textures/block/{texture}.png"));
            assert!(
                bytes.starts_with(b"\x89PNG"),
                "textures/block/{texture}.png is not a PNG"
            );
        }
    }

    #[test]
    fn the_ritual_and_silver_bowls_are_not_carried() {
        // The silver pair's grid needs `#c:ingots/silver` (no member in this
        // package) and the two iesnium bowls are pentacle-ritual results. A future
        // jar that makes them obtainable must fail here instead of shipping a
        // creative-only block.
        for name in [
            "silver_sacrificial_bowl",
            "dark_silver_sacrificial_bowl",
            "iesnium_sacrificial_bowl",
            "dark_iesnium_sacrificial_bowl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("data/occultism/horizon/block/{name}.json"));
            assert!(
                !path.exists(),
                "{name} is carried, but its recipe cannot be satisfied by this package"
            );
        }
    }

    #[test]
    fn the_two_ingot_tags_hold_vanillas_metals() {
        // The golden and copper grids name the jar's common tags; the engine ships
        // no `c:` namespace, so the package declares them (as it does `c:feathers`).
        let gold = content("c/tags/item/ingots/gold.json");
        let copper = content("c/tags/item/ingots/copper.json");
        assert!(
            gold.contains("\"minecraft:gold_ingot\""),
            "c:ingots/gold does not hold vanilla's gold ingot: {gold}"
        );
        assert!(
            copper.contains("\"minecraft:copper_ingot\""),
            "c:ingots/copper does not hold vanilla's copper ingot: {copper}"
        );
        assert!(
            !gold.contains("occultism:") && !copper.contains("occultism:"),
            "the two ingot tags must only name what exists: {gold} {copper}"
        );
    }

    #[test]
    fn the_bowl_slot_takes_one_and_gives_the_whole_slot() {
        // `SacrificialBowlBlockEntity`: `ItemStackHandler(1)` with
        // `getSlotLimit → 1`, used by `insertItem(0, held, false)` and
        // `extractItem(0, 64, false)`.
        assert_eq!(BOWL_SLOT_LIMIT, 1);
        assert_eq!(bowl_insert(0), None);
        assert_eq!(bowl_insert(1), Some((1, 0)));
        assert_eq!(bowl_insert(16), Some((1, 15)));
        assert_eq!(bowl_insert(64), Some((1, 63)));
        assert_eq!(bowl_take(0), 0);
        assert_eq!(bowl_take(1), 1);
        assert_eq!(bowl_take(64), 64);
        assert_eq!(
            bowl_take(65),
            64,
            "extractItem asks for 64, so a hopper's over-full slot leaves the rest"
        );
    }

    #[test]
    fn only_the_plain_bowls_convert_over_spirit_fire() {
        for name in CARRIED_BOWLS {
            assert!(is_bowl(&format!("occultism:{name}")), "{name} is a bowl");
        }
        assert!(!is_bowl(SPIRIT_FIRE));
        assert!(
            !is_bowl("occultism:silver_sacrificial_bowl"),
            "an uncarried bowl is not a bowl of this slice"
        );
        for name in [
            "sacrificial_bowl",
            "dark_sacrificial_bowl",
            "copper_sacrificial_bowl",
            "dark_copper_sacrificial_bowl",
        ] {
            assert!(
                converts_in_bowl(&format!("occultism:{name}")),
                "{name} converts its contents over a spirit fire"
            );
        }
        for name in GOLDEN_BOWLS {
            assert!(
                is_bowl(name) && is_golden_bowl(name),
                "{name} is the golden pair"
            );
            assert!(
                !converts_in_bowl(name),
                "{name}: upstream's GoldenSacrificialBowlBlockEntity skips the conversion"
            );
        }
        assert!(bowl_converts_item("minecraft:feather"));
        assert!(
            !bowl_converts_item(OTHERFLOWER),
            "the jar excludes otherflower from the bowl's conversion"
        );
    }

    #[test]
    fn only_the_main_hand_fills_a_bowl() {
        // One click reaches the Part once per hand on this pin: acting on both
        // hands made the main hand's insert and the off hand's take-out cancel out
        // in the same click (measured in game, 2026-11-23).
        assert!(ClickHand::Main.may_use_bowl());
        assert!(
            !ClickHand::Off.may_use_bowl(),
            "the off hand's turn must not undo the main hand's click"
        );
        assert!(ClickHand::Main.may_harvest());
        assert!(!ClickHand::Off.may_harvest());
    }

    #[test]
    fn a_plain_bowl_converts_its_stored_item_and_a_golden_one_does_not() {
        // `SacrificialBowlBlockEntity.onContentsChanged`: with a spirit fire below, a
        // **non-golden** bowl replaces the stack it holds with the spirit-fire
        // recipe's result, unless the item is `otherflower`. The count is the
        // stack's own (one, the slot's limit).
        let table = feather_table();
        assert_eq!(
            bowl_conversion(&table, "occultism:sacrificial_bowl", "minecraft:feather"),
            Some(Conversion {
                result: "occultism:awakened_feather".to_owned(),
                count: 1,
            })
        );
        assert_eq!(
            bowl_conversion(
                &table,
                "occultism:dark_copper_sacrificial_bowl",
                "minecraft:feather"
            ),
            Some(Conversion {
                result: "occultism:awakened_feather".to_owned(),
                count: 1,
            })
        );
        assert_eq!(
            bowl_conversion(
                &table,
                "occultism:golden_sacrificial_bowl",
                "minecraft:feather"
            ),
            None,
            "the golden bowl skips the conversion"
        );
        assert_eq!(
            bowl_conversion(
                &table,
                "occultism:dark_golden_sacrificial_bowl",
                "minecraft:feather"
            ),
            None
        );
        assert_eq!(
            bowl_conversion(&table, "occultism:sacrificial_bowl", OTHERFLOWER),
            None,
            "the jar excludes otherflower"
        );
        assert_eq!(
            bowl_conversion(&table, "occultism:sacrificial_bowl", "minecraft:dirt"),
            None,
            "an item no recipe accepts stays"
        );
    }

    #[test]
    fn the_bowl_readout_names_the_stored_stack() {
        assert_eq!(
            bowl_line([10, 64, 0], None),
            "Bowl at [10, 64, 0]: stored=empty"
        );
        assert_eq!(
            bowl_line([10, 64, 0], Some(("occultism:datura", 1))),
            "Bowl at [10, 64, 0]: stored=occultism:datura x1"
        );
    }

    // ---- the impure chalks (0.2.13) ---------------------------------------

    /// The 17 impure-chalk crafting recipes the jar ships: 15 coloured shapeless
    /// grids plus two shaped grids for `chalk_white_impure` (mirrored columns).
    /// `(file, result, references)`, the references repeated as often as the
    /// recipe declares them; every one must survive in the 26.3 file.
    const IMPURE_CHALKS: [(&str, &str, &[&str]); 17] = [
        (
            "chalk_white_impure",
            "chalk_white_impure",
            &["#occultism:dusts/chalk_base", "#c:dusts/otherworld_wood"],
        ),
        (
            "chalk_white_impure2",
            "chalk_white_impure",
            &["#c:dusts/otherworld_wood", "#occultism:dusts/chalk_base"],
        ),
        (
            "chalk_pink_impure",
            "chalk_pink_impure",
            &[
                "occultism:chalk_white_impure",
                "occultism:demonic_meat",
                "occultism:demonic_meat",
                "occultism:demonic_meat",
            ],
        ),
        (
            "chalk_brown_impure",
            "chalk_brown_impure",
            &[
                "occultism:chalk_white_impure",
                "occultism:cruelty_essence",
                "minecraft:cocoa_beans",
                "minecraft:brown_mushroom",
            ],
        ),
        (
            "chalk_gray_impure",
            "chalk_gray_impure",
            &["occultism:chalk_white_impure", "occultism:gray_paste"],
        ),
        (
            "chalk_cyan_impure",
            "chalk_cyan_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/iesnium",
                "#c:dusts/echo",
                "minecraft:glow_ink_sac",
            ],
        ),
        (
            "chalk_black_impure",
            "chalk_black_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/witherite",
                "#c:dusts/witherite",
                "#c:dusts/witherite",
            ],
        ),
        (
            "chalk_green_impure",
            "chalk_green_impure",
            &["occultism:chalk_white_impure", "occultism:nature_paste"],
        ),
        (
            "chalk_light_blue_impure",
            "chalk_light_blue_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/ice",
                "#c:dusts/packed_ice",
                "#c:dusts/blue_ice",
            ],
        ),
        (
            "chalk_purple_impure",
            "chalk_purple_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/end_stone",
                "#c:dusts/obsidian",
                "#c:dusts/obsidian",
            ],
        ),
        (
            "chalk_light_gray_impure",
            "chalk_light_gray_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/silver",
                "#c:dusts/iron",
                "#c:dusts/calcite",
            ],
        ),
        (
            "chalk_lime_impure",
            "chalk_lime_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/research",
                "#c:dusts/emerald",
                "#c:slime_balls",
            ],
        ),
        (
            "chalk_orange_impure",
            "chalk_orange_impure",
            &[
                "occultism:chalk_white_impure",
                "occultism:cursed_honey",
                "minecraft:glow_berries",
                "#c:dusts/blaze",
            ],
        ),
        (
            "chalk_yellow_impure",
            "chalk_yellow_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/glowstone",
                "#c:dusts/gold",
                "#c:dusts/gold",
            ],
        ),
        (
            "chalk_blue_impure",
            "chalk_blue_impure",
            &[
                "occultism:chalk_white_impure",
                "occultism:marid_essence",
                "#c:dusts/lapis",
                "#c:tube_corals",
            ],
        ),
        (
            "chalk_magenta_impure",
            "chalk_magenta_impure",
            &[
                "occultism:chalk_white_impure",
                "#c:dusts/dragonyst",
                "#c:dusts/amethyst",
                "minecraft:chorus_fruit",
            ],
        ),
        (
            "chalk_red_impure",
            "chalk_red_impure",
            &[
                "occultism:chalk_white_impure",
                "occultism:afrit_essence",
                "minecraft:torchflower",
                "#c:dusts/redstone",
            ],
        ),
    ];

    /// The 26 ingredient items the 17 grids name that this package did not yet
    /// declare: nineteen dusts (fourteen of them crushing/ritual results, the
    /// two burnt stones smelted from the carried polished pair) and the seven
    /// essences/pastes.
    const IMPURE_INGREDIENTS: [&str; 26] = [
        "burnt_otherstone",
        "burnt_otherrock",
        "crushed_ice",
        "crushed_packed_ice",
        "crushed_blue_ice",
        "crushed_end_stone",
        "crushed_calcite",
        "obsidian_dust",
        "silver_dust",
        "iron_dust",
        "emerald_dust",
        "gold_dust",
        "lapis_dust",
        "dragonyst_dust",
        "amethyst_dust",
        "witherite_dust",
        "iesnium_dust",
        "echo_dust",
        "research_fragment_dust",
        "demonic_meat",
        "cruelty_essence",
        "gray_paste",
        "nature_paste",
        "cursed_honey",
        "marid_essence",
        "afrit_essence",
    ];

    /// Every item tag the 17 grids reach, as the package must declare it: the
    /// file under `data/` and its ordered member list. The member lists are the
    /// jar's own `data/c/tags/item/…` files, except the three NeoForge-only tags
    /// (`blaze`, `glowstone`, `redstone`, `slime_balls`, `tube_corals` — the jar
    /// relies on the loader for those), which name vanilla's items the way this
    /// port already declares `c:ingots/*`.
    const IMPURE_TAGS: [(&str, &[&str]); 26] = [
        (
            "occultism/tags/item/dusts/chalk_base",
            &["#c:dusts/otherstone", "#c:dusts/otherrock"],
        ),
        (
            "c/tags/item/dusts/otherstone",
            &["occultism:burnt_otherstone"],
        ),
        (
            "c/tags/item/dusts/otherrock",
            &["occultism:burnt_otherrock"],
        ),
        (
            "c/tags/item/dusts/otherworld_wood",
            &["occultism:otherworld_ashes"],
        ),
        ("c/tags/item/dusts/iesnium", &["occultism:iesnium_dust"]),
        ("c/tags/item/dusts/echo", &["occultism:echo_dust"]),
        ("c/tags/item/dusts/witherite", &["occultism:witherite_dust"]),
        ("c/tags/item/dusts/ice", &["occultism:crushed_ice"]),
        (
            "c/tags/item/dusts/packed_ice",
            &["occultism:crushed_packed_ice"],
        ),
        (
            "c/tags/item/dusts/blue_ice",
            &["occultism:crushed_blue_ice"],
        ),
        (
            "c/tags/item/dusts/end_stone",
            &["occultism:crushed_end_stone"],
        ),
        ("c/tags/item/dusts/obsidian", &["occultism:obsidian_dust"]),
        ("c/tags/item/dusts/silver", &["occultism:silver_dust"]),
        ("c/tags/item/dusts/iron", &["occultism:iron_dust"]),
        ("c/tags/item/dusts/calcite", &["occultism:crushed_calcite"]),
        (
            "c/tags/item/dusts/research",
            &["occultism:research_fragment_dust"],
        ),
        ("c/tags/item/dusts/emerald", &["occultism:emerald_dust"]),
        ("c/tags/item/dusts/blaze", &["minecraft:blaze_powder"]),
        ("c/tags/item/dusts/glowstone", &["minecraft:glowstone_dust"]),
        ("c/tags/item/dusts/gold", &["occultism:gold_dust"]),
        ("c/tags/item/dusts/lapis", &["occultism:lapis_dust"]),
        ("c/tags/item/dusts/dragonyst", &["occultism:dragonyst_dust"]),
        ("c/tags/item/dusts/amethyst", &["occultism:amethyst_dust"]),
        ("c/tags/item/dusts/redstone", &["minecraft:redstone"]),
        ("c/tags/item/slime_balls", &["minecraft:slime_ball"]),
        (
            "c/tags/item/tube_corals",
            &["minecraft:tube_coral", "minecraft:tube_coral_fan"],
        ),
    ];

    /// The vanilla items those tags may name (this port carries the rest).
    const VANILLA_TAG_MEMBERS: [&str; 6] = [
        "minecraft:blaze_powder",
        "minecraft:glowstone_dust",
        "minecraft:redstone",
        "minecraft:slime_ball",
        "minecraft:tube_coral",
        "minecraft:tube_coral_fan",
    ];

    /// The members of one authored tag file, in file order.
    fn tag_values(path: &str) -> Vec<String> {
        let text = content(&format!("{path}.json"));
        let values = text
            .find("\"values\"")
            .unwrap_or_else(|| panic!("{path} has no values: {text}"));
        let rest = &text[values..];
        let open = rest.find('[').expect("a values array");
        let close = rest[open..].find(']').expect("a closed values array");
        rest[open + 1..open + close]
            .split(',')
            .filter_map(|chunk| chunk.split('"').nth(1))
            .map(str::to_owned)
            .collect()
    }

    /// Every item a tag resolves to, following nested `#tag` members. A tag that
    /// resolves to nothing is the `#c:cobblestones` failure mode: it loads as an
    /// empty ingredient and the grid silently matches nothing.
    fn resolve_tag(path: &str, seen: &mut Vec<String>) -> Vec<String> {
        assert!(
            !seen.contains(&path.to_owned()),
            "tag cycle at {path}: {seen:?}"
        );
        seen.push(path.to_owned());
        let mut members = Vec::new();
        for value in tag_values(path) {
            if let Some(nested) = value.strip_prefix('#') {
                let (namespace, tag_path) =
                    nested.split_once(':').expect("a tag is namespace:path");
                members.extend(resolve_tag(
                    &format!("{namespace}/tags/item/{tag_path}"),
                    seen,
                ));
            } else {
                members.push(value);
            }
        }
        members
    }

    fn item_declared(id: &str) -> bool {
        let name = id.strip_prefix("occultism:").unwrap_or(id);
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("data/occultism/horizon/item/{name}.json"))
            .is_file()
    }

    #[test]
    fn the_seventeen_impure_chalk_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's `data/occultism/recipe/crafting/chalk_*_impure*.json`: 15
        // shapeless grids (impure white + three colour materials) and the two
        // mirrored shaped white grids (`xy`/`xy`/`xy` of chalk base + otherworld
        // wood). The 26.3 loader reads an ingredient as a bare `"id"` / `"#tag"`;
        // the jar's `{"item": …}` / `{"tag": …}` objects are dropped without a
        // word and the grid then yields nothing (the 0.2.5 finding).
        for (file, result, references) in IMPURE_CHALKS {
            let text = content(&format!("occultism/recipe/crafting/{file}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{file} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains(&format!("\"id\": \"occultism:{result}\"")),
                "{file} does not make occultism:{result}: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{file} does not make one: {text}"
            );
            let shaped = file.starts_with("chalk_white_impure");
            assert!(
                text.contains(if shaped {
                    "\"minecraft:crafting_shaped\""
                } else {
                    "\"minecraft:crafting_shapeless\""
                }),
                "{file} is not the jar's {} recipe: {text}",
                if shaped { "shaped" } else { "shapeless" }
            );
            for reference in references {
                let want = references.iter().filter(|r| *r == reference).count();
                let got = text.matches(&format!("\"{reference}\"")).count();
                assert_eq!(
                    got, want,
                    "{file} declares {got} x {reference}, the jar has {want}: {text}"
                );
            }
        }
        // The two white grids are mirrors: the same two keys in the opposite
        // columns (`xy`/`xy`/`xy` both, but the key letters swap).
        let first = content("occultism/recipe/crafting/chalk_white_impure.json");
        let second = content("occultism/recipe/crafting/chalk_white_impure2.json");
        assert!(first.contains("\"x\": \"#occultism:dusts/chalk_base\""));
        assert!(first.contains("\"y\": \"#c:dusts/otherworld_wood\""));
        assert!(second.contains("\"x\": \"#c:dusts/otherworld_wood\""));
        assert!(second.contains("\"y\": \"#occultism:dusts/chalk_base\""));
        for text in [first, second] {
            for row in ["\"xy\"", "\"xy\"", "\"xy\""] {
                assert!(
                    text.contains(row),
                    "a white grid lost the row {row}: {text}"
                );
            }
        }
    }

    #[test]
    fn every_impure_chalk_tag_resolves_to_a_declared_item() {
        // The 17 grids reach 26 tags, one of them nested (`chalk_base` ->
        // `#c:dusts/otherstone` + `#c:dusts/otherrock`). A tag that resolves to
        // nothing loads as an empty ingredient: the cell accepts nothing and the
        // grid yields nothing, silently. Each declared member is either carried by
        // this package (an `occultism:` item declaration) or a vanilla item this
        // port names on purpose.
        for (path, expected) in IMPURE_TAGS {
            let got = tag_values(path);
            assert_eq!(got, expected, "{path} is not the declared member list");
        }
        let mut resolved = Vec::new();
        for (path, _) in IMPURE_TAGS {
            let mut seen = Vec::new();
            let members = resolve_tag(path, &mut seen);
            assert!(
                !members.is_empty(),
                "{path} resolves to no item (the #c:cobblestones failure mode)"
            );
            resolved.extend(members);
        }
        for member in &resolved {
            if let Some(name) = member.strip_prefix("occultism:") {
                assert!(
                    item_declared(member),
                    "the tag member {member} has no item declaration \
                     (data/occultism/horizon/item/{name}.json)"
                );
            } else {
                assert!(
                    VANILLA_TAG_MEMBERS.contains(&member.as_str()),
                    "the tag member {member} is not a vanilla item this port knows"
                );
            }
        }
        // The nested base is the two burnt stones, one member each.
        assert_eq!(
            resolve_tag("occultism/tags/item/dusts/chalk_base", &mut Vec::new()),
            vec![
                "occultism:burnt_otherstone".to_owned(),
                "occultism:burnt_otherrock".to_owned()
            ]
        );
    }

    #[test]
    fn the_two_burnt_dusts_are_smelted_from_the_carried_polished_pair() {
        // The only ingredient of the white grids whose source exists in this
        // package: `smelting/burnt_otherstone.json` / `…otherrock.json` take the
        // carried polished pair (200 ticks, 0.15 experience). Everything else the
        // coloured grids name is a ritual/possession result or a crushing result,
        // recorded in notes.md, and is carried as a declared item only.
        for (name, polished) in [
            ("burnt_otherstone", "occultism:polished_otherstone"),
            ("burnt_otherrock", "occultism:polished_otherrock"),
        ] {
            let text = content(&format!("occultism/recipe/smelting/{name}.json"));
            assert!(
                text.contains("\"minecraft:smelting\""),
                "{name} is not a smelting recipe: {text}"
            );
            assert!(
                text.contains(&format!("\"ingredient\": \"{polished}\"")),
                "{name} is not smelted from {polished}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"occultism:{name}\"")),
                "{name}'s recipe does not make it: {text}"
            );
            assert!(
                text.contains("\"cookingtime\": 200"),
                "{name} is not the jar's 200 ticks: {text}"
            );
            assert!(
                text.contains("\"experience\": 0.15"),
                "{name} is not the jar's 0.15 experience: {text}"
            );
        }
    }

    #[test]
    fn every_impure_ingredient_is_declared_named_and_in_the_tab() {
        // The 26 ingredient ids exist in the jar's item registry but not in this
        // package. A recipe naming an undeclared id loads with an "invalid
        // ingredient" and the cell refuses every item, so each one is declared,
        // named in lang and shown in the creative tab beside the impure chalks.
        let lang = asset_text("lang/en_us.json");
        let tab = content("occultism/horizon/creative_tab/spirit_fire.json");
        for name in IMPURE_INGREDIENTS {
            let declaration = content(&format!("occultism/horizon/item/{name}.json"));
            assert!(
                declaration.contains("\"template\": \"minecraft:paper\""),
                "{name} is not a plain carried item: {declaration}"
            );
            let id = format!("occultism:{name}");
            assert!(
                lang.contains(&format!("\"item.occultism.{name}\"")),
                "{id} has no lang entry"
            );
            assert!(
                tab.contains(&format!("\"{id}\"")),
                "{id} is not in the creative tab"
            );
        }
        // The 16 impure chalk results are already carried and shown.
        for name in [
            "chalk_white_impure",
            "chalk_light_gray_impure",
            "chalk_gray_impure",
            "chalk_black_impure",
            "chalk_brown_impure",
            "chalk_red_impure",
            "chalk_orange_impure",
            "chalk_yellow_impure",
            "chalk_lime_impure",
            "chalk_green_impure",
            "chalk_cyan_impure",
            "chalk_light_blue_impure",
            "chalk_blue_impure",
            "chalk_purple_impure",
            "chalk_magenta_impure",
            "chalk_pink_impure",
        ] {
            let id = format!("occultism:{name}");
            assert!(item_declared(&id), "{id} is not declared");
            assert!(tab.contains(&format!("\"{id}\"")), "{id} is not in the tab");
        }
    }

    // ---- the otherstone tablet (0.2.14) ---------------------------------

    /// The jar's two tablet grids: the same `aga`/`gsg`/`aga` shape over
    /// `#c:dusts/otherworld_wood` (ashes), `#c:ingots/gold` and one of the two
    /// raw stones. `(file, stone)`; both make the one result.
    const TABLETS: [(&str, &str); 2] = [
        ("otherstone_tablet", "occultism:otherstone"),
        ("otherstone_tablet2", "occultism:otherrock"),
    ];

    #[test]
    fn the_two_tablet_recipes_are_the_jars_shapes_in_the_26_3_dialect() {
        // The jar's `data/occultism/recipe/crafting/otherstone_tablet{,2}.json`:
        // shaped `aga`/`gsg`/`aga`, with `a` = #c:dusts/otherworld_wood (the
        // ashes carried in 0.2.13), `g` = #c:ingots/gold (carried in 0.2.12)
        // and `s` = otherstone in the first file, otherrock in the second.
        // The 26.3 loader reads bare `"#tag"` / `"id"` strings; the jar's
        // `{"tag": …}` objects are dropped without a word (0.2.5 finding).
        for (file, stone) in TABLETS {
            let text = content(&format!("occultism/recipe/crafting/{file}.json"));
            for object_form in ["\"item\":", "\"tag\":"] {
                assert!(
                    !text.contains(object_form),
                    "{file} carries the 1.21.1 object-form ingredient {object_form}, \
                     which the 26.3 loader drops: {text}"
                );
            }
            assert!(
                text.contains("\"minecraft:crafting_shaped\""),
                "{file} is not the jar's shaped recipe: {text}"
            );
            for row in ["aga", "gsg", "aga"] {
                assert!(
                    text.contains(&format!("\"{row}\"")),
                    "{file} lost the row {row}: {text}"
                );
            }
            for key in [
                "\"a\": \"#c:dusts/otherworld_wood\"",
                "\"g\": \"#c:ingots/gold\"",
                &format!("\"s\": \"{stone}\""),
            ] {
                assert!(text.contains(key), "{file} lost the key {key}: {text}");
            }
            assert!(
                text.contains("\"id\": \"occultism:otherstone_tablet\""),
                "{file} does not make the tablet: {text}"
            );
            assert!(
                text.contains("\"count\": 1"),
                "{file} does not make one: {text}"
            );
        }
    }

    #[test]
    fn the_tablet_is_declared_named_modelled_and_in_the_tab() {
        // The recipes' result must be a real item with a model, a texture and a
        // name; an undeclared result would load as a recipe that can never be
        // crafted. Both ingredient tags must also resolve to real members.
        let declaration = content("occultism/horizon/item/otherstone_tablet.json");
        assert!(
            declaration.contains("\"template\": \"minecraft:paper\""),
            "the tablet is not a plain carried item: {declaration}"
        );
        let model = asset_text("models/item/otherstone_tablet.json");
        assert!(
            model.contains("\"parent\": \"minecraft:item/generated\""),
            "the tablet's model is not a generated item model: {model}"
        );
        assert!(
            model.contains("\"layer0\": \"occultism:item/otherstone_tablet\""),
            "the tablet's model does not name its texture: {model}"
        );
        assert!(
            asset("textures/item/otherstone_tablet.png").len() > 100,
            "the tablet's texture is missing or empty"
        );
        let lang = asset_text("lang/en_us.json");
        assert!(
            lang.contains("\"item.occultism.otherstone_tablet\": \"Otherworldly Tablet\""),
            "the tablet has no jar lang entry"
        );
        let tab = content("occultism/horizon/creative_tab/spirit_fire.json");
        assert!(
            tab.contains("\"occultism:otherstone_tablet\""),
            "the tablet is not in the creative tab"
        );
        assert_eq!(
            tag_values("c/tags/item/dusts/otherworld_wood"),
            vec!["occultism:otherworld_ashes".to_owned()]
        );
        assert!(item_declared("occultism:otherworld_ashes"));
        assert_eq!(
            tag_values("c/tags/item/ingots/gold"),
            vec!["minecraft:gold_ingot".to_owned()]
        );
    }

    // ---- the stonecutting family (0.2.15) --------------------------------

    /// The jar's `data/occultism/recipe/stonecutting/*.json`, measured as
    /// **32** files (FINDINGS.md's inventory; the 0.2.14 note's "31" undercounted
    /// the raw-stone family by one). Each row is
    /// `(file stem, bare ingredient, result id, result count)`.
    const STONECUTTING: [(&str, &str, &str, u32); 32] = [
        (
            "othercobblerock_slab_from_othercobblerock",
            "occultism:othercobblerock",
            "occultism:othercobblerock_slab",
            2,
        ),
        (
            "othercobblerock_stairs_from_othercobblerock",
            "occultism:othercobblerock",
            "occultism:othercobblerock_stairs",
            1,
        ),
        (
            "othercobblerock_wall_from_othercobblerock",
            "occultism:othercobblerock",
            "occultism:othercobblerock_wall",
            1,
        ),
        (
            "othercobblestone_slab_from_othercobblestone",
            "occultism:othercobblestone",
            "occultism:othercobblestone_slab",
            2,
        ),
        (
            "othercobblestone_stairs_from_othercobblestone",
            "occultism:othercobblestone",
            "occultism:othercobblestone_stairs",
            1,
        ),
        (
            "othercobblestone_wall_from_othercobblestone",
            "occultism:othercobblestone",
            "occultism:othercobblestone_wall",
            1,
        ),
        (
            "otherrock_bricks_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_bricks",
            1,
        ),
        (
            "otherrock_bricks_slab_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_bricks_slab",
            2,
        ),
        (
            "otherrock_bricks_slab_from_otherrock_bricks",
            "occultism:otherrock_bricks",
            "occultism:otherrock_bricks_slab",
            2,
        ),
        (
            "otherrock_bricks_stairs_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_bricks_stairs",
            1,
        ),
        (
            "otherrock_bricks_stairs_from_otherrock_bricks",
            "occultism:otherrock_bricks",
            "occultism:otherrock_bricks_stairs",
            1,
        ),
        (
            "otherrock_bricks_wall_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_bricks_wall",
            1,
        ),
        (
            "otherrock_bricks_wall_from_otherrock_bricks",
            "occultism:otherrock_bricks",
            "occultism:otherrock_bricks_wall",
            1,
        ),
        (
            "otherrock_slab_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_slab",
            2,
        ),
        (
            "otherrock_stairs_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_stairs",
            1,
        ),
        (
            "otherrock_wall_from_otherrock",
            "occultism:otherrock",
            "occultism:otherrock_wall",
            1,
        ),
        (
            "otherstone_bricks_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_bricks",
            1,
        ),
        (
            "otherstone_bricks_slab_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_bricks_slab",
            2,
        ),
        (
            "otherstone_bricks_slab_from_otherstone_bricks",
            "occultism:otherstone_bricks",
            "occultism:otherstone_bricks_slab",
            2,
        ),
        (
            "otherstone_bricks_stairs_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_bricks_stairs",
            1,
        ),
        (
            "otherstone_bricks_stairs_from_otherstone_bricks",
            "occultism:otherstone_bricks",
            "occultism:otherstone_bricks_stairs",
            1,
        ),
        (
            "otherstone_bricks_wall_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_bricks_wall",
            1,
        ),
        (
            "otherstone_bricks_wall_from_otherstone_bricks",
            "occultism:otherstone_bricks",
            "occultism:otherstone_bricks_wall",
            1,
        ),
        (
            "otherstone_slab_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_slab",
            2,
        ),
        (
            "otherstone_stairs_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_stairs",
            1,
        ),
        (
            "otherstone_wall_from_otherstone",
            "occultism:otherstone",
            "occultism:otherstone_wall",
            1,
        ),
        (
            "polished_otherrock_slab_from_polished_otherrock",
            "occultism:polished_otherrock",
            "occultism:polished_otherrock_slab",
            2,
        ),
        (
            "polished_otherrock_stairs_from_polished_otherrock",
            "occultism:polished_otherrock",
            "occultism:polished_otherrock_stairs",
            1,
        ),
        (
            "polished_otherrock_wall_from_polished_otherrock",
            "occultism:polished_otherrock",
            "occultism:polished_otherrock_wall",
            1,
        ),
        (
            "polished_otherstone_slab_from_polished_otherstone",
            "occultism:polished_otherstone",
            "occultism:polished_otherstone_slab",
            2,
        ),
        (
            "polished_otherstone_stairs_from_polished_otherstone",
            "occultism:polished_otherstone",
            "occultism:polished_otherstone_stairs",
            1,
        ),
        (
            "polished_otherstone_wall_from_polished_otherstone",
            "occultism:polished_otherstone",
            "occultism:polished_otherstone_wall",
            1,
        ),
    ];

    #[test]
    fn the_32_stonecutting_recipes_are_in_the_jars_26_3_dialect() {
        // Every input and result here is a block item, so `item_declared` (which
        // reads only horizon/item/) is not enough: a block declaration creates
        // its item. Mirror check_recipe_keys.py's rule.
        fn carried(id: &str) -> bool {
            let name = id.strip_prefix("occultism:").unwrap_or(id);
            let horizon =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/occultism/horizon");
            horizon.join("item").join(format!("{name}.json")).is_file()
                || horizon.join("block").join(format!("{name}.json")).is_file()
        }
        // The jar's object-form `{"item": "occultism:…"}` ingredient is dropped at
        // load by 26.3, which reads a bare string (measured against vanilla's
        // `andesite_slab_from_andesite_stonecutting.json`). The result's
        // `{"count", "id"}` shape is already 26.3's and carries over verbatim.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/occultism/recipe/stonecutting");
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| {
                entry
                    .expect("readable stonecutting entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            files.len(),
            STONECUTTING.len(),
            "the stonecutting family has {} files, expected {} ({files:?})",
            files.len(),
            STONECUTTING.len()
        );
        for (file, ingredient, result, count) in STONECUTTING {
            let text = content(&format!("occultism/recipe/stonecutting/{file}.json"));
            assert!(
                text.contains("\"type\": \"minecraft:stonecutting\""),
                "{file} is not a stonecutting recipe: {text}"
            );
            assert!(
                !text.contains("\"item\":") && !text.contains("\"tag\":"),
                "{file} carries the 1.21.1 object-form ingredient, which the 26.3 loader drops: {text}"
            );
            assert!(
                text.contains(&format!("\"ingredient\": \"{ingredient}\"")),
                "{file} lost the bare ingredient {ingredient}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"{result}\"")),
                "{file} does not make {result}: {text}"
            );
            assert!(
                text.contains(&format!("\"count\": {count}")),
                "{file} does not make {count}: {text}"
            );
            // A recipe whose ingredient or result is not carried is dead data.
            assert!(carried(ingredient), "{file}: {ingredient} is not carried");
            assert!(carried(result), "{file}: {result} is not carried");
        }
    }

    /// The jar ships 19 `data/occultism/recipe/smelting/*.json`. A row is
    /// carryable when its ingredient resolves (a carried item, or a carried tag
    /// with a carried member) and its result exists (a carried item or a vanilla
    /// item). Ten qualify: the eight carried since 0.2.7–0.2.13, and the two
    /// dust → vanilla-ingot rows this slice adds. The other nine name an absent
    /// tag or an absent result and are recorded as `blocked(...)` in notes.md —
    /// never invented here. Every carried row must be 26.3 dialect (a bare
    /// string or `#tag`, not the jar's `{"item":...}`/`{"tag":...}` object,
    /// which the loader drops without a word) and keep the jar's time and XP.
    #[test]
    fn the_smelting_family_covers_every_carryable_jar_recipe() {
        fn carried(id: &str) -> bool {
            let name = id.strip_prefix("occultism:").unwrap_or(id);
            let horizon =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/occultism/horizon");
            horizon.join("item").join(format!("{name}.json")).is_file()
                || horizon.join("block").join(format!("{name}.json")).is_file()
        }
        let rows: &[(&str, &str, &str, f64)] = &[
            (
                "burnt_otherstone",
                "occultism:polished_otherstone",
                "occultism:burnt_otherstone",
                0.15,
            ),
            (
                "burnt_otherrock",
                "occultism:polished_otherrock",
                "occultism:burnt_otherrock",
                0.15,
            ),
            (
                "cracked_otherstone_bricks",
                "occultism:otherstone_bricks",
                "occultism:cracked_otherstone_bricks",
                0.3,
            ),
            (
                "cracked_otherrock_bricks",
                "occultism:otherrock_bricks",
                "occultism:cracked_otherrock_bricks",
                0.3,
            ),
            (
                "otherstone",
                "occultism:othercobblestone",
                "occultism:otherstone",
                0.5,
            ),
            (
                "otherrock",
                "occultism:othercobblerock",
                "occultism:otherrock",
                0.5,
            ),
            (
                "polished_otherstone",
                "occultism:otherstone",
                "occultism:polished_otherstone",
                0.5,
            ),
            (
                "polished_otherrock",
                "occultism:otherrock",
                "occultism:polished_otherrock",
                0.5,
            ),
            (
                "gold_ingot_from_dust",
                "#c:dusts/gold",
                "minecraft:gold_ingot",
                0.7,
            ),
            (
                "iron_ingot_from_dust",
                "#c:dusts/iron",
                "minecraft:iron_ingot",
                0.7,
            ),
        ];
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/occultism/recipe/smelting");
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| {
                entry
                    .expect("readable smelting entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            files.len(),
            rows.len(),
            "the smelting family has {} files, expected {} ({files:?})",
            files.len(),
            rows.len()
        );
        for (name, ingredient, result, xp) in rows {
            let text = content(&format!("occultism/recipe/smelting/{name}.json"));
            assert!(
                text.contains("\"minecraft:smelting\""),
                "{name} is not a smelting recipe: {text}"
            );
            assert!(
                !text.contains("\"item\":") && !text.contains("\"tag\":"),
                "{name} carries the 1.21.1 object-form ingredient, which the 26.3 loader drops: {text}"
            );
            assert!(
                text.contains(&format!("\"ingredient\": \"{ingredient}\"")),
                "{name} lost the bare ingredient {ingredient}: {text}"
            );
            assert!(
                text.contains(&format!("\"id\": \"{result}\"")),
                "{name} does not make {result}: {text}"
            );
            assert!(
                text.contains(&format!("\"experience\": {xp}")),
                "{name} does not declare the jar's {xp} experience: {text}"
            );
            assert!(
                text.contains("\"cookingtime\": 200"),
                "{name} does not declare the jar's 200-tick cooking time: {text}"
            );
            if let Some(tag) = ingredient.strip_prefix('#') {
                let (namespace, path) = tag.split_once(':').expect("a namespaced tag");
                let tag_text = content(&format!("{namespace}/tags/item/{path}.json"));
                let member = if tag == "c:dusts/gold" {
                    "occultism:gold_dust"
                } else {
                    "occultism:iron_dust"
                };
                assert!(
                    tag_text.contains(&format!("\"{member}\"")) && carried(member),
                    "{name}: the tag {tag} has no carried member ({member})"
                );
            }
        }
    }
}
