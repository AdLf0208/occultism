//! Game-side glue for Occultism slice 1 (the spirit-fire conversion loop).
//!
//! [`crate::runtime`] is the pure truth: the recipe table as it is matched, the lighters, the
//! chalk-to-colour table and the click geometry. This module is the only place that reads the
//! game and writes it back: the recipe read, the datura search, the ignition, the conversion,
//! the per-fire counters and the two commands.
//!
//! Read from Occultism 1.225.0 (`SpiritFireBlock.entityInside/useItemOn`,
//! `SpiritFireRecipe.assemble`, `PlayerEventHandler.spiritFire`), tag
//! `release/v1.21.1-1.225.0`.
//!
//! The recipe table is the carried content package's own `occultism:spirit_fire` recipes, read
//! through `Host::recipes_of_type` and rebuilt on every conversion, so an edited recipe,
//! another pack or another world's tags are always the ones in use. The conversion runs off
//! each item entity's own tick, which is where vanilla's `Entity.checkInsideBlocks` calls
//! `entityInside` for upstream.

use std::sync::{LazyLock, Mutex};

use mod_api::{
    BlockClick, BlockUpdateFlags, Command, CommandArgumentValue, EntityRef,
    EntityTickListenedEvent, EntityTickPostEvent, Flow, GameMode, Hand, Host, HostBlockStates,
    HostSide, Identifier, InteractionResult, ItemStack, LevelRef, LootDropsEvent, RandomSource,
    Recipe, RightClickBlockEvent, RunModCommandEvent, ServerHost, Text,
};

use crate::runtime::{
    self, Conversion, DATURA, Mode, RECIPE_TYPE, Recent, RecentConversion, SPIRIT_FIRE,
};

/// The block state property the chalks set (`SpiritFireBlock.COLOR`).
const COLOR: &str = "color";
/// The food exhaustion one harvest costs (`IReplantableCrops.EXHAUSTION_PER_HARVEST`).
const HARVEST_EXHAUSTION: f32 = 0.005;
/// `/occultism info` for a missing position.
const INFO_USAGE: &str = "usage: /occultism info <x> <y> <z>";
/// `/occultism` with no recognised subcommand.
const USAGE: &str = "usage: /occultism info <x> <y> <z> | /occultism recipes";

/// The recipe table as it is loaded now.
struct Cached {
    files: usize,
    skipped: usize,
    table: runtime::Table,
}

/// The recent conversions `/occultism info` reads. Bounded, and keyed by the
/// open of the world and the dimension as well as the position, so an earlier
/// world (or another dimension) at the same coordinates is never reported.
static STATS: LazyLock<Mutex<Recent>> = LazyLock::new(|| Mutex::new(Recent::default()));

/// The table, rebuilt from `host`'s recipes every time it is read.
///
/// It used to be cached for as long as the recipe *count* and the first recipe id matched.
/// That kept a stale table after a recipe's ingredient or result changed, and after another
/// world or pack with the same ids and count loaded a different tag set — conversion then used
/// data that was no longer loaded (the reviewer's finding). The table is a handful of
/// spirit-fire recipes and a conversion happens on an ignition, so rebuilding it is free and
/// cannot go stale.
fn with_table<R>(host: &dyn Host, f: impl FnOnce(&Cached) -> R) -> R {
    f(&build(host))
}

/// A bare `"id"` or `"#tag"` as the reference `Host::recipe_ingredient` takes.
fn bare_ref(text: &str) -> String {
    if text.starts_with('#') {
        runtime::IngredientRef::tag(text).reference()
    } else {
        runtime::IngredientRef::item(text).reference()
    }
}

/// The references as the JSON array `Host::recipe_ingredient` reads: `["id"]` or `["#tag"]`.
/// Item ids are `[a-z0-9_.:/-]`, so nothing in practice needs escaping; a quote or backslash is
/// escaped anyway so a malformed file cannot break the array.
fn reference_text(refs: &[String]) -> String {
    let mut text = String::from("[");
    for (index, reference) in refs.iter().enumerate() {
        if index > 0 {
            text.push(',');
        }
        text.push('"');
        for character in reference.chars() {
            if matches!(character, '"' | '\\') {
                text.push('\\');
            }
            text.push(character);
        }
        text.push('"');
    }
    text.push(']');
    text
}

/// A recipe's `ingredient` as the references `recipe_ingredient` takes: `{"item": …}` is a bare
/// id, `{"tag": …}` is `#tag`, a list of those shapes is a list of both, and a bare `"id"` /
/// `"#tag"` is kept. The game's helper reads strings, so the object shape is reduced here (the
/// 0.1.0 port's ingredient-shape repair, kept).
fn ingredient_text(recipe: &Recipe) -> Option<String> {
    let value = recipe.json.get("ingredient")?;
    let mut refs = Vec::new();
    if let Some(list) = value.as_array() {
        for child in list {
            if let Some(item) = child.get("item").and_then(|value| value.as_str()) {
                refs.push(runtime::IngredientRef::item(item).reference());
            } else if let Some(tag) = child.get("tag").and_then(|value| value.as_str()) {
                refs.push(runtime::IngredientRef::tag(tag).reference());
            } else {
                refs.push(bare_ref(child.as_str()?));
            }
        }
    } else if let Some(item) = value.get("item").and_then(|value| value.as_str()) {
        refs.push(runtime::IngredientRef::item(item).reference());
    } else if let Some(tag) = value.get("tag").and_then(|value| value.as_str()) {
        refs.push(runtime::IngredientRef::tag(tag).reference());
    } else {
        refs.push(bare_ref(value.as_str()?));
    }
    Some(reference_text(&refs))
}

/// Reads every `occultism:spirit_fire` recipe; one whose ingredient or result names nothing the
/// game knows is counted as skipped instead of matching everything.
fn build(host: &dyn Host) -> Cached {
    let recipes = host.recipes_of_type(RECIPE_TYPE);
    let mut rules = Vec::new();
    let mut skipped = 0;
    for recipe in recipes.iter() {
        let ingredient =
            ingredient_text(recipe).and_then(|text| host.recipe_ingredient(&text).ok().flatten());
        let result = recipe
            .json
            .get("result")
            .map(|value| value.to_string())
            .and_then(|text| host.recipe_result(&text).ok().flatten());
        match (ingredient, result) {
            (Some(ingredient), Some(result)) => rules.push(runtime::Rule {
                inputs: ingredient.order,
                result: result.item().id(host),
            }),
            _ => skipped += 1,
        }
    }
    Cached {
        files: recipes.len(),
        skipped,
        table: runtime::Table::new(rules),
    }
}

/// Whether the block at `pos` can be replaced (`BlockState.canBeReplaced`), which decides
/// whether the fire lands on it or on the clicked face. A position outside the world or in an
/// unloaded chunk has no state and so is not replaceable, as the 0.1.0 explicit bounds check
/// concluded.
fn replaceable(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> bool {
    level
        .block_state(host, pos)
        .and_then(|state| host.block_states().replaceable(state))
        .unwrap_or(false)
}

/// The hand a click arrived on, as the harvest guard needs it. Horizon calls
/// `right_click_block` once per hand; vanilla's block `useItemOn` runs for the
/// main hand and only reaches the off hand when the main hand's action did not
/// consume the click, so a click that writes the block must act on one hand.
fn hand(hand: Hand) -> runtime::ClickHand {
    match hand {
        Hand::Main => runtime::ClickHand::Main,
        Hand::Off => runtime::ClickHand::Off,
    }
}

/// The mode the clicking player is in, or `None` when the game cannot name it.
/// The click guard **fails closed**: a click whose player or mode is unknown
/// changes nothing, because an unauthorised write is worse than a lost click.
fn click_mode(host: &dyn Host, player: u128) -> Option<Mode> {
    let mode = host.player(player)?.game_mode(host)?;
    Some(match mode {
        GameMode::Survival => Mode::Survival,
        GameMode::Creative => Mode::Creative,
        GameMode::Adventure => Mode::Adventure,
        GameMode::Spectator => Mode::Spectator,
    })
}

/// A right click on a block: harvest a ripe datura, paint the fire with a chalk, or strike
/// the datura lying on the clicked block alight (NeoForge's
/// `PlayerInteractEvent.RightClickBlock`, which is what upstream's
/// `PlayerEventHandler.spiritFire` listens on: before the block's own use and
/// before the item's).
pub fn right_click(host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if event.cancelled {
        return Flow::Continue;
    }
    let click = event.click;
    let pos = click.pos;
    let face = click.hit.face;
    let HostSide::Server(server) = host else {
        return Flow::Continue;
    };
    // Horizon calls this hook for a spectator too (the hook's own doc says so:
    // "for a spectator too, the server then only lets one open a menu"), so the
    // mode is checked before anything is written: a spectator recolours
    // nothing, lights nothing, consumes no datura and harvests no crop.
    if !click_mode(&*server, click.player).is_some_and(Mode::may_change) {
        return Flow::Continue;
    }
    let Some(level) = server.level(server.dimension()) else {
        return Flow::Continue;
    };
    // The crop harvest comes first: an empty hand is the normal way to click it.
    let clicked = level
        .block(&*server, pos)
        .map(|block| block.id(&*server))
        .unwrap_or_default();
    let crop_age = level.block_property(&*server, pos, runtime::AGE);
    if let Some(harvest_age) = crop_age
        .as_deref()
        .and_then(|age| runtime::harvest_age(&clicked, age))
    {
        // The hook is called once per hand and the harvest writes the block, so
        // the off hand's turn would harvest the crop the main hand just
        // replanted (measured in game: one click dropped 2 fruit and 6 seeds,
        // 2026-11-21). Vanilla never reaches the off hand after a main-hand
        // success; the guard makes this path equivalent.
        if !hand(click.hand).may_harvest() {
            return Flow::Continue;
        }
        if !harvest(&mut *server, level, pos, click, harvest_age) {
            // The loot roll was refused: nothing changed, so the click is not claimed.
            return Flow::Continue;
        }
        // Upstream answers `SUCCESS` for a harvest, so the crop's own use and the
        // held item's never run.
        *event.cancellation_result = InteractionResult::Success;
        return Flow::Cancel;
    }
    let Some(stack) = click.stack else {
        return Flow::Continue;
    };
    let item = stack.item().id(server);
    let fire = server.block_key(SPIRIT_FIRE);
    if let Some(colour) = runtime::chalk_color(&item) {
        if level.block(&*server, pos) == fire {
            server.set_block_property(pos, COLOR, colour);
            // Upstream answers `sidedSuccess` for any chalk on the fire, whether or not the
            // colour changed.
            *event.cancellation_result = InteractionResult::Success;
            return Flow::Cancel;
        }
        return Flow::Continue;
    }
    if !runtime::starts_fire(&item) {
        return Flow::Continue;
    }
    let (min, max) = runtime::datura_box(pos);
    let Some(entities) = level.entities_in(&*server, min, max) else {
        return Flow::Continue;
    };
    let datura: Vec<EntityRef> = entities
        .into_iter()
        .filter(|entity| {
            server
                .item_entity(*entity)
                .and_then(|item| item.stack(&*server))
                .is_some_and(|stack| stack.item().id(&*server) == DATURA)
        })
        .collect();
    if datura.is_empty() {
        return Flow::Continue;
    }
    let target = runtime::fire_target(pos, face, replaceable(&*server, level, pos));
    if replaceable(&*server, level, target)
        && let Some(state) = server.block_state_id(SPIRIT_FIRE)
    {
        let _ = level.set_block(
            &mut *server,
            target,
            state,
            BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
        );
    }
    // Upstream consumes every datura in the box whatever the target was.
    for entity in datura {
        entity.discard(&mut *server);
    }
    // ... and claims the click, so the lighter's own use never runs.
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

/// Upstream's `IReplantableCrops.onHarvest`: the ripe crop's own drops (its loot table
/// rolled with the held stack and the player as the breaker), the block replanted at
/// `harvest_age`, each stack dropped at the block's centre, and the harvest's exhaustion.
///
/// `Block.getDrops(state, level, pos, null, player, stack)` is `level_block_drops`; the
/// stacks then go through `Containers.dropItemStack(level, x+0.5, y+0.5, z+0.5, stack)`,
/// which is `level_spawn_item` at the block's centre with vanilla's pickup delay of 10.
fn harvest(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    click: &BlockClick<'_>,
    harvest_age: &str,
) -> bool {
    let player = host.player(click.player);
    // A refused roll (the level's random is not writable in this scope) returns
    // `None`. The crop is left ripe then: losing a harvest's yield would be
    // worse than a click that does nothing, and the next click can roll again.
    let Some(stacks) = level
        .block_drops(host, pos, click.stack, player.map(|player| player.entity()))
        .ok()
        .flatten()
    else {
        return false;
    };
    // The roll above sees the crop at its ripe age, as upstream's does: the reset comes after.
    host.set_block_property(pos, runtime::AGE, harvest_age);
    let centre = [
        f64::from(pos[0]) + 0.5,
        f64::from(pos[1]) + 0.5,
        f64::from(pos[2]) + 0.5,
    ];
    for stack in stacks {
        let _ = level.spawn_item(host, centre[0], centre[1], centre[2], stack, 10);
    }
    if let Some(player) = player {
        player.add_exhaustion(host, HARVEST_EXHAUSTION);
    }
    true
}

/// One item entity's tick ended: convert it when it stands in a spirit fire and a recipe accepts
/// it (upstream's `entityInside`).
pub fn entity_tick_post(host: &mut dyn ServerHost, event: &mut EntityTickPostEvent<'_>) {
    if event.kind != "minecraft:item" {
        return;
    }
    let Some(level) = host.level(event.dimension) else {
        return;
    };
    let Some(entity) = host.entity(event.entity) else {
        return;
    };
    let Some(pos) = entity.block_position(host) else {
        return;
    };
    if level.block(&*host, pos) != host.block_key(SPIRIT_FIRE) {
        return;
    }
    let Some(item) = host.item_entity(entity) else {
        return;
    };
    let Some(stack) = item.stack(host) else {
        return;
    };
    let item_id = stack.item().id(host);
    let Some(conversion) = with_table(host, |cached| cached.table.convert(&item_id, stack.count()))
    else {
        return;
    };
    let Some(key) = Identifier::parse(&conversion.result)
        .ok()
        .and_then(|id| host.item_key(&id))
    else {
        return;
    };
    let Some(result) = ItemStack::new(key, conversion.count) else {
        return;
    };
    if item.set_stack(host, result).unwrap_or(false) {
        record(level, event.dimension, pos, &conversion);
    }
}

/// One conversion, for `/occultism info`.
fn record(level: LevelRef, dimension: &str, pos: [i32; 3], conversion: &Conversion) {
    let mut guard = STATS.lock().expect("occultism stats poisoned");
    guard.record(RecentConversion {
        world: level.epoch().get(),
        dimension: dimension.to_owned(),
        pos,
        result: conversion.result.clone(),
        count: conversion.count,
    });
}

/// Whether the mod hears entity ticks: without this answer the game tells none of them.
pub fn listen(event: &mut EntityTickListenedEvent) {
    event.listened = true;
}

/// A block broke and `event.drops` are what its loot table rolled: a hand-broken
/// `short_grass`/`tall_grass` has the jar's 2% chance of a datura seed
/// (`data/occultism/loot_modifiers/datura_seed_from_grass.json`, `…_from_tall_grass.json`:
/// NeoForge's `add_item` global loot modifier, which is this hook).
pub fn loot_drops(host: &mut dyn ServerHost, event: &mut LootDropsEvent<'_>) {
    let context = event.context;
    let Some((_pos, state)) = context.block else {
        return;
    };
    // `minecraft:short_grass[<properties>]`: the block's id before the state's properties.
    let Some(text) = host.block_state_string(state) else {
        return;
    };
    let block = text.split('[').next().unwrap_or(&text);
    let tool = context.tool.map(|stack| stack.item().id(&*host));
    if !runtime::seed_source(block, tool.as_deref()) {
        return;
    }
    let Some(level) = host.level(context.dimension) else {
        return;
    };
    // `random.nextFloat() < 0.02`, drawn from the level's own stream in vanilla's order.
    let Some(draw) = level.with_random(host, |random| random.next_float()) else {
        return;
    };
    if !runtime::seed_rolls(draw) {
        return;
    }
    let Some(key) = Identifier::parse(runtime::DATURA_SEEDS)
        .ok()
        .and_then(|id| host.item_key(&id))
    else {
        return;
    };
    if let Some(stack) = ItemStack::new(key, 1) {
        event.drops.push(stack);
    }
}

/// `/occultism info|recipes`.
pub fn declare_commands(commands: &mut Vec<Command>) {
    // `/occultism info` names the block at coordinates the caller chooses, so it
    // is registered at the gamemaster level (2), as `/data` is.
    commands.push(Command::op(
        "occultism info <pos:block_pos>",
        runtime::INFO_LEVEL,
    ));
    commands.push(Command::new("occultism recipes"));
}

/// Run one of the commands above and take it.
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let literals = &event.command.literals;
    if literals.first().map(String::as_str) != Some("occultism") {
        return Flow::Continue;
    }
    let line = match literals.get(1).map(String::as_str) {
        Some("info") => {
            // The usage is registered at `INFO_LEVEL` and the source is checked
            // again here: a source that reaches this body by another route (a
            // function, a future dispatcher change) is still refused, and the
            // refusal names no block.
            if !runtime::info_permitted(event.source.permission().level()) {
                runtime::INFO_DENIED.to_owned()
            } else if let Some(CommandArgumentValue::BlockPos(pos)) = event.command.argument("pos")
            {
                let pos = *pos;
                let (block, colour, rules, files, world, dimension) = {
                    let view: &dyn Host = &*host;
                    let level = view.level(view.dimension());
                    let block = level
                        .and_then(|level| level.block(view, pos))
                        .map(|block| block.id(view))
                        .unwrap_or_default();
                    let colour = level
                        .and_then(|level| level.block_property(view, pos, COLOR))
                        .unwrap_or_else(|| "none".to_owned());
                    let (rules, files) =
                        with_table(view, |cached| (cached.table.rules().len(), cached.files));
                    let world = level.map(|level| level.epoch().get()).unwrap_or(0);
                    let dimension = level
                        .and_then(|level| level.dimension().id(view))
                        .unwrap_or_default();
                    (block, colour, rules, files, world, dimension)
                };
                let last = STATS
                    .lock()
                    .expect("occultism stats poisoned")
                    .last(world, &dimension, pos)
                    .map_or_else(
                        || "none".to_owned(),
                        |recent| format!("{} x{}", recent.result, recent.count),
                    );
                // A position holding no spirit fire is answered without naming
                // the block, so the readout is not a world probe.
                runtime::info_line(pos, &block, &colour, &last, rules, files)
            } else {
                INFO_USAGE.to_owned()
            }
        }
        Some("recipes") => {
            let (rules, files, items, skipped) = {
                let view: &dyn Host = &*host;
                with_table(view, |cached| {
                    (
                        cached.table.rules().len(),
                        cached.files,
                        cached.table.inputs(),
                        cached.skipped,
                    )
                })
            };
            format!(
                "Spirit fire recipes: {rules} rules, {files} files, {items} items, {skipped} skipped."
            )
        }
        _ => USAGE.to_owned(),
    };
    let _ = host.commands().feedback(Text::literal(line));
    Flow::Cancel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_reference_is_a_json_array() {
        assert_eq!(
            reference_text(&["occultism:chalk_white_impure".to_owned()]),
            "[\"occultism:chalk_white_impure\"]"
        );
    }

    #[test]
    fn a_list_keeps_every_reference_in_order() {
        assert_eq!(
            reference_text(&["minecraft:book".to_owned(), "#c:feathers".to_owned()]),
            "[\"minecraft:book\",\"#c:feathers\"]"
        );
    }

    #[test]
    fn a_quote_cannot_break_the_array() {
        assert_eq!(reference_text(&["a\"b".to_owned()]), "[\"a\\\"b\"]");
    }

    #[test]
    fn a_bare_tag_keeps_its_hash() {
        assert_eq!(
            bare_ref("#occultism:books/books_for_empty"),
            "#occultism:books/books_for_empty"
        );
        assert_eq!(bare_ref("minecraft:diorite"), "minecraft:diorite");
    }

    #[test]
    fn the_info_command_is_declared_for_operators() {
        // The readout names the block at coordinates the caller chooses, so the
        // usage itself must be level 2 — the dispatcher then refuses a player
        // before the body runs.
        let mut commands = Vec::new();
        declare_commands(&mut commands);
        let info = commands
            .iter()
            .find(|command| command.usage.starts_with("occultism info "))
            .expect("info is declared");
        assert_eq!(info.level, runtime::INFO_LEVEL);
        assert_eq!(
            info.level, 2,
            "a non-operator must not read a block at chosen coordinates"
        );
        let recipes = commands
            .iter()
            .find(|command| command.usage.starts_with("occultism recipes"))
            .expect("recipes is declared");
        assert_eq!(
            recipes.level, 0,
            "the recipe table names no world state, so it stays a player command"
        );
    }
}
