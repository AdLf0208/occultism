//! Occultism for Horizon — slice 1: the spirit-fire conversion loop, as one `impl Hooks`.
//!
//! Ported from Occultism 1.225.0 (`klikli-dev/occultism`, MIT); see `FINDINGS.md` for the
//! behaviour read from the jar and the upstream repository, and `notes.md` for the hook map,
//! the gaps and the migration to the SDK format.
//!
//! The loop is small and every moment belongs to the whole server, so this file needs no
//! Part: a right click with a lighter strikes the datura lying on the clicked block alight
//! ([`Mod::right_click_block`]), a chalk on the fire recolours it, each item entity's tick
//! converts it when it stands in the fire ([`Mod::entity_tick_post`], which is only told when
//! [`Mod::entity_tick_listened`] asks), and `/occultism info|recipes` are the port's readout
//! surface. The rules — the recipe table, the lighter and chalk tables, the click geometry —
//! live in [`runtime`], which has no `mod_api` type and is unit-tested by `cargo test` alone.
//! [`support`] is the glue that reads the game and writes it back.
//!
//! # Where the retired container's hooks went
//!
//! The 2026-11-17 mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`: the plan, the
//! injection points and the per-hook helper files are gone. A hook is a typed method here.
//!
//! | `.hnmod` hook (API) | here |
//! |---|---|
//! | the `content_root` injection (AE2 pattern, embedded content) | gone: one flat `.hmod` carries `assets/` and `data/` itself |
//! | `use_item_on` (13, after the block's own use) | [`Mod::right_click_block`] (NeoForge's `PlayerInteractEvent.RightClickBlock`, the pre-block route upstream itself listens on) |
//! | `entity_tick_listened` (146) | [`Mod::entity_tick_listened`] |
//! | `entity_tick_post` (146) | [`Mod::entity_tick_post`] |
//! | `register_commands` (8) | [`Mod::register_commands`] |
//! | `perform_command` (8, split line) | [`Mod::run_mod_command`] (typed `CommandInvocation`) |
pub mod runtime;
pub mod support;

use std::sync::Arc;

use mod_api::{
    self, CraftingGrid, CraftingRecipeHandler, CraftingRecipeValue, EntityTickListenedEvent,
    EntityTickPostEvent, Flow, Hooks, HostSide, ItemStack, LootDropsEvent, RegisterCommandsEvent,
    Registrar, RegistrationContext, RightClickBlockEvent, RunModCommandEvent, ServerHost,
};

/// The serializer the carried `occultism:spirit_fire` files need. The port reads
/// the recipes' raw JSON through the recipe service, so the decoded value is
/// empty and no crafting-grid callback is used: this only admits the files.
struct SpiritFireSerializer;

impl CraftingRecipeHandler for SpiritFireSerializer {
    fn decode(&self, _json: &str) -> Result<CraftingRecipeValue, String> {
        Ok(CraftingRecipeValue::new(Vec::new()))
    }

    fn write(&self, _value: &CraftingRecipeValue) -> Result<String, String> {
        Ok("{}".to_owned())
    }

    fn matches(&self, _value: &CraftingRecipeValue, _grid: CraftingGrid) -> bool {
        false
    }

    fn assemble(&self, _value: &CraftingRecipeValue, _grid: CraftingGrid) -> Option<ItemStack> {
        None
    }

    fn remaining_items(
        &self,
        _value: &CraftingRecipeValue,
        _grid: CraftingGrid,
    ) -> Option<Vec<Option<ItemStack>>> {
        None
    }
}

/// The mod keeps its state (the per-fire conversion counters) in `support`; one loop serves
/// every fire in every dimension.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the carried `occultism:spirit_fire` recipe type and a serializer
    /// for it. The data files name that type; without a registered serializer the
    /// game refuses the package at load (`unregistered recipe serializer
    /// occultism:spirit_fire`). The port reads the recipes through the recipe
    /// service, so the serializer needs no crafting handler.
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        r.register_recipe_type("occultism:spirit_fire");
        r.add_recipe_serializer(
            "occultism:spirit_fire",
            "occultism:spirit_fire",
            Some(Arc::new(SpiritFireSerializer)),
        );
    }

    /// `/occultism info|recipes`: the fire's readout and the loaded recipe table.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::declare_commands(event.commands);
    }

    /// Run one of the commands above and take it.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }

    /// A right click on a block: chalk paints a spirit fire, a lighter strikes the datura
    /// lying on the clicked block alight.
    fn right_click_block(&self, host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        support::right_click(host, event)
    }

    /// Whether the server tells this mod any entity's tick: without this answer none is told.
    fn entity_tick_listened(
        &self,
        _host: &mut dyn ServerHost,
        event: &mut EntityTickListenedEvent,
    ) {
        support::listen(event);
    }

    /// One entity ticked: an item entity standing in spirit fire is converted when a carried
    /// `occultism:spirit_fire` recipe accepts it (upstream's `entityInside`).
    fn entity_tick_post(&self, host: &mut dyn ServerHost, event: &mut EntityTickPostEvent<'_>) {
        support::entity_tick_post(host, event);
    }

    /// A block broke and the game built its drops: a hand-broken grass carries the jar's 2%
    /// chance of a datura seed (upstream's `add_item` loot modifier).
    fn loot_drops(&self, host: &mut dyn ServerHost, event: &mut LootDropsEvent<'_>) {
        support::loot_drops(host, event);
    }
}
