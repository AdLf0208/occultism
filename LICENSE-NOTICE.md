# Licence notice — Occultism for Horizon

This package is a port of **Occultism 1.225.0** (`occultism-1.21.1-neoforge-1.225.0.jar`).

- Upstream: <https://github.com/klikli-dev/occultism>
- Commit: `91de0d2a46e05ce315cc5afd292d266d4f46e7ab`, tag `release/v1.21.1-1.225.0`
- Licence: **MIT**, `Copyright 2020 klikli-dev (Occultism)`

The port is a clean-room Rust rewrite: the upstream source was read for behaviour,
and the carried `assets/` and `data/` files (blockstates, models, textures, lang,
recipes, loot tables, tags) are the mod's own MIT data, read from the exact ATM10
jar. No Java code is translated or copied.

The package's `LICENSE` reproduces the MIT text and the upstream copyright line;
the port's own copyright line is added beside it. Redistribution and modification
are permitted under the same MIT terms.

Port to Horizon by AdLf0208. Local-only until the maintainer approves publishing
(ARCHITECTURE.md §8 of the campaign).
