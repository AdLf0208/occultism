// The bundle hears entity ticks (API 146): without this answer the game tells
// no entity's tick at all.
fn observe_entity_tick_listened(game: &crate::game::Game<'_>, listened: &mut bool) {
    if crate::game::occultism_state::listens_to_entity_ticks(game) {
        *listened = true;
    }
}
