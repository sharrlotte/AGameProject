## ADDED Requirements

### Requirement: Editor left-panel tabs are scrollable
All three left-panel tabs (`BlockListPanel`, `ShipListPanel`, `AiPanel`) SHALL use `spawn_scroll_area` with `flex_shrink: 0.0` on fixed-height list items (`BlockSelectButton`, ship list rows, `AiPresetButton`) so long lists overflow and scroll with a visual scrollbar.

#### Scenario: Long block list scrolls
- **WHEN** the `Blocks` tab contains more blocks than fit in the left panel height
- **THEN** block buttons keep their 60 px height, mouse-wheel scrolling moves through the list, and the scrollbar thumb updates

#### Scenario: Long ship list scrolls without losing scrollbar on refresh
- **WHEN** `refresh_ship_list` repopulates ship rows into `ShipListContent`
- **THEN** ship rows keep their height, the list scrolls vertically, and the sibling `ScrollbarTrack` remains intact

#### Scenario: Long AI preview scrolls
- **WHEN** the `AI` tab's preset buttons and RON preview exceed the left panel height
- **THEN** the `AI` tab scrolls vertically and displays its scrollbar
