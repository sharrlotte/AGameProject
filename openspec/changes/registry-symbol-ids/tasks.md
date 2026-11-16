## 1. Symbol infrastructure

- [x] 1.1 Add `lasso` dependency; create `Symbols` resource + `Symbol` newtype
- [x] 1.2 Intern every `block.ron` key at config load; build keyed-map schema
- [x] 1.3 Update `config/block.rs` mirror types for keyed `.ron` and symbol
  lookup

## 2. Registry rework

- [x] 2.1 Replace `BlockType`/`BlockGroup` enums with interned symbols in
  `content/block/component.rs` (BlockRegistry keyed by Symbol)
- [x] 2.2 Update editor palette, selection counts, drops, item registry to
  symbol keys
- [x] 2.3 Update spawn/lookup call sites (ship spawn, mission, drops) to
  symbol lookup
- [x] 2.4 Add a fresh save-load smoke check and document the breakage; update
  `docs/defects.md`

## 3. Verification

- [x] 3.1 `cargo check -p core` green
- [x] 3.2 `docs/config.md`, `docs/assets.md`, `docs/defects.md` sync'd with the
  keyed-map schema and symbol strategy
