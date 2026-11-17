## ADDED Requirements

### Requirement: Grouped conditional compilation blocks
Conditional compilation attributes (`#[cfg(debug_assertions)]`, `#[cfg(feature = "dev")]`, `#[cfg(test)]`) MUST be grouped into minimal module blocks rather than repeated on individual imports and items.

#### Scenario: Debug or test-only code in a module
- **WHEN** a file contains multiple `#[cfg(debug_assertions)]`, `#[cfg(feature = "dev")]`, or `#[cfg(test)]` items
- **THEN** those items are grouped into a single `#[cfg(...)] mod ...` block per condition
