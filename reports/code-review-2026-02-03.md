# Daily Report - 2026-02-03

## Project
Quill TUI - Terminal-based annotation tool for text documents (Rust/Ratatui)

## Key Metrics
- Lines of code: 1,673
- Compiler warnings: 18 (all from dead code)
- Test coverage: 2 tests (export format validation only)
- Build status: Compiles successfully

## Critical Issues

### 1. UTF-8 Boundary Panic Risk (SECURITY/CRASH)
**Location:** `src/app.rs:230`, `src/actions/annotation.rs:18`
**Impact:** Application will panic/crash if annotation offsets fall mid-character in multi-byte UTF-8 sequences
**Details:** String slicing `doc.content[range.start_offset..range.end_offset]` without `is_char_boundary()` validation
**Frequency:** Will occur with any non-ASCII content (international users)

### 2. Dead Code: 60+ Lines of Unused Functions
**Location:** `src/actions/annotation.rs`, `src/actions/navigation.rs`
**Impact:** Code bloat, maintenance burden, 18 compiler warnings
**Details:**
- 5 unused action functions: `create_annotation()`, `delete_annotation()`, `toggle_resolved()`, `update_comment()`, `update_category()`, `update_severity()`
- 3 unused navigation functions: `next_annotation_offset()`, `prev_annotation_offset()`, `current_annotation_index()`
- These duplicate methods already on `Document` struct

### 3. O(n×m) Per-Frame Rendering Performance
**Location:** `src/ui.rs:139-145`
**Impact:** UI lag on large documents with many annotations
**Details:** For every character rendered, iterates through all annotations to check containment. 10KB doc with 50 annotations = 500,000 checks per frame at 60fps

## High Priority Issues

### 4. annotations_sorted() Called Multiple Times Per Frame
**Location:** `src/ui.rs:110, 199`
**Impact:** Redundant O(n log n) sorting twice per render
**Fix:** Compute once, pass as reference

### 5. Path Traversal via Unsanitized Input
**Location:** `src/io/load.rs:8-21`, `src/main.rs:36-45`
**Impact:** Users can load arbitrary files, content exports to predictable `~/.quill/document.json`
**Fix:** Add directory allowlist or confirmation for non-CWD paths

### 6. Public Fields in App Struct (Encapsulation Violation)
**Location:** `src/app.rs:33-65`
**Impact:** 13 public fields allow invalid state, blocks safe refactoring
**Fix:** Make private, expose via methods

### 7. Duplicate Picker Navigation Logic
**Location:** `src/main.rs:241-276, 279-315`
**Impact:** DRY violation, bug fixes must be applied twice
**Fix:** Extract generic SelectionList helper

## Medium Priority Issues

### 8. offset_to_cursor() Linear Search
**Location:** `src/app.rs:125-131`
**Fix:** Use binary_search() for O(log n)

### 9. Multi-Modal Picker Complexity
**Location:** `src/app.rs:7-15`, various handlers
**Fix:** Combine Severity + Category pickers into single dialog

### 10. String Clones in Sidebar Per Frame
**Location:** `src/ui.rs:208-222`
**Fix:** Cache truncated preview strings

## User Feedback
- (Self-observed) Visual mode selection works but no visual feedback when nothing selected
- (Self-observed) Help overlay could show keybinding for current mode
- (Self-observed) Export success message disappears too quickly

## Recommendations (Priority Order)

1. **Fix UTF-8 boundary validation** - Prevents crashes on international text
2. **Delete dead code** - Eliminates 18 warnings, reduces maintenance
3. **Cache sorted annotations** - Single biggest performance win
4. **Build annotation index** - O(1) lookup instead of O(n) per character
5. **Encapsulate App fields** - Enables safe refactoring for other fixes

## Constraints
- No database (file-based only)
- Must maintain JSON compatibility with macOS Quill app
- Single-binary distribution preferred
