// Game-side glue for Occultism slice 1 (the spirit-fire conversion loop).
//
// Injected into `crate::game` as `occultism_state`; it needs the game, so it is
// not part of the standalone library (only `runtime.rs` is).
//
// Read from Occultism 1.225.0 (`SpiritFireBlock.entityInside/useItemOn`,
// `SpiritFireRecipe.assemble`, `PlayerEventHandler.spiritFire`), tag
// `release/v1.21.1-1.225.0`. The pure rules live in `crate::game::occultism`.
//
// The recipe table is the carried content mod's own `occultism:spirit_fire`
// recipes, read through API 36 and cached until the recipe list changes. The
// conversion runs off each item entity's own tick (API 146), which is where
// vanilla's `Entity.checkInsideBlocks` calls `entityInside` for upstream.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::level::flags;

use crate::game::Game;
use crate::game::native_server::{NativeCommand, NativeServerPlayer};
use crate::game::occultism::{Conversion, DATURA, RECIPE_TYPE, SPIRIT_FIRE};

/// The bundle's id: the id a player can turn off in Options > Mods.
const MOD_ID: &str = "occultism";
/// The block state property the chalks set (`SpiritFireBlock.COLOR`).
const COLOR: &str = "color";
/// The empty stack that removes an item entity (`native_set_item_entity`).
const AIR: &str = "minecraft:air";

/// The loaded table, rebuilt when the server's recipe list no longer matches it.
struct Cached {
    files: usize,
    first: Option<String>,
    skipped: usize,
    table: crate::game::occultism::Table,
}

static CACHE: LazyLock<Mutex<Option<Cached>>> = LazyLock::new(|| Mutex::new(None));

/// What one spirit-fire block has converted, for the command readout.
#[derive(Debug, Clone, Default)]
struct Stats {
    conversions: u32,
    last: Option<(String, u32)>,
}

static STATS: LazyLock<Mutex<HashMap<[i32; 3], Stats>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The table, from the cache or rebuilt from `game`'s custom recipes.
fn cached<R>(game: &Game<'_>, f: impl FnOnce(&Cached) -> R) -> R {
    let recipes = game.native_recipes_of_type(RECIPE_TYPE);
    let files = recipes.len();
    let first = recipes.first().map(|recipe| recipe.id.as_str());
    let mut guard = CACHE.lock().expect("occultism recipe cache poisoned");
    let fresh = guard
        .as_ref()
        .is_some_and(|cached| cached.files == files && cached.first.as_deref() == first);
    if !fresh {
        *guard = Some(build(game));
    }
    f(guard.as_ref().expect("the recipe cache was just filled"))
}

/// A 1.21.1 recipe's `ingredient` as the references `native_recipe_ingredient`
/// takes: `{"item": …}` is a bare id, `{"tag": …}` is `#tag`, and the older
/// list of those shapes is a list of both. A bare `"id"`/`"#tag"` is kept.
fn ingredient_references(value: &serde_json::Value) -> Vec<serde_json::Value> {
    let one = |child: &serde_json::Value| -> Option<serde_json::Value> {
        if let Some(item) = child.get("item").and_then(serde_json::Value::as_str) {
            return Some(
                crate::game::occultism::IngredientRef::item(item)
                    .reference()
                    .into(),
            );
        }
        if let Some(tag) = child.get("tag").and_then(serde_json::Value::as_str) {
            return Some(
                crate::game::occultism::IngredientRef::tag(tag)
                    .reference()
                    .into(),
            );
        }
        let bare = child.as_str()?;
        Some(
            if bare.starts_with('#') {
                crate::game::occultism::IngredientRef::tag(bare)
            } else {
                crate::game::occultism::IngredientRef::item(bare)
            }
            .reference()
            .into(),
        )
    };
    match value {
        serde_json::Value::Array(list) => list.iter().filter_map(one).collect(),
        other => one(other).into_iter().collect(),
    }
}

/// Reads every `occultism:spirit_fire` recipe; one whose ingredient or result
/// names nothing the game knows is counted as skipped instead of matching all.
fn build(game: &Game<'_>) -> Cached {
    let recipes = game.native_recipes_of_type(RECIPE_TYPE);
    let mut rules = Vec::new();
    let mut skipped = 0;
    for recipe in recipes {
        let references = ingredient_references(&recipe.json["ingredient"]);
        let ingredient = (!references.is_empty())
            .then(|| game.native_recipe_ingredient(&serde_json::Value::Array(references)))
            .flatten();
        let result = game.native_recipe_result(&recipe.json["result"]);
        match (ingredient, result) {
            (Some(ingredient), Some(result)) => rules.push(crate::game::occultism::Rule {
                inputs: ingredient.order,
                result: result.item.to_string(),
            }),
            _ => skipped += 1,
        }
    }
    Cached {
        files: recipes.len(),
        first: recipes.first().map(|recipe| recipe.id.clone()),
        skipped,
        table: crate::game::occultism::Table::new(rules),
    }
}

/// A default-component stack: the recipe results carry no components.
fn plain_stack(item: &str, count: u32) -> ItemStack {
    ItemStack {
        item: item.into(),
        count,
        damage: 0,
        extra: None,
    }
}

/// Whether the block at `pos` can be replaced (`BlockState.canBeReplaced`),
/// which decides whether the fire lands on it or on the clicked face.
fn replaceable(game: &Game<'_>, pos: [i32; 3]) -> bool {
    let world = &game.world;
    let [x, y, z] = pos;
    if !(-30_000_000..30_000_000).contains(&x)
        || !(-30_000_000..30_000_000).contains(&z)
        || !(world.shape.min_y..world.shape.max_y()).contains(&y)
        || !world.loaded(x, z)
    {
        return false;
    }
    game.res.facts().get(world.block(x, y, z)).replaceable
}

/// A right click with an item on a block: paint the fire with a chalk, or
/// strike the datura lying on it alight (API 13, upstream's `useItemOn` and
/// `PlayerInteractEvent.RightClickBlock`).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    if !game.native_on(MOD_ID) {
        return;
    }
    let dimension = game.dim.id().to_owned();
    if let Some(colour) = crate::game::occultism::chalk_color(&stack.item) {
        if game.native_block(pos) == SPIRIT_FIRE {
            game.native_set_block_property(pos, COLOR, colour);
            // Upstream answers `sidedSuccess` for any chalk on the fire,
            // whether or not the colour changed.
            *claimed = true;
        }
        return;
    }
    if !crate::game::occultism::starts_fire(&stack.item) {
        return;
    }
    let (min, max) = crate::game::occultism::datura_box(pos);
    let datura: Vec<u32> = game
        .native_item_entities(&dimension, min, max)
        .into_iter()
        .filter(|entity| &*entity.stack.item == DATURA)
        .map(|entity| entity.id)
        .collect();
    if datura.is_empty() {
        return;
    }
    let target = crate::game::occultism::fire_target(pos, face, replaceable(game, pos));
    if replaceable(game, target)
        && let Some(state) = game.pack.assets.registry.default_state(SPIRIT_FIRE)
    {
        game.native_set_block(&dimension, target, state, flags::UPDATE_ALL);
    }
    // Upstream consumes every datura in the box whatever the target was.
    for id in datura {
        game.native_set_item_entity(&dimension, id, plain_stack(AIR, 0));
    }
    // ... and cancels the click, so the lighter's own use never runs.
    *claimed = true;
}

/// One item entity's tick ended: convert it when it stands in a spirit fire
/// and a recipe accepts it (API 146, upstream's `entityInside`).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_entity_tick_post(game: &mut Game<'_>, dimension: &str, entity: u32, kind: &str) {
    if kind != "minecraft:item" {
        return;
    }
    let Some(reference) = game.native_entity(entity) else {
        return;
    };
    let Some(pos) = game.native_entity_block_position(reference) else {
        return;
    };
    if game.native_block(pos) != SPIRIT_FIRE {
        return;
    }
    let [x, y, z] = pos;
    let Some(stack) = game
        .native_item_entities(
            dimension,
            [x as f64, y as f64, z as f64],
            [x as f64 + 1.0, y as f64 + 1.0, z as f64 + 1.0],
        )
        .into_iter()
        .find(|candidate| candidate.id == entity)
        .map(|candidate| candidate.stack)
    else {
        return;
    };
    let Some(conversion) = cached(game, |cached| {
        cached.table.convert(&stack.item, stack.count)
    }) else {
        return;
    };
    let stack = plain_stack(&conversion.result, conversion.count);
    if game.native_set_item_entity(dimension, entity, stack) {
        record(pos, &conversion);
    }
}

/// One conversion, for `/occultism info`.
fn record(pos: [i32; 3], conversion: &Conversion) {
    let mut guard = STATS.lock().expect("occultism stats poisoned");
    let stats = guard.entry(pos).or_default();
    stats.conversions += 1;
    stats.last = Some((conversion.result.clone(), conversion.count));
}

/// Whether the bundle hears entity ticks: without this answer the game tells
/// none of them (API 146).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn listens_to_entity_ticks(game: &Game<'_>) -> bool {
    game.native_on(MOD_ID)
}

/// `/occultism info|recipes` (API 8).
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("occultism info <pos:block_pos>"));
    commands.push(NativeCommand::new("occultism recipes"));
}

#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_perform_command(
    game: &mut Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() != Some(&"occultism") {
        return;
    }
    *ran = true;
    match words.get(1).copied() {
        Some("info") => {
            let Some(pos) = parse_pos(&words[2..]) else {
                game.native_feedback(Text::literal("usage: /occultism info <x> <y> <z>"));
                return;
            };
            let colour = game
                .native_block_property(pos, COLOR)
                .unwrap_or_else(|| "none".to_owned());
            let block = game.native_block(pos);
            let stats = STATS
                .lock()
                .expect("occultism stats poisoned")
                .get(&pos)
                .cloned()
                .unwrap_or_default();
            let last = match &stats.last {
                Some((item, count)) => format!("{item} x{count}"),
                None => "none".to_owned(),
            };
            let (rules, files) = cached(game, |cached| (cached.table.rules().len(), cached.files));
            game.native_feedback(Text::literal(format!(
                "Spirit Fire at {pos:?}: block={block}, colour={colour}, conversions={}, last={last}, rules={rules}/{files}",
                stats.conversions
            )));
        }
        Some("recipes") => {
            let (rules, files, items, skipped) = cached(game, |cached| {
                (
                    cached.table.rules().len(),
                    cached.files,
                    cached.table.inputs(),
                    cached.skipped,
                )
            });
            game.native_feedback(Text::literal(format!(
                "Spirit fire recipes: {rules} rules, {files} files, {items} items, {skipped} skipped."
            )));
        }
        _ => {
            game.native_feedback(Text::literal(
                "usage: /occultism info <x> <y> <z> | /occultism recipes",
            ));
        }
    }
}

/// `<x> <y> <z>` as a block position.
fn parse_pos(words: &[&str]) -> Option<[i32; 3]> {
    if words.len() < 3 {
        return None;
    }
    Some([
        words[0].parse().ok()?,
        words[1].parse().ok()?,
        words[2].parse().ok()?,
    ])
}
