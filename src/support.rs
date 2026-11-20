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
    BlockUpdateFlags, Command, CommandArgumentValue, EntityRef, EntityTickListenedEvent,
    EntityTickPostEvent, Flow, Host, HostBlockStates, HostSide, Identifier, InteractionResult,
    ItemStack, LevelRef, Recipe, RightClickBlockEvent, RunModCommandEvent, ServerHost, Text,
};

use crate::runtime::{
    self, Conversion, DATURA, RECIPE_TYPE, Recent, RecentConversion, SPIRIT_FIRE,
};

/// The block state property the chalks set (`SpiritFireBlock.COLOR`).
const COLOR: &str = "color";
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

/// A right click with an item on a block: paint the fire with a chalk, or strike the datura
/// lying on the clicked block alight (NeoForge's `PlayerInteractEvent.RightClickBlock`, which is
/// what upstream's `PlayerEventHandler.spiritFire` listens on: before the block's own use and
/// before the item's).
pub fn right_click(host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if event.cancelled {
        return Flow::Continue;
    }
    let click = event.click;
    let pos = click.pos;
    let face = click.hit.face;
    let Some(stack) = click.stack else {
        return Flow::Continue;
    };
    let HostSide::Server(server) = host else {
        return Flow::Continue;
    };
    let Some(level) = server.level(server.dimension()) else {
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

/// `/occultism info|recipes`.
pub fn declare_commands(commands: &mut Vec<Command>) {
    commands.push(Command::new("occultism info <pos:block_pos>"));
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
            let Some(CommandArgumentValue::BlockPos(pos)) = event.command.argument("pos") else {
                let _ = host.commands().feedback(Text::literal(INFO_USAGE));
                return Flow::Cancel;
            };
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
            Text::literal(format!(
                "Spirit Fire at {pos:?}: block={block}, colour={colour}, last={last}, rules={rules}/{files}"
            ))
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
            Text::literal(format!(
                "Spirit fire recipes: {rules} rules, {files} files, {items} items, {skipped} skipped."
            ))
        }
        _ => Text::literal(USAGE),
    };
    let _ = host.commands().feedback(line);
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
}
