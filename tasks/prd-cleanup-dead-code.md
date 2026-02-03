# PRD: Quill TUI Dead Code Cleanup

## Overview
Remove all unused functions and constants from the Quill TUI codebase to eliminate compiler warnings and reduce maintenance burden.

## Problem Statement
The codebase contains 60+ lines of dead code across multiple modules, causing 18 compiler warnings. This code duplication (actions wrapping Document methods) adds confusion and maintenance overhead.

## Goals
1. Eliminate all 18 compiler warnings
2. Remove unused functions without breaking functionality
3. Simplify the actions module structure
4. Maintain all existing functionality

## Non-Goals
- Refactoring working code
- Adding new features
- Changing public API
- Performance optimization (separate task)

## Success Criteria
- `cargo check` exits with 0 warnings
- `cargo test` passes (2 existing tests)
- `cargo build --release` succeeds
- Application runs and functions identically

## Technical Approach

### Phase 1: Remove Unused Action Functions
Delete from `src/actions/annotation.rs`:
- `create_annotation()` - duplicates `Annotation::new()` + `doc.add_annotation()`
- `delete_annotation()` - wraps `doc.remove_annotation()`
- `toggle_resolved()` - wraps `doc.toggle_resolved()`
- `update_comment()` - never called
- `update_category()` - never called
- `update_severity()` - never called

### Phase 2: Remove Unused Navigation Functions
Delete from `src/actions/navigation.rs`:
- `next_annotation_offset()` - never called
- `prev_annotation_offset()` - never called
- `current_annotation_index()` - never called

Keep `annotation_offset_by_index()` - this IS used in `app.rs`

### Phase 3: Clean Up Module Exports
Fix `src/actions/mod.rs`:
- Remove wildcard `pub use annotation::*`
- Keep only `pub use navigation::annotation_offset_by_index`

Fix `src/io/mod.rs`:
- Remove unused `ExportDocument` from exports (only used internally)

### Phase 4: Remove Unused Model Methods
Delete from `src/model/text_range.rs`:
- `len()` - never called
- `is_empty()` - never called
- `overlaps()` - never called

Delete from `src/model/annotation.rs`:
- `Category::short()` - never called
- `Annotation::with_category()` - never called
- `Annotation::with_severity()` - never called

Delete from `src/model/document.rs`:
- `annotation_at()` - never called

Delete from `src/app.rs`:
- `cursor_offset()` - never called externally
- `sorted_annotation_ids()` - never called

### Phase 5: Remove Unused UI Constants
Delete from `src/ui.rs`:
- `BASE` color constant - never used

Fix unused variable:
- Change `for (line_idx, line_text)` to `for (_line_idx, line_text)` or remove enumeration

## Risks
- **Low**: Removing code that appears unused but is called via macro/reflection
  - Mitigation: Rust doesn't have reflection, all calls are static
- **Low**: Breaking internal module dependencies
  - Mitigation: `cargo check` will catch any missing imports

## Timeline
- Estimated: 30 minutes
- Verification: 10 minutes

## Tasks

1. Delete unused functions in `src/actions/annotation.rs`
2. Delete unused functions in `src/actions/navigation.rs`
3. Update `src/actions/mod.rs` exports
4. Update `src/io/mod.rs` exports
5. Delete unused methods in `src/model/text_range.rs`
6. Delete unused methods in `src/model/annotation.rs`
7. Delete unused method in `src/model/document.rs`
8. Delete unused methods in `src/app.rs`
9. Remove unused constant and fix unused variable in `src/ui.rs`
10. Run `cargo check` to verify 0 warnings
11. Run `cargo test` to verify tests pass
12. Run `cargo build --release` to verify release build
