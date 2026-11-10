// Declare `/occultism info|recipes` (API 8).
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::occultism_state::on_register_commands(commands);
}
