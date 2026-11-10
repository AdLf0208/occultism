# occultism (Horizon port)

A Horizon port of [occultism](klikli-dev/occultism) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- The block.** `occultism:spirit_fire` on the **`minecraft:torch`** template
- (vanilla's only fire-like class with no fire damage and a sturdy-face
- survival rule): no collision, instabreak, light 12, wool sound, purple map
- colour, empty drops, a declinable `color` state with the jar's 18 values
- (default `white`), and the jar's animated model/blockstate.
- Ignition.** Flint and steel or a fire charge struck on a block with a
- `#occultism:start_spiritfire` item entity (datura) inside upstream's
- `AABB(-1,-1,-1,1,1,1)` around the clicked block's centre: every datura is

## Not yet ported

- | Declared library | What it would have provided | What this slice does |
- |---|---|---|
- | **Modonomicon** (required) | the guide book, its 260 pages, ritual and recipe pages | nothing: no guide-book reader at the pin; the knowledge is in this file |
- | **Curios** (required) | accessory slots (familiar ring, spirit lamp) | nothing: no accessory-slot API (planned 0.4) |
- | **GeckoLib** | the 21 animated spirit models | nothing: no spirits in this slice |
- | **SmartBrainLib** | the spirits' job AI | nothing: no spirits in this slice |
- Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: klikli-dev/occultism. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
