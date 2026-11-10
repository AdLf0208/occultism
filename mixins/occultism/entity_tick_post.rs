// One entity ticked: an item entity standing in spirit fire is converted when
// a carried `occultism:spirit_fire` recipe accepts it (API 146).
fn observe_entity_tick_post(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    entity: u32,
    kind: &str,
) {
    crate::game::occultism_state::on_entity_tick_post(game, dimension, entity, kind);
}
