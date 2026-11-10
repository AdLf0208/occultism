// A right click with an item on a block: chalk paints spirit fire, flint and
// steel (or a fire charge) lights the datura lying on it (API 13).
fn observe_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::occultism_state::on_use_item_on(game, stack, pos, face, claimed);
}
