## 1. Symbol infrastructure

- [ ] 1.1 Add `lasso` dependency; create `Symbols` resource + `Symbol` newtype
- [ ] 1.2 Intern every `blocks.ron` key at config load; build keyed-map schema
- [ ] 1.3 Update `config/blocks.rs` mirror types for keyed `.ron` and symbol
  lookup

## 2. Registry rework

- [ ] 2.1 Replace `BlockType`/`BlockGroup` enums with interned symbols in
  `content/block/components.rs` (BlockRegistry keyed by Symbol)
- [ ] 2.2 Update editor palette, selection counts, drops, item registry to
  symbol keys
- [ ] 2.3 Update spawn/lookup call sites (ship spawn, mission, drops) to
  symbol lookup
- [ ] 2.4 Add a fresh save-load smoke check and document the breakage; update
  `docs/defects.md`

## 3. Verification

- [ ] 3.1 `cargo check -p core` green
- [ ] 3.2 `docs/config.md`, `docs/assets.md`, `docs/defects.md` sync'd with the
  keyed-map schema and symbol strategy
