# Cosmic Notebook - Implementation Plan

## Project Overview
Cosmic Notebook is a lightweight Markdown viewer/editor built with libCosmic for the Fedora Operating System. It provides a responsive, minimal-footprint experience for editing Markdown files with VS Code-like functionality.

### Design Philosophy
- **Minimal footprint**: Keep binary size under 15MB and runtime memory under 100MB for typical usage
- **Native feel**: Integrate seamlessly with COSMIC desktop and Fedora ecosystem
- **Responsive editing**: Sub-50ms input latency for all editing operations
- **Data safety**: Never lose user work through crashes, power loss, or conflicts
- **Keyboard-first**: Every operation accessible via keyboard shortcuts

### Target User Profiles
1. **Technical writers** documenting software projects
2. **Developers** editing README files and documentation
3. **Note-takers** using Markdown for personal knowledge management
4. **Students** writing assignments and notes in Markdown format

---

## Phase 1: Core Infrastructure & Setup

### 1.1 Project Foundation
- **Objective**: Establish the basic project structure and dependencies
- **Tasks**:
  - Configure Cargo.toml with required dependencies (libCosmic, file system libraries, text editing libraries)
  - Set up module organization (main.rs, ui/, state/, file_handler/, editor/)
  - Implement basic error handling framework
  - Create application state management structure
- **Dependencies**: libCosmic, tokio, serde, notify (for file watching)
- **Success Criteria**: Project builds successfully with no errors

#### 1.1.1 Cargo.toml Dependencies Specification

**Core Dependencies (Required)**:
| Crate | Version | Purpose | Size Impact |
|-------|---------|---------|-------------|
| libcosmic | git (pop-os) | GUI framework with iced backend | Large - core framework |
| cosmic-config | git (pop-os) | Configuration management integration | Small |
| cosmic-theme | git (pop-os) | Theme integration | Small |
| tokio | 1.35+ | Async runtime for file operations | Medium |
| ropey | 1.6+ | Efficient rope data structure for text | Small |
| serde | 1.0 | Serialization framework | Small |
| notify | 6.1+ | File system watching | Small |

**Recommended libCosmic Features**:
- `tokio` - Async runtime integration
- `winit` - Window management
- `wgpu` - GPU-accelerated rendering
- `a11y` - Accessibility support
- `multi-window` - For potential future dialogs

**Secondary Dependencies**:
| Crate | Purpose | When to Add |
|-------|---------|-------------|
| pulldown-cmark | Markdown parsing | Phase 5 |
| arboard | Clipboard access | Phase 4.2 |
| dirs | Standard directory paths | Phase 1 |
| walkdir | Directory traversal | Phase 2.1 |
| chrono | Timestamps | Phase 2.2 |
| uuid | Document identifiers | Phase 1 |
| thiserror | Error type definitions | Phase 1 |
| anyhow | Error handling | Phase 1 |
| log + env_logger | Logging | Phase 1 |

**Optional/Future Dependencies**:
| Crate | Purpose | Feature Flag |
|-------|---------|--------------|
| enchant | Spell checking | `spell-check` |
| printpdf or headless_chrome | PDF export | `pdf-export` |
| syntect | Code syntax highlighting | `code-highlight` |

#### 1.1.2 Module Organization

The codebase follows a layered architecture with clear separation of concerns. Refer to `structure.md` for the complete directory layout.

**Layer Responsibilities**:

| Layer | Directory | Responsibility |
|-------|-----------|----------------|
| Entry | `src/main.rs` | Application bootstrap, CLI argument parsing |
| Application | `src/app.rs` | Cosmic Application trait implementation, message routing |
| State | `src/state/` | All application state management, data models |
| UI | `src/ui/` | Widget composition, view rendering, user interaction |
| Editor Core | `src/editor/` | Text manipulation, cursor, selection, undo/redo |
| File Handling | `src/file_handler/` | I/O, watching, backup, recovery |
| Markdown | `src/markdown/` | Parsing, tokenizing, rendering, export |
| Search | `src/search/` | In-file and global search algorithms |
| Utilities | `src/utils/` | Shared helpers (debouncing, paths, platform) |

**Module Dependencies Flow**:
```
main.rs → app.rs → state/ + ui/
                      ↓
              editor/ + file_handler/ + markdown/ + search/
                      ↓
                   utils/
```

#### 1.1.3 Error Handling Strategy

**Error Categories**:
1. **File I/O Errors**: Read failures, write failures, permission denied, file not found
2. **Encoding Errors**: Non-UTF-8 files, BOM handling issues
3. **Configuration Errors**: Corrupt settings, parse failures
4. **Clipboard Errors**: Access denied, empty clipboard
5. **Editor Errors**: Invalid cursor position, selection range errors
6. **Watcher Errors**: Failed to watch directory, too many watches

**Error Handling Principles**:
- Use `thiserror` for defining error types in `src/error.rs`
- Wrap all I/O operations with path context for meaningful messages
- Never panic in production code; propagate errors to UI layer
- Display user-friendly messages in status bar or dialogs
- Log detailed error information for debugging

**User-Facing Error Messages**:
- Keep messages actionable: "Could not save file. Check disk space and permissions."
- Avoid technical jargon in dialogs
- Offer recovery options where possible (retry, save elsewhere)

#### 1.1.4 Application State Architecture

**State Organization** (in `src/state/`):

| File | Contents |
|------|----------|
| `app_state.rs` | Root state container, document map, active document tracking |
| `editor_state.rs` | Per-document: cursor position, selection, scroll offset, undo stack |
| `tab_state.rs` | Tab ordering, tab metadata, drag state |
| `sidebar_state.rs` | File tree entries, expanded folders, filter text, selection |
| `session_state.rs` | Serializable session data for persistence |

**Key State Entities**:

1. **DocumentId**: UUID-based unique identifier for each open document
2. **Document**: Contains buffer, path, modification state, editor state
3. **AppState**: Central state with HashMap of documents, UI state, dialogs

**State Update Principles**:
- All state changes flow through the message/update cycle
- State is immutable during view rendering
- Use interior mutability sparingly and only where required
- Keep derived state minimal; compute in view when possible

### 1.2 libCosmic Integration
- **Objective**: Initialize libCosmic runtime and basic window setup
- **Tasks**:
  - Create main application struct
  - Initialize Cosmic application with theme support
  - Set up window event loop
  - Implement basic message routing
- **Success Criteria**: Application window opens with blank interface

#### 1.2.1 Application Struct Design

**CosmicNotebook struct fields**:
- `core: Core` - libCosmic core reference (required by trait)
- `state: AppState` - Complete application state
- `config: Config` - User configuration
- `file_watcher: Option<FileWatcher>` - File system watcher instance
- `autosave_pending: bool` - Flag for autosave scheduling

**Cosmic Application Trait Implementation**:
- `APP_ID`: "com.cosmic.Notebook" (follows reverse-DNS convention)
- `init()`: Load config, parse CLI args, open initial files, start watcher
- `update()`: Central message router with match on all Message variants
- `view()`: Delegate to MainWindow::view() for rendering
- `subscription()`: Keyboard events, window events, file watcher events, timers

#### 1.2.2 Message System Design

**Message Categories**:

| Category | Examples | Handler Location |
|----------|----------|------------------|
| File Operations | NewFile, OpenFile, SaveFile, CloseTab | `app.rs` file handlers |
| Tab Operations | SelectTab, NextTab, ReorderTab | `app.rs` tab handlers |
| Editor Actions | TextChanged, CursorMoved, Undo, Redo | `app.rs` editor handlers |
| Clipboard | Cut, Copy, Paste, ClipboardResult | `app.rs` clipboard handlers |
| Search | FindNext, ReplaceAll, GlobalSearch | `app.rs` search handlers |
| View | ToggleViewMode, ToggleSidebar | `app.rs` view handlers |
| Dialogs | ToggleCommandPalette, DialogConfirm | `app.rs` dialog handlers |
| System | WindowResized, QuitRequested, Error | `app.rs` system handlers |

**Message Flow**:
1. User interaction or system event triggers Message
2. `update()` receives Message, pattern matches to handler
3. Handler modifies state and returns Command (side effects)
4. Commands execute asynchronously, may produce new Messages
5. `view()` re-renders based on updated state

#### 1.2.3 Window Configuration

**Initial Window Settings**:
- Default size: 1200x800 pixels
- Minimum size: 400x300 pixels
- Title format: "{filename} {modified_indicator} - Cosmic Notebook"
- Modified indicator: "•" (bullet) for unsaved changes

**Window Event Handling**:
- CloseRequested: Check for unsaved changes, show confirmation if needed
- Focused: Trigger file change detection, resume autosave timer
- Unfocused: Trigger autosave if changes pending
- Resized: Store dimensions for session restoration

---

## Phase 2: File Management System

### 2.1 File Browser & Sidebar
- **Objective**: Implement sidebar for quick file navigation
- **Tasks**:
  - Create file tree data structure
  - Implement recursive directory scanning
  - Build sidebar UI component with scrollable file list
  - Add file filtering (show only .md files by default)
  - Implement click handlers for file selection
  - Add folder expand/collapse functionality
- **Success Criteria**: Sidebar displays folder structure; clicking files updates selection state

#### 2.1.1 File Tree Data Model

**FileEntry Structure**:
- `path`: Full filesystem path
- `name`: Display name (filename only)
- `is_directory`: Boolean flag
- `depth`: Nesting level from root (0-based)
- `parent_index`: Index of parent in flat list (for efficient traversal)
- `modified_time`: Last modification timestamp (for sorting/display)
- `size_bytes`: File size (for display)

**SidebarState Structure**:
- `root`: Current working directory path
- `entries`: Flat vector of FileEntry (efficient for rendering)
- `expanded_folders`: HashSet of expanded directory paths
- `selected_path`: Currently highlighted file
- `filter_text`: Search/filter input
- `filtered_indices`: Indices matching current filter
- `visible`: Sidebar visibility toggle
- `width`: Sidebar width in pixels (resizable)
- `is_scanning`: Loading indicator flag

#### 2.1.2 Directory Scanning Configuration

**Default Scan Settings**:
| Setting | Default Value | Purpose |
|---------|---------------|---------|
| max_depth | 10 levels | Prevent infinite recursion |
| include_extensions | ["md", "markdown"] | File type filtering |
| show_hidden | false | Hide dotfiles by default |
| max_entries | 10,000 | Memory protection |

**Ignored Directories** (never scanned):
- `.git` - Version control
- `node_modules` - Node.js dependencies
- `target` - Rust build output
- `__pycache__` - Python cache
- `.venv`, `venv` - Python virtual environments
- `build`, `dist` - Common build outputs

#### 2.1.3 Sidebar UI Specifications

**Visual Layout**:
- Search/filter input at top with placeholder "Search files..."
- Scrollable file list below search
- Indentation: 16px per depth level
- Row height: 28px
- Icons: Folder (open/closed), File (markdown icon)

**Interaction Behaviors**:
| Action | Behavior |
|--------|----------|
| Single-click file | Open in new tab (or switch to existing tab) |
| Single-click folder | Toggle expand/collapse |
| Double-click file | Open and focus editor |
| Right-click | Context menu (future: rename, delete, reveal in file manager) |
| Drag file | Reorder tabs (if dragged to tab bar) |

**Keyboard Navigation**:
- Arrow Up/Down: Move selection
- Enter: Open selected file / toggle folder
- Left: Collapse folder or move to parent
- Right: Expand folder or move to first child
- Home/End: Jump to first/last item
- Type characters: Quick filter/search

#### 2.1.4 Performance Considerations

- Scan directories asynchronously to avoid blocking UI
- Use virtualized scrolling for large file lists (render only visible items)
- Debounce filter input (150ms) to avoid excessive re-filtering
- Cache directory scans; invalidate on file watcher events
- Lazy-load deep directories (expand triggers scan of children)

### 2.2 File I/O Operations
- **Objective**: Handle reading and writing Markdown files
- **Tasks**:
  - Implement file read operation with error handling
  - Implement file write operation with atomic writes (safety)
  - Create backup system before file modifications
  - Implement file existence validation
  - Add file encoding detection and UTF-8 handling
- **Success Criteria**: Files can be read from disk and changes persisted safely

#### 2.2.1 File Reading Specifications

**Read Operation Flow**:
1. Validate file exists and is readable
2. Check file size against maximum (10MB default)
3. Read raw bytes into memory
4. Attempt UTF-8 decoding
5. On failure, try encoding detection (UTF-8 BOM, UTF-16 LE/BE)
6. Return content with detected encoding metadata

**Size Limits**:
| Limit | Value | Rationale |
|-------|-------|-----------|
| MAX_FILE_SIZE | 10 MB | Performance and memory constraints |
| WARNING_SIZE | 1 MB | Show warning before opening |

**Encoding Support Priority**:
1. UTF-8 (no BOM) - Most common
2. UTF-8 with BOM - Windows compatibility
3. UTF-16 LE with BOM - Legacy Windows files
4. UTF-16 BE with BOM - Rare but supported
5. Lossy UTF-8 - Last resort, replaces invalid bytes

#### 2.2.2 Atomic Write Implementation

**Why Atomic Writes Matter**:
- Prevents data loss if write is interrupted (crash, power loss)
- Ensures file is either fully written or unchanged
- Protects against partial writes corrupting documents

**Atomic Write Process**:
1. Generate temporary filename: `.{original_name}.{timestamp}.tmp`
2. Write content to temporary file in same directory
3. Flush buffers and sync to disk (`fsync`)
4. Atomically rename temp file to target (single syscall)
5. On any failure, delete temp file and report error

**Write Error Recovery**:
- If temp file creation fails: Report disk full or permission error
- If write fails: Delete temp, report error, original file unchanged
- If rename fails: Keep temp file, notify user to manually recover

#### 2.2.3 Backup System Design

**Recovery Directory Location**:
`~/.local/share/cosmic-notebook/recovery/`

**Recovery File Naming**:
`{document_uuid}.md.recovery`

**Recovery Manifest** (`manifest.json`):
- `version`: Schema version for future compatibility
- `files`: Map of UUID to recovery metadata
  - `document_id`: UUID string
  - `original_path`: Where file should be saved (null for new files)
  - `created_at`: When recovery file was first created
  - `last_modified`: Most recent autosave timestamp
  - `content_hash`: Simple hash for change detection

**Recovery Flow on Startup**:
1. Check recovery directory for manifest
2. For each recovery entry, check if original file exists and matches hash
3. If mismatched or original missing, offer recovery dialog
4. User chooses: Recover (open recovered version) or Discard (delete recovery file)

#### 2.2.4 File Metadata Tracking

**FileInfo Structure**:
- `path`: Canonical file path
- `size_bytes`: File size
- `modified`: System modification timestamp
- `created`: Creation timestamp (if available)
- `is_readonly`: Permission check result

**Metadata Uses**:
- Display in status bar: "Last saved: 2 minutes ago"
- Detect external modifications by comparing timestamps
- Warn before editing read-only files

### 2.3 Dynamic File Watching System
- **Objective**: Detect external file changes and update UI
- **Tasks**:
  - Integrate file system watcher (notify crate)
  - Detect file modifications, additions, and deletions
  - Implement refresh mechanism for sidebar
  - Handle conflicts when file edited externally while open
  - Implement conflict resolution strategy (Prompt user: Overwrite/Reload/Diff)
  - Create notification system for user awareness
- **Success Criteria**: External file changes trigger sidebar updates; user is notified of conflicts and offered resolution options

#### 2.3.1 File Watcher Configuration

**Watcher Settings**:
| Setting | Value | Purpose |
|---------|-------|---------|
| Debounce interval | 500ms | Batch rapid changes |
| Recursive watching | Yes | Watch entire directory tree |
| Follow symlinks | No | Avoid loops and confusion |

**Watched Events**:
| Event Type | Action |
|------------|--------|
| File Created | Add to sidebar, notify if in current folder |
| File Modified | Check if open, trigger conflict resolution |
| File Deleted | Remove from sidebar, mark tab as "deleted externally" |
| File Renamed | Update sidebar, update tab path if open |
| Folder Created | Add to sidebar if parent expanded |
| Folder Deleted | Remove from sidebar, close affected tabs |

#### 2.3.2 Conflict Resolution Strategy

**Conflict Scenarios**:

| Scenario | User Has Changes | External Change | Resolution Options |
|----------|------------------|-----------------|-------------------|
| A | No | Modified | Auto-reload silently |
| B | Yes | Modified | Show conflict dialog |
| C | No | Deleted | Show "file deleted" notification |
| D | Yes | Deleted | Show "file deleted, keep editing?" dialog |

**Conflict Dialog Options**:
1. **Reload from Disk**: Discard local changes, load external version
2. **Keep My Version**: Ignore external change, overwrite on next save
3. **Save As...**: Keep both versions, save local to new location
4. **Show Diff** (future): Side-by-side comparison view

**Conflict Dialog Content**:
- Icon: Warning symbol
- Title: "External File Change Detected"
- Message: Explain what happened, show filename
- Buttons: Clear action labels with icons

#### 2.3.3 Sidebar Refresh Behavior

**On File System Events**:
- Created file in watched directory: Insert into entries list, maintain sort order
- Deleted file: Remove from entries, update indices
- Modified file: Update metadata (size, timestamp)
- For bulk changes (>10 events): Full rescan instead of incremental updates

**Refresh Triggers**:
- File watcher event (debounced)
- Manual refresh command (Ctrl+Shift+R or button)
- Window focus gained (check for changes while app was backgrounded)
- Working directory changed

---

## Phase 3: Editor & UI Core

### 3.1 Main Viewport & Text Editor
- **Objective**: Build the primary text editing area
- **Tasks**:
  - Create scrollable text widget using libCosmic
  - Implement text rendering with proper line wrapping
  - Add line numbers display
  - Implement cursor positioning and navigation
  - Add keyboard input handling (character insertion, deletion)
  - Support arrow keys, Home/End, Page Up/Down navigation
  - Implement Distraction-Free Mode (toggle to hide sidebar/tabs)
  - Add status bar with Word/Character count
- **Success Criteria**: Users can view and edit text; cursor movement is fluid; status info visible

#### 3.1.1 Text Buffer Architecture

**Why Use Ropey**:
- O(log n) operations for insert/delete (vs O(n) for String/Vec)
- Efficient for large files (10MB+)
- Built-in line indexing
- Memory efficient through structural sharing

**TextBuffer Wrapper Responsibilities**:
- Wrap Rope with document-specific metadata
- Track version number (increments on each change)
- Maintain line ending style (LF vs CRLF)
- Provide convenient methods for common operations

**Buffer Operations**:
| Operation | Description | Complexity |
|-----------|-------------|------------|
| insert_char | Insert single character at position | O(log n) |
| insert_text | Insert string at position | O(log n + m) |
| delete_range | Delete from start to end | O(log n) |
| get_line | Get specific line content | O(log n) |
| line_count | Total number of lines | O(1) |
| char_count | Total characters | O(1) |

#### 3.1.2 Cursor Management

**Cursor Position Representation**:
- `line`: 0-based line number
- `column`: 0-based column (grapheme cluster index, not byte)
- `offset`: Absolute byte offset in document
- `preferred_column`: Remembered column for vertical movement

**Cursor Behaviors**:
| Movement | Behavior |
|----------|----------|
| Left | Move one grapheme left; wrap to previous line end |
| Right | Move one grapheme right; wrap to next line start |
| Up | Move to same column on previous line (or preferred) |
| Down | Move to same column on next line (or preferred) |
| Home | Move to first non-whitespace, then to column 0 |
| End | Move to end of line |
| Ctrl+Left | Move to start of previous word |
| Ctrl+Right | Move to start of next word |
| Page Up | Move viewport height up |
| Page Down | Move viewport height down |
| Ctrl+Home | Move to document start |
| Ctrl+End | Move to document end |

**Cursor Rendering**:
- Blink rate: 530ms on, 530ms off (standard)
- Style: Vertical bar (I-beam) in insert mode
- Color: Contrasting to background, respects theme
- Stop blinking on movement, restart after 500ms idle

#### 3.1.3 Line Numbers Gutter

**Gutter Specifications**:
- Width: Dynamic based on line count (min 3 digits, e.g., "  1")
- Padding: 8px left, 12px right
- Font: Monospace, same size as editor or slightly smaller
- Color: Muted/secondary text color
- Current line: Highlighted (bold or brighter)

**Gutter Behavior**:
- Click on line number: Select entire line
- Drag across numbers: Select multiple lines
- Gutter width updates when line count crosses 10, 100, 1000, etc.

#### 3.1.4 Text Rendering

**Rendering Requirements**:
- Monospace font by default (configurable)
- Support variable-width fonts (future)
- Proper grapheme cluster handling (emoji, combining characters)
- Syntax highlighting integration (Phase 5)
- Selection highlighting with distinct background color

**Line Wrapping Modes**:
| Mode | Behavior |
|------|----------|
| No wrap | Horizontal scroll for long lines |
| Word wrap | Break at word boundaries |
| Character wrap | Break at any character (emergency fallback) |

**Scroll Behavior**:
- Smooth scrolling with animation (optional, configurable)
- Scroll margin: Keep cursor 3 lines from edge when possible
- Horizontal scroll: Only in no-wrap mode
- Mouse wheel: Scroll 3 lines per notch

#### 3.1.5 Status Bar Design

**Status Bar Sections** (left to right):
1. **File info**: Line X, Column Y | Encoding (UTF-8) | Line ending (LF/CRLF)
2. **Statistics**: Words: N | Characters: M
3. **Status message**: Temporary messages ("Saved", "Searching...", errors)
4. **Mode indicator**: "Markdown" | View mode (Edit/Preview/Split)

**Status Bar Specifications**:
- Height: 24-28px
- Background: Slightly different from editor (theme-aware)
- Message timeout: 5 seconds for transient messages
- Click actions: Click on encoding to change, click on line ending to change

#### 3.1.6 Distraction-Free Mode

**What Gets Hidden**:
- Sidebar
- Tab bar
- Status bar (optional, user preference)
- Line numbers (optional)

**What Remains**:
- Editor area (centered, with max-width)
- Minimal title bar or none (system dependent)

**Distraction-Free Layout**:
- Max editor width: 80 characters or 800px (configurable)
- Centered horizontally
- Increased line height: 1.6x normal
- Optional background dimming on sides

**Exit Distraction-Free**:
- Press Escape
- Press Ctrl+Shift+D (toggle shortcut)
- Move mouse to edge to peek UI (optional)

### 3.2 Tab System
- **Objective**: Implement multiple file tabs like VS Code
- **Tasks**:
  - Create tab management data structure
  - Build tab bar UI component
  - Implement tab switching logic
  - Add tab closing functionality
  - Implement active tab highlighting
  - Store tab open/close state
- **Success Criteria**: Multiple files can be open simultaneously with easy switching

#### 3.2.1 Tab State Management

**TabState Structure**:
- `tabs`: Ordered vector of DocumentId
- `active_index`: Currently selected tab index
- `drag_state`: For tab reordering (source index, current position)
- `scroll_offset`: For scrollable tab bar when many tabs

**Tab Operations**:
| Operation | Behavior |
|-----------|----------|
| Open file | Add tab if not open, else switch to existing |
| Close tab | Prompt if unsaved, remove from list, switch to adjacent |
| Close all | Prompt for each unsaved, clear list |
| Reorder | Drag and drop within tab bar |
| Switch | Click or Ctrl+Tab/Ctrl+Shift+Tab |

#### 3.2.2 Tab Bar UI Specifications

**Tab Item Layout**:
- Icon: File type indicator (markdown icon)
- Name: Filename (no path)
- Modified indicator: Dot before close button when unsaved
- Close button: X icon, appears on hover or for active tab

**Tab Sizing**:
- Min width: 100px
- Max width: 200px
- Flexible: Shrink equally when space constrained
- Overflow: Scroll arrows or dropdown menu

**Tab Visual States**:
| State | Appearance |
|-------|------------|
| Active | Highlighted background, connected to editor |
| Inactive | Normal background, subtle border |
| Hover | Slight highlight |
| Unsaved | Dot indicator (●) |
| Dragging | Elevated/shadow effect |

#### 3.2.3 Tab Interactions

**Mouse Interactions**:
- Click: Switch to tab
- Middle-click: Close tab (common convention)
- Click close button: Close tab
- Drag: Reorder tabs
- Double-click empty area: New file (optional)
- Right-click: Context menu (Close, Close Others, Close All, Copy Path)

**Keyboard Navigation**:
- Ctrl+Tab: Next tab
- Ctrl+Shift+Tab: Previous tab
- Ctrl+W: Close current tab
- Ctrl+1-9: Switch to tab by position
- Ctrl+Shift+T: Reopen last closed tab (future)

### 3.3 File Status Indicator
- **Objective**: Visual indication of file save state
- **Tasks**:
  - Add dot/indicator next to unsaved file names in tabs
  - Implement save state tracking in application state
  - Add visual distinction between saved and unsaved states
  - Display file modification time
- **Success Criteria**: Users can immediately see which files have unsaved changes

#### 3.3.1 Modification Tracking

**Document Modification State**:
- `is_modified`: Boolean, true when buffer differs from disk
- `last_saved_version`: Buffer version number at last save
- `disk_modified_time`: Timestamp of file on disk at last read/write

**When is_modified Becomes True**:
- Any text insertion
- Any text deletion
- Paste operation
- Undo/redo that changes content

**When is_modified Becomes False**:
- Successful save operation
- Reload from disk (discard changes)
- Undo back to saved state (version matches last_saved_version)

#### 3.3.2 Visual Indicators

**Tab Indicators**:
- Unsaved: Filled dot (●) or asterisk (*) before close button
- Externally modified: Different icon or color (⚠)
- Read-only: Lock icon

**Window Title Format**:
- Saved: `filename.md - Cosmic Notebook`
- Unsaved: `filename.md • - Cosmic Notebook`
- New file: `Untitled - Cosmic Notebook`
- New unsaved: `Untitled • - Cosmic Notebook`

**Color Coding** (optional, theme-dependent):
- Unsaved tab: Subtle accent color
- Error state: Red tint

---

## Phase 4: Editing Features

### 4.1 Undo/Redo System
- **Objective**: Implement robust undo/redo with copy/paste support
- **Tasks**:
  - Create operation history data structure (Command pattern)
  - Implement text insertion/deletion as reversible operations
  - Build undo/redo stack management
  - Add keyboard shortcuts (Ctrl+Z for undo, Ctrl+Shift+Z for redo)
  - Integrate with copy/paste operations
  - Implement history clearing on file switch
  - Add memory optimization for large undo stacks
- **Success Criteria**: Undo/redo works correctly for all text operations; history persists during session

#### 4.1.1 Command Pattern Implementation

**EditOperation Types**:
| Operation | Data Stored | Reverse Operation |
|-----------|-------------|-------------------|
| Insert | Position, inserted text | Delete same range |
| Delete | Position, deleted text | Insert deleted text at position |
| Replace | Position, old text, new text | Replace with old text |

**Operation Grouping**:
- Consecutive character inserts within 500ms: Group into single operation
- Consecutive deletes (backspace/delete): Group into single operation
- Paste always creates new group
- Cut creates delete operation
- Any cursor movement breaks grouping

**UndoStack Structure**:
- `undo_stack`: Vec of EditOperation groups
- `redo_stack`: Vec of EditOperation groups (cleared on new edit)
- `max_depth`: Maximum operations to keep (default 1000)
- `current_group`: Operations being accumulated before commit

#### 4.1.2 Memory Optimization

**Memory Constraints**:
- Target: <10MB for undo history on typical usage
- Each operation stores: ~32 bytes metadata + text content
- 1000 operations with average 100 chars = ~3.2MB

**Optimization Strategies**:
1. **Merge consecutive operations**: Reduce count
2. **Limit max depth**: Drop oldest operations when exceeded
3. **Compress large text**: For deletions >1KB, consider compression
4. **Clear on save** (optional): Some editors clear history on save

**Undo History Per Document**:
- Each document has independent undo/redo stacks
- Switching tabs preserves undo history
- Closing tab clears undo history (not recoverable)

#### 4.1.3 Edge Cases

**Handling Complex Scenarios**:
| Scenario | Behavior |
|----------|----------|
| Undo to saved state | Clear modified flag |
| Undo past file open | Stop at file open state |
| Redo after new edit | Clear redo stack |
| Large paste (>1MB) | Single undo operation, may be slow |
| Reload from disk | Clear undo/redo (can't undo external changes) |

### 4.2 Copy/Paste System
- **Objective**: Implement text clipboard operations
- **Tasks**:
  - Implement text selection (mouse drag or Shift+arrow keys)
  - Add visual selection highlighting
  - Implement copy (Ctrl+C) - write to system clipboard
  - Implement paste (Ctrl+V) - read from system clipboard
  - Implement cut (Ctrl+X) - combine copy and delete
  - Track paste operations in undo/redo history
  - Handle clipboard access errors gracefully
- **Success Criteria**: Copy/paste works with system clipboard; operations are undoable

#### 4.2.1 Selection Model

**Selection Representation**:
- `anchor`: Position where selection started
- `cursor`: Current cursor position (selection end)
- Selection range: min(anchor, cursor) to max(anchor, cursor)
- Direction matters for Shift+Arrow extending

**Selection Types**:
| Type | How Created | Behavior |
|------|-------------|----------|
| None | Click without drag | No selection, just cursor |
| Character | Shift+Arrow, drag | Select characters |
| Word | Double-click, Ctrl+Shift+Arrow | Select whole words |
| Line | Triple-click, line number click | Select whole lines |
| All | Ctrl+A | Select entire document |

**Selection Visual**:
- Background color: Theme-defined selection color
- Selected text: May have different foreground (theme-dependent)
- Selection across lines: Full-width highlight per line

#### 4.2.2 Clipboard Integration

**Using arboard Crate**:
- Cross-platform clipboard access
- Text-only support initially
- Future: HTML/RTF for rich paste

**Clipboard Operations**:
| Operation | Shortcut | Behavior |
|-----------|----------|----------|
| Copy | Ctrl+C | Copy selection to clipboard; no-op if no selection |
| Cut | Ctrl+X | Copy selection, then delete; no-op if no selection |
| Paste | Ctrl+V | Insert clipboard at cursor, replacing selection |
| Paste without formatting | Ctrl+Shift+V | Same as Paste for plain text |

**Error Handling**:
- Clipboard access denied: Show non-intrusive error in status bar
- Empty clipboard on paste: Silent no-op
- Clipboard contains non-text: Silent no-op or notification

#### 4.2.3 Selection Keyboard Shortcuts

**Selection Modifiers**:
| Base Movement | + Shift | Result |
|---------------|---------|--------|
| Arrow keys | Shift+Arrow | Extend selection by character |
| Ctrl+Arrow | Ctrl+Shift+Arrow | Extend selection by word |
| Home/End | Shift+Home/End | Extend to line start/end |
| Ctrl+Home/End | Ctrl+Shift+Home/End | Extend to document start/end |
| Page Up/Down | Shift+Page Up/Down | Extend by page |

**Selection Commands**:
- Ctrl+A: Select all
- Ctrl+D: Select word at cursor (future, VS Code-like)
- Ctrl+L: Select line (future)
- Escape: Clear selection

### 4.3 Find & Replace (Optional Enhancement)
- **Objective**: Quick search functionality
- **Tasks**:
  - Create find dialog UI
  - Implement text search algorithm
  - Add highlighting of search results
  - Implement replace single/replace all
  - Add keyboard shortcut (Ctrl+H)
  - Implement Global Search (search text across all files in folder)
- **Success Criteria**: Users can search and replace text efficiently; find text across project

#### 4.3.1 Find Dialog UI Design

**Find Bar Layout** (appears below tab bar):
- Find input field with placeholder "Find"
- Match count indicator: "3 of 15"
- Navigation buttons: Previous (↑), Next (↓)
- Options: Case sensitive (Aa), Whole word (W), Regex (.*)
- Close button (X)

**Find & Replace Bar** (expanded):
- Find input (same as above)
- Replace input field with placeholder "Replace"
- Replace button: Replace current match
- Replace All button: Replace all matches
- Preserve case option (future)

**Keyboard Shortcuts**:
| Action | Shortcut |
|--------|----------|
| Open Find | Ctrl+F |
| Open Find & Replace | Ctrl+H |
| Find Next | F3 or Enter in find field |
| Find Previous | Shift+F3 or Shift+Enter |
| Replace | Ctrl+Shift+1 (when replace open) |
| Replace All | Ctrl+Shift+Enter |
| Close Find | Escape |

#### 4.3.2 Search Algorithm

**Search Modes**:
| Mode | Description |
|------|-------------|
| Plain text | Literal string matching |
| Case insensitive | Ignore case differences |
| Whole word | Match only complete words (word boundaries) |
| Regex | Full regular expression support |

**Search Implementation**:
- Use Rust's regex crate for regex mode
- For plain text, use simple string search (faster)
- Search incrementally as user types (debounced 150ms)
- Highlight all matches in document
- Current match: Distinct highlight color

**Performance**:
- For documents <100KB: Search entire document
- For larger documents: Search visible area first, then background search rest
- Cache regex compilation
- Cancel pending search on new query

#### 4.3.3 Global Search

**Global Search Panel**:
- Full-height panel replacing sidebar or as overlay
- Search input with same options as local find
- File pattern filter: "*.md" by default
- Results grouped by file
- Click result to open file and jump to match

**Global Search Results Display**:
- File path (relative to workspace)
- Line number and match preview
- Context: Show line with match highlighted
- Limit: 1000 results max, show "and N more..."

**Global Search Performance**:
- Search in background thread
- Stream results as found
- Cancel on new search query
- Skip binary files and very large files

### 4.4 Advanced Editing Features
- **Objective**: Enhance editing capabilities
- **Tasks**:
  - Implement Auto-Save (configurable interval/on focus lost)
  - Integrate lightweight Spell Checker
  - Add Frontmatter support (highlight YAML/TOML headers)
- **Success Criteria**: Editor supports auto-save, spell check, and frontmatter

#### 4.4.1 Auto-Save Implementation

**Auto-Save Triggers**:
| Trigger | Behavior |
|---------|----------|
| Timer | Save every N seconds if modified (default 60s) |
| Focus lost | Save when window loses focus |
| Tab switch | Optionally save when switching tabs |
| Idle | Save after N seconds of no activity |

**Auto-Save Locations**:
- **Saved files**: Write to recovery file, not original (safer)
- **New/untitled files**: Write to recovery directory only

**Auto-Save Settings**:
- Enable/disable toggle
- Interval in seconds (30-300, default 60)
- Save on focus lost toggle
- Save to original vs recovery file

**Auto-Save Status**:
- Show "Auto-saved" briefly in status bar
- Don't clear modified indicator (recovery file ≠ saved)

#### 4.4.2 Spell Checker Integration

**Spell Check Strategy**:
- Use system spell checker via enchant/hunspell wrapper
- Check as you type with debouncing
- Underline misspelled words with wavy red line

**Spell Check Scope**:
- Check prose text only
- Skip code blocks (fenced and indented)
- Skip inline code
- Skip URLs and file paths
- Respect frontmatter boundaries

**Spell Check UI**:
- Red wavy underline on misspelled words
- Right-click for suggestions
- "Add to dictionary" option
- "Ignore" option (session only)
- "Ignore all" (all instances in session)

**Dictionary Management**:
- Use system dictionaries
- User dictionary in config directory
- Per-document ignored words (stored in frontmatter or .spelling file)

#### 4.4.3 Frontmatter Support

**Frontmatter Detection**:
- YAML: Document starts with `---` on first line
- TOML: Document starts with `+++` on first line
- Must end with matching delimiter

**Frontmatter Handling**:
| Feature | Behavior |
|---------|----------|
| Syntax highlighting | Different color scheme for metadata |
| Spell check | Disabled within frontmatter |
| Word count | Exclude frontmatter from statistics |
| Folding (future) | Collapse frontmatter block |

**Common Frontmatter Fields**:
- title, date, author, tags, categories
- Provide subtle completion hints (future)

---

## Phase 5: Markdown Specific Features

### 5.1 Syntax Highlighting
- **Objective**: Visual differentiation of Markdown elements
- **Tasks**:
  - Tokenize Markdown syntax (headers, bold, italic, code blocks, links)
  - Support GitHub Flavored Markdown (tables, task lists, strikethrough)
  - Implement color scheme for different syntax elements
  - Apply highlighting during text rendering
  - Optimize performance for large files
  - Support light/dark theme variations
- **Success Criteria**: Markdown elements (including GFM) are color-coded and easy to distinguish

#### 5.1.1 Markdown Token Types

**Standard Markdown Tokens**:
| Token | Pattern | Example |
|-------|---------|---------|
| H1-H6 | `#` to `######` | `# Heading` |
| Bold | `**text**` or `__text__` | `**bold**` |
| Italic | `*text*` or `_text_` | `*italic*` |
| Bold+Italic | `***text***` | `***both***` |
| Code inline | `` `code` `` | `` `code` `` |
| Code block | ` ``` ` fenced | Fenced code |
| Blockquote | `>` prefix | `> quote` |
| List item | `- ` or `* ` or `1. ` | `- item` |
| Link | `[text](url)` | `[link](url)` |
| Image | `![alt](url)` | `![img](url)` |
| Horizontal rule | `---` or `***` | `---` |

**GitHub Flavored Markdown Extensions**:
| Token | Pattern | Example |
|-------|---------|---------|
| Strikethrough | `~~text~~` | `~~struck~~` |
| Task list | `- [ ]` or `- [x]` | `- [x] done` |
| Table | Pipe-delimited | `| A | B |` |
| Autolink | URL or email | `https://...` |
| Footnote | `[^id]` | `[^1]` |

#### 5.1.2 Syntax Highlighting Color Scheme

**Color Categories** (theme-aware):
| Category | Light Theme | Dark Theme | Elements |
|----------|-------------|------------|----------|
| Heading | Dark blue | Light blue | H1-H6 text |
| Emphasis | Italic style | Italic style | *text* |
| Strong | Bold + darker | Bold + brighter | **text** |
| Code | Gray background | Gray background | Inline code |
| Link | Blue, underlined | Cyan, underlined | Link text |
| URL | Muted gray | Muted gray | Link URLs |
| List marker | Accent color | Accent color | -, *, 1. |
| Quote | Muted + left border | Muted + left border | Blockquotes |
| Metadata | Purple/magenta | Purple/magenta | Frontmatter |

**Syntax Colors Configuration**:
- Store in `assets/themes/syntax-light.toml` and `syntax-dark.toml`
- Allow user overrides in config
- Support importing custom themes (future)

#### 5.1.3 Tokenizer Implementation

**Tokenizer Strategy**:
- Line-based tokenization for efficiency
- Cache tokens per line, invalidate on line change
- Re-tokenize visible lines first on scroll

**Token Structure**:
- `token_type`: Enum of token types
- `start`: Byte offset in line
- `end`: Byte offset in line (exclusive)
- `style`: Optional nested style (e.g., bold inside heading)

**Performance Targets**:
- Tokenize 1000-line document: <50ms
- Re-tokenize single line: <1ms
- Memory per line: ~100 bytes average

#### 5.1.4 Code Block Language Detection

**Fenced Code Block Handling**:
- Detect language from info string: ` ```rust `
- Apply language-specific highlighting (future: use syntect)
- Fallback to plain monospace if language unknown

**Supported Languages** (Phase 1):
- No syntax highlighting inside blocks initially
- Just monospace font + background color

**Supported Languages** (Future with syntect):
- Common: rust, python, javascript, typescript, json, yaml, toml
- Markup: html, css, xml, markdown
- Shell: bash, sh, zsh
- Others: sql, c, cpp, java, go, ruby

### 5.2 Preview Mode & Image Handling
- **Objective**: Display Markdown rendered output and handle media
- **Tasks**:
  - Integrate Markdown parser (comrak or pulldown-cmark)
  - Convert Markdown to renderable format
  - Create toggle between edit and preview modes
  - Implement Split View (Side-by-side Editor and Preview)
  - Handle code block rendering with language detection
  - Implement Image Drag & Drop (generate markdown link)
  - Implement Paste Image from Clipboard (save to assets/ and link)
  - Implement Export options (HTML, PDF)
- **Success Criteria**: Users can toggle to see rendered Markdown output; images handled seamlessly

#### 5.2.1 Markdown Parser Selection

**pulldown-cmark vs comrak**:
| Feature | pulldown-cmark | comrak |
|---------|----------------|--------|
| Speed | Faster | Slightly slower |
| GFM support | Basic | Full |
| Safety | No raw HTML by default | Configurable |
| Size | Smaller | Larger |
| Tables | Yes | Yes |
| Footnotes | No | Yes |

**Recommendation**: Start with pulldown-cmark for speed; switch to comrak if footnotes or advanced GFM needed.

#### 5.2.2 Preview Rendering Architecture

**Rendering Pipeline**:
1. Parse Markdown to AST (Abstract Syntax Tree)
2. Convert AST to cosmic/iced widget tree
3. Render widgets in preview pane

**Widget Mapping**:
| Markdown Element | Widget |
|------------------|--------|
| Paragraph | Text container |
| Heading | Text with size/weight |
| List | Vertical column with bullets |
| Code block | Container with monospace text + background |
| Blockquote | Container with left border |
| Image | Image widget (async load) |
| Link | Styled text with click handler |
| Table | Grid layout |
| Horizontal rule | Divider line |

#### 5.2.3 View Modes

**Available Modes**:
| Mode | Layout | Use Case |
|------|--------|----------|
| Edit | Full-width editor | Writing/editing |
| Preview | Full-width rendered | Reading/reviewing |
| Split | 50/50 side-by-side | Writing with live preview |

**Split View Details**:
- Divider: Draggable to adjust ratio
- Scroll sync: Preview follows editor scroll position
- Default ratio: 50/50
- Min pane width: 300px

**Mode Switching**:
- Ctrl+Shift+V: Toggle between Edit and Preview
- Ctrl+Shift+B: Toggle Split view
- Commands in command palette
- Toolbar buttons (if toolbar enabled)

#### 5.2.4 Image Handling

**Image Display in Preview**:
- Load images asynchronously
- Support local paths (relative to document)
- Support URLs (fetch with timeout)
- Show placeholder while loading
- Show error placeholder if failed

**Image Drag & Drop**:
1. User drags image file onto editor
2. Prompt: Copy to assets folder or link original location?
3. If copy: Create `assets/` folder if needed, copy image
4. Insert Markdown: `![filename](assets/filename.png)`
5. Position cursor after inserted text

**Image Paste from Clipboard**:
1. User pastes (Ctrl+V) with image in clipboard
2. Generate filename: `image-{timestamp}.png`
3. Save to `assets/` folder (create if needed)
4. Insert Markdown: `![image](assets/image-{timestamp}.png)`

**Assets Folder Convention**:
- Default: `assets/` in same directory as markdown file
- Configurable: User can set different folder name
- Relative paths only for portability

#### 5.2.5 Export Options

**HTML Export**:
- Self-contained HTML with embedded styles
- Option to embed images as base64
- Include syntax highlighting CSS
- Template customization (future)

**PDF Export** (requires feature flag):
- Use headless browser (Chrome/Chromium) or printpdf
- Render Markdown to HTML first
- Convert HTML to PDF
- Options: Page size, margins, header/footer

**Export Dialog**:
- Output format selection
- Output path selection
- Options specific to format
- Preview of first page (future)

---

## Phase 6: Performance & Polish

### 6.1 Performance Optimization
- **Objective**: Ensure minimal resource usage
- **Tasks**:
  - Implement lazy loading for large files
  - Optimize text rendering performance
  - Add debouncing for file watching events
  - Profile memory usage
  - Implement efficient scroll handling
  - Cache rendered content
- **Success Criteria**: Application runs smoothly with minimal CPU/memory usage; maintains small binary and runtime footprint

#### 6.1.1 Performance Budgets

**Startup Performance**:
| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold start | <500ms | Time to interactive window |
| Warm start | <200ms | With cached config |
| Open file | <100ms | For files <1MB |
| Open large file | <500ms | For files 1-10MB |

**Runtime Performance**:
| Metric | Target | Measurement |
|--------|--------|-------------|
| Input latency | <50ms | Key press to render |
| Scroll FPS | 60fps | Smooth scrolling |
| Search latency | <100ms | Find results for <1MB file |
| Save latency | <200ms | Write to disk |

**Memory Usage**:
| Scenario | Target |
|----------|--------|
| Empty (no files) | <30MB |
| Single small file | <50MB |
| 10 files open | <80MB |
| Large file (10MB) | <150MB |

#### 6.1.2 Lazy Loading Strategy

**Virtual Scrolling for Large Files**:
- Only render visible lines + buffer (50 lines above/below)
- Track viewport: start line, line count
- On scroll: Update visible range, render new lines
- Reuse line widgets where possible

**Lazy Syntax Highlighting**:
- Tokenize visible lines immediately
- Queue off-screen lines for background processing
- Priority: Current viewport > nearby > distant

**Lazy Preview Rendering**:
- Render visible section of preview
- Lazy-load images only when scrolled into view
- Cache rendered blocks, invalidate on edit

#### 6.1.3 Debouncing and Throttling

**Event Debouncing**:
| Event | Debounce Time | Reason |
|-------|---------------|--------|
| File watcher | 500ms | Batch rapid file changes |
| Search input | 150ms | Avoid search on every keystroke |
| Sidebar filter | 150ms | Smooth filtering |
| Window resize | 100ms | Avoid layout thrashing |
| Auto-save | 2000ms | Don't save on every keystroke |

**Throttling**:
| Operation | Throttle Rate | Reason |
|-----------|---------------|--------|
| Preview sync scroll | 16ms (60fps) | Smooth scrolling |
| Cursor blink | 530ms | Standard blink rate |
| Status message clear | Check every 1s | Clean up expired messages |

#### 6.1.4 Caching Strategy

**What to Cache**:
| Data | Cache Location | Invalidation |
|------|----------------|--------------|
| File tree | Memory | File watcher event |
| Syntax tokens | Per-line in buffer | Line modification |
| Preview render | Memory | Document change |
| Search results | Memory | Query or document change |
| Configuration | Memory + disk | Settings change |
| Recent files | Disk | Never (append only) |

**Cache Size Limits**:
- Preview cache: Last 100 blocks
- Search cache: Last 10 queries
- File tree cache: Unlimited (small per entry)

### 6.2 Keyboard Shortcuts
- **Objective**: Comprehensive keyboard support
- **Tasks**:
  - Implement Ctrl+S (Save)
  - Implement Ctrl+O (Open file dialog)
  - Implement Ctrl+N (New file)
  - Implement Ctrl+W (Close tab)
  - Implement Ctrl+Tab/Ctrl+Shift+Tab (Tab switching)
  - Implement Ctrl+Z/Ctrl+Shift+Z (Undo/Redo)
  - Implement Ctrl+C/X/V (Copy/Cut/Paste)
  - Implement Command Palette (Ctrl+Shift+P) for quick access to commands
  - Add help overlay showing all shortcuts
- **Success Criteria**: Most operations accessible via keyboard

#### 6.2.1 Complete Keyboard Shortcut Reference

**File Operations**:
| Action | Primary | Alternative |
|--------|---------|-------------|
| New file | Ctrl+N | |
| Open file | Ctrl+O | |
| Save | Ctrl+S | |
| Save As | Ctrl+Shift+S | |
| Save All | Ctrl+Alt+S | |
| Close tab | Ctrl+W | Ctrl+F4 |
| Close all | Ctrl+Shift+W | |
| Quit | Ctrl+Q | Alt+F4 |

**Edit Operations**:
| Action | Primary | Alternative |
|--------|---------|-------------|
| Undo | Ctrl+Z | |
| Redo | Ctrl+Shift+Z | Ctrl+Y |
| Cut | Ctrl+X | |
| Copy | Ctrl+C | |
| Paste | Ctrl+V | |
| Select All | Ctrl+A | |
| Delete line | Ctrl+Shift+K | |
| Duplicate line | Ctrl+Shift+D | |

**Navigation**:
| Action | Primary | Alternative |
|--------|---------|-------------|
| Go to line | Ctrl+G | |
| Go to start | Ctrl+Home | |
| Go to end | Ctrl+End | |
| Next tab | Ctrl+Tab | Ctrl+PageDown |
| Previous tab | Ctrl+Shift+Tab | Ctrl+PageUp |
| Tab by number | Ctrl+1-9 | |

**Search**:
| Action | Primary | Alternative |
|--------|---------|-------------|
| Find | Ctrl+F | |
| Find & Replace | Ctrl+H | |
| Find next | F3 | Enter in find field |
| Find previous | Shift+F3 | Shift+Enter |
| Global search | Ctrl+Shift+F | |

**View**:
| Action | Primary | Alternative |
|--------|---------|-------------|
| Toggle sidebar | Ctrl+B | |
| Toggle preview | Ctrl+Shift+V | |
| Toggle split view | Ctrl+Shift+B | |
| Toggle distraction-free | Ctrl+Shift+D | F11 |
| Zoom in | Ctrl++ | Ctrl+= |
| Zoom out | Ctrl+- | |
| Reset zoom | Ctrl+0 | |
| Command palette | Ctrl+Shift+P | F1 |

**Markdown Specific**:
| Action | Primary | Purpose |
|--------|---------|---------|
| Bold | Ctrl+B (with selection) | Wrap in ** |
| Italic | Ctrl+I (with selection) | Wrap in * |
| Code | Ctrl+` (with selection) | Wrap in ` |
| Link | Ctrl+K | Insert link template |
| Heading increase | Ctrl+] | Add # |
| Heading decrease | Ctrl+[ | Remove # |

#### 6.2.2 Command Palette Design

**Palette Features**:
- Fuzzy search for commands
- Show keyboard shortcut next to command
- Recently used commands at top
- Category prefixes: `>` for commands, `@` for symbols (future), `:` for go to line

**Command List** (subset):
- New File
- Open File...
- Save
- Save As...
- Close Tab
- Undo / Redo
- Find / Find and Replace
- Toggle Sidebar / Preview / Split View
- Toggle Distraction-Free Mode
- Open Settings
- Keyboard Shortcuts
- About Cosmic Notebook

**Palette UI**:
- Modal overlay, centered
- Search input at top
- Scrollable list of matching commands
- Keyboard navigation: Up/Down, Enter to execute, Escape to close

#### 6.2.3 Shortcut Help Overlay

**Overlay Design**:
- Triggered by Ctrl+? or from Help menu
- Modal overlay showing all shortcuts
- Grouped by category
- Searchable (filter as you type)
- Dismissible by Escape or clicking outside

### 6.3 User Experience Polish
- **Objective**: Refine UI/UX for better usability
- **Tasks**:
  - Implement proper error messages and dialogs
  - Add confirmation for destructive operations (close unsaved files)
  - Create smooth animations/transitions
  - Implement responsive layout resizing
  - Add tooltips for UI elements
  - Implement file drag-and-drop support
- **Success Criteria**: Application feels polished and responsive

#### 6.3.1 Dialog System

**Dialog Types**:
| Type | Use Case | Buttons |
|------|----------|---------|
| Confirmation | Destructive actions | Yes/No or specific actions |
| Information | Notifications | OK |
| Error | Error reporting | OK or Retry/Cancel |
| Input | Request user input | OK/Cancel |

**Standard Dialogs**:
1. **Unsaved Changes**: "Save changes to {filename}?" → Save / Don't Save / Cancel
2. **File Conflict**: "File changed externally" → Reload / Keep Mine / Cancel
3. **Delete Confirmation**: "Delete {filename}?" → Delete / Cancel
4. **Large File Warning**: "File is large, may be slow" → Open Anyway / Cancel

**Dialog Guidelines**:
- Clear, action-oriented button labels (not just "OK")
- Destructive action button on right, styled as warning
- Escape key closes (equivalent to Cancel)
- Enter key activates primary action

#### 6.3.2 Animations and Transitions

**Recommended Animations** (subtle, optional):
| Element | Animation | Duration |
|---------|-----------|----------|
| Tab switch | Crossfade | 100ms |
| Sidebar toggle | Slide | 150ms |
| Dialog appear | Fade in + scale | 100ms |
| Search results highlight | Pulse | 200ms |
| Status message | Fade in/out | 150ms |

**Animation Principles**:
- Keep animations under 200ms
- Provide "Reduce motion" setting to disable
- Never block user interaction during animation
- Use easing functions for natural feel

#### 6.3.3 Tooltips

**Elements with Tooltips**:
- Toolbar buttons: Action name + shortcut
- Tab close button: "Close (Ctrl+W)"
- Status bar items: Detailed explanation
- Sidebar icons: Item type

**Tooltip Behavior**:
- Delay: 500ms before showing
- Duration: Visible while hovering, fade after 5s
- Position: Above or below element, avoid edge clipping

#### 6.3.4 Drag and Drop Support

**Supported Drop Targets**:
| Target | Drop Type | Result |
|--------|-----------|--------|
| Editor | Files | Open file(s) in new tab(s) |
| Editor | Images | Insert image markdown |
| Editor | Text | Insert text at cursor |
| Tab bar | Files | Open file(s) as new tab(s) |
| Tab bar | Tab | Reorder tabs |
| Sidebar | Files | Copy/move to folder |

**Drag Visual Feedback**:
- Drag cursor: Indicates allowed action
- Drop target highlight: Visual indicator of drop zone
- Invalid target: Show "not allowed" cursor

---

## Phase 7: Configuration & Settings

### 7.1 Application Settings
- **Objective**: Allow user customization
- **Tasks**:
  - Create settings file (JSON/TOML in ~/.config/cosmic-notebook/)
  - Implement theme selection (light/dark)
  - Add font size settings
  - Add tab width configuration
  - Add word wrap toggle
  - Persist settings between sessions
- **Success Criteria**: Users can customize appearance and behavior

#### 7.1.1 Configuration File Locations

**XDG Base Directory Compliance**:
| Purpose | Path | Contents |
|---------|------|----------|
| Configuration | `~/.config/cosmic-notebook/` | `config.toml`, custom themes |
| Data | `~/.local/share/cosmic-notebook/` | Recent files, session, recovery |
| Cache | `~/.cache/cosmic-notebook/` | Temporary files, preview cache |

**Configuration Files**:
- `config.toml` - Main settings file
- `keybindings.toml` - Custom keyboard shortcuts (future)
- `snippets.toml` - User text snippets (future)

#### 7.1.2 Settings Schema

**Editor Settings**:
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| font_family | string | "monospace" | Editor font |
| font_size | u32 | 14 | Font size in points |
| line_height | f32 | 1.5 | Line height multiplier |
| tab_width | u32 | 4 | Spaces per tab |
| use_spaces | bool | true | Insert spaces for tab |
| word_wrap | bool | true | Wrap long lines |
| show_line_numbers | bool | true | Display line numbers |
| highlight_current_line | bool | true | Highlight cursor line |
| auto_indent | bool | true | Auto-indent new lines |

**Appearance Settings**:
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| theme | string | "system" | "light", "dark", or "system" |
| syntax_theme | string | "default" | Syntax highlighting theme |
| sidebar_width | u32 | 250 | Sidebar width in pixels |
| sidebar_visible | bool | true | Show sidebar on startup |

**Behavior Settings**:
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| auto_save | bool | true | Enable auto-save |
| auto_save_interval | u32 | 60 | Auto-save interval (seconds) |
| save_on_focus_lost | bool | true | Save when window loses focus |
| restore_session | bool | true | Restore previous session |
| confirm_close_unsaved | bool | true | Confirm before closing unsaved |
| check_for_updates | bool | false | Check for updates (future) |

**File Settings**:
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| default_encoding | string | "utf-8" | Default file encoding |
| default_line_ending | string | "lf" | "lf" or "crlf" |
| show_hidden_files | bool | false | Show dotfiles in sidebar |
| file_extensions | [string] | ["md", "markdown"] | File extensions to show |

**Markdown Settings**:
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| default_view_mode | string | "edit" | "edit", "preview", "split" |
| preview_sync_scroll | bool | true | Sync editor/preview scroll |
| image_assets_folder | string | "assets" | Folder for pasted images |

#### 7.1.3 Settings UI Design

**Settings Dialog Layout**:
- Sidebar with category navigation
- Main panel with settings for selected category
- Categories: Editor, Appearance, Behavior, Files, Markdown, Keyboard Shortcuts

**Settings Controls**:
| Setting Type | Control |
|--------------|---------|
| Boolean | Toggle switch |
| Number | Spinner or slider |
| String (short) | Text input |
| String (enum) | Dropdown select |
| Color | Color picker (future) |
| Font | Font picker (future) |

**Settings Behavior**:
- Apply changes immediately (live preview)
- No explicit save button (auto-save settings)
- Reset to defaults button per category
- Search/filter settings (future)

### 7.2 Recent Files & Sessions
- **Objective**: Improve workflow efficiency
- **Tasks**:
  - Track recently opened files
  - Implement "Open Recent" menu
  - Save/restore last session (open files, tabs, scroll positions)
  - Add history persistence
- **Success Criteria**: Users can quickly access recent work; session state restored

#### 7.2.1 Recent Files Management

**Recent Files Data**:
- Path: Absolute file path
- Last opened: Timestamp
- Pin status: User can pin favorites (future)

**Recent Files Storage**:
- Location: `~/.local/share/cosmic-notebook/recent.json`
- Max entries: 50 (configurable)
- Cleanup: Remove entries for non-existent files on load

**Recent Files UI**:
- Welcome screen: Show recent files when no file open
- File menu: "Open Recent" submenu
- Command palette: "Open Recent" command with fuzzy search

#### 7.2.2 Session State

**Session Data Structure**:
- `working_directory`: Last open directory
- `open_documents`: List of open file paths
- `active_document`: Index or path of active tab
- `tab_order`: Order of tabs
- `scroll_positions`: Map of path to (line, column)
- `window_state`: Size, position, maximized state
- `sidebar_state`: Width, visible, expanded folders
- `view_mode`: Last view mode per document

**Session Storage**:
- Location: `~/.local/share/cosmic-notebook/session.json`
- Save: On clean exit, periodic backup
- Load: On startup if `restore_session` enabled

**Session Restore Behavior**:
1. Load session file
2. Validate all file paths exist
3. Open existing files, skip missing (show notification)
4. Restore tab order and active tab
5. Restore scroll positions
6. Restore window geometry

---

## Phase 8: Testing & Deployment

### 8.1 Testing
- **Objective**: Ensure reliability and correctness
- **Tasks**:
  - Write unit tests for file operations
  - Write tests for undo/redo system
  - Write tests for text manipulation
  - Implement integration tests for UI workflows
  - Test on various Fedora versions
  - Performance testing with large files
- **Success Criteria**: Test suite covers core functionality with >80% coverage

#### 8.1.1 Unit Test Coverage

**Critical Components to Test**:
| Component | Test Focus |
|-----------|------------|
| TextBuffer | Insert, delete, line operations, large files |
| Cursor | Movement, boundary conditions, word navigation |
| Selection | Range calculation, expansion, edge cases |
| UndoRedo | Operation recording, undo, redo, grouping |
| FileIO | Read, write, atomic write, encoding |
| FileTree | Scanning, filtering, sorting |
| Search | Plain text, regex, replace |
| Markdown tokenizer | All token types, edge cases |

**Test File Locations** (per `structure.md`):
- Unit tests: `tests/unit/`
- Integration tests: `tests/integration/`

#### 8.1.2 Test Data and Fixtures

**Test Files to Create**:
| File | Purpose |
|------|---------|
| `empty.md` | Empty file handling |
| `small.md` | Normal file (<10KB) |
| `large.md` | Large file (1MB) |
| `unicode.md` | Unicode characters, emoji, RTL |
| `gfm.md` | All GFM elements |
| `malformed.md` | Invalid/edge case markdown |
| `bom.md` | UTF-8 with BOM |
| `utf16.md` | UTF-16 encoded file |

**Test Scenarios**:
1. **File Operations**: Open, edit, save, close cycle
2. **Undo/Redo**: Complex edit sequences, memory limits
3. **Search**: Various patterns, large files, no matches
4. **Concurrent Edits**: External modification during edit
5. **Recovery**: Crash simulation, recovery file validation

#### 8.1.3 Performance Testing

**Performance Test Cases**:
| Test | Target | How to Measure |
|------|--------|----------------|
| Startup time | <500ms | Time to first frame |
| Open 10MB file | <1s | Time to interactive |
| Type in large file | <50ms latency | Input event to render |
| Scroll large file | 60fps | Frame rate measurement |
| Search in 10MB | <500ms | Time to first result |
| Save 1MB file | <200ms | Time to disk sync |

**Performance Test Tools**:
- `criterion` crate for micro-benchmarks
- Manual timing for startup/large operations
- Memory profiling with `valgrind` or `heaptrack`

#### 8.1.4 Fedora Version Testing

**Target Fedora Versions**:
- Fedora 39 (current stable)
- Fedora 40 (upcoming)
- Fedora Rawhide (development)

**Test Matrix**:
| Version | Desktop | Test Focus |
|---------|---------|------------|
| F39 | GNOME | Standard testing |
| F39 | KDE | Theme integration |
| F39 | COSMIC | Primary target |
| F40 | COSMIC | Future compatibility |

### 8.2 Documentation
- **Objective**: Create user and developer documentation
- **Tasks**:
  - Write user guide
  - Create keyboard shortcuts reference
  - Document build/installation process
  - Add contributing guidelines
  - Create API documentation for developers
- **Success Criteria**: Users can install and use application without external help

#### 8.2.1 User Documentation

**USER_GUIDE.md Contents**:
1. Introduction and features overview
2. Installation instructions (Fedora, Flatpak)
3. Getting started tutorial
4. Editor features walkthrough
5. Markdown syntax reference
6. Configuration guide
7. Troubleshooting common issues
8. FAQ

**KEYBOARD_SHORTCUTS.md Contents**:
- Organized by category
- Searchable format
- Printable reference card

#### 8.2.2 Developer Documentation

**CONTRIBUTING.md Contents**:
1. Development environment setup
2. Building from source
3. Project structure overview
4. Code style guidelines
5. Testing procedures
6. Pull request process
7. Issue reporting guidelines

**ARCHITECTURE.md Contents**:
1. High-level architecture diagram
2. Module responsibilities
3. State management explanation
4. Message flow documentation
5. Extension points (future)

#### 8.2.3 In-App Help

**Help System Features**:
- Keyboard shortcut overlay (Ctrl+?)
- Tooltips on all interactive elements
- Status bar hints for current context
- Link to online documentation

### 8.3 Packaging & Distribution
- **Objective**: Make application available for Fedora
- **Tasks**:
  - Create RPM spec file for Fedora packaging
  - Test RPM build process
  - Submit to Fedora repositories (optional)
  - Create AppImage or Flatpak package (optional)
  - Add application icon and desktop file
- **Success Criteria**: Application installable via standard package manager

#### 8.3.1 Desktop Integration Files

**.desktop File** (`cosmic-notebook.desktop`):
- Name: Cosmic Notebook
- Comment: Markdown editor for COSMIC desktop
- Exec: cosmic-notebook %F
- Icon: cosmic-notebook
- Categories: Office;TextEditor;Utility
- MimeType: text/markdown;text/x-markdown

**AppStream Metadata** (`cosmic-notebook.metainfo.xml`):
- Application description
- Screenshots
- Release notes
- Developer information
- License information

#### 8.3.2 RPM Packaging

**Spec File Key Sections**:
- Name, version, release
- BuildRequires: rust, cargo, libcosmic-devel
- Requires: libcosmic runtime
- %build: cargo build --release
- %install: Binary, desktop file, icons, metainfo
- %files: List all installed files

**Build Dependencies**:
- Rust 1.75+
- Cargo
- libcosmic development headers
- GTK4 development headers (via libcosmic)
- pkg-config

#### 8.3.3 Flatpak Packaging

**Manifest Key Elements**:
- App ID: com.cosmic.Notebook
- Runtime: org.freedesktop.Platform
- SDK: org.freedesktop.Sdk
- Rust extension for building
- Permissions: Home directory access, network (optional)

**Flatpak Advantages**:
- Sandboxed installation
- Cross-distribution compatibility
- Easy updates via Flathub

#### 8.3.4 Icon Requirements

**Icon Sizes** (per `structure.md`):
- 16x16 px - Small lists, menus
- 32x32 px - Desktop, file manager
- 48x48 px - App launcher
- 128x128 px - App stores
- 256x256 px - High-DPI displays
- SVG - Scalable source

**Icon Design Guidelines**:
- Follow COSMIC/Freedesktop icon guidelines
- Recognizable at small sizes
- Work on light and dark backgrounds
- Suggest "notebook" or "markdown"

---

## Phase 9: Desktop Integration & Resilience

### 9.1 Minimal Footprint & Binary Size
- **Objective**: Honor the "minimal footprint" design goal
- **Tasks**:
  - Set target binary size budget and monitor in CI
  - Avoid heavy or unused dependencies; regularly audit `Cargo.toml`
  - Use feature flags to disable non-essential functionality in constrained environments
  - Lazy-initialize optional subsystems (preview, spell checker) only when invoked
- **Success Criteria**: Binary size and runtime memory usage stay within defined budgets without regressions

#### 9.1.1 Binary Size Budget

**Size Targets**:
| Build | Target | Maximum |
|-------|--------|---------|
| Release | 10MB | 15MB |
| Release (stripped) | 8MB | 12MB |
| Release (LTO) | 7MB | 10MB |

**Size Reduction Techniques**:
1. Enable LTO (Link Time Optimization) in release profile
2. Set `codegen-units = 1` for better optimization
3. Strip symbols with `strip = true`
4. Use `opt-level = "z"` for size optimization
5. Set `panic = "abort"` to remove unwinding code

**Dependency Audit Checklist**:
- [ ] Remove unused dependencies monthly
- [ ] Check for lighter alternatives
- [ ] Use feature flags to disable unused crate features
- [ ] Prefer pure Rust over system library wrappers when possible

#### 9.1.2 Feature Flags

**Core Features** (always included):
- Basic editor functionality
- File operations
- Syntax highlighting

**Optional Features**:
| Feature Flag | Contents | Size Impact |
|--------------|----------|-------------|
| `spell-check` | Spell checker integration | +2MB |
| `pdf-export` | PDF export capability | +5MB |
| `code-highlight` | Code block syntax highlighting | +3MB |

**Minimal Build Command**:
`cargo build --release --no-default-features`

**Full Build Command**:
`cargo build --release --all-features`

#### 9.1.3 Lazy Initialization

**Components to Lazy-Initialize**:
| Component | Trigger | Why |
|-----------|---------|-----|
| Spell checker | First document opened or setting enabled | Heavy dependency |
| Markdown preview | First preview mode activation | Parser + renderer |
| PDF exporter | Export menu selection | External process |
| File watcher | Directory opened | System resources |
| Global search index | First global search | Memory for index |

**Implementation Pattern**:
- Store as `Option<T>` in state
- Initialize on first use
- Consider background initialization after startup

### 9.2 Fedora / Desktop Integration
- **Objective**: Integrate cleanly with the Fedora desktop environment
- **Tasks**:
  - Register `.md` (and optionally `.markdown`) file associations
  - Ensure single-instance behavior when opening files from the file manager
  - Follow system theme and font defaults by default; allow overrides via settings
  - Provide proper `.desktop` entry metadata (categories, mime types, icon)
- **Success Criteria**: App can be used as default Markdown editor/viewer and respects system look-and-feel

#### 9.2.1 MIME Type Registration

**Supported MIME Types**:
- `text/markdown` - Standard markdown
- `text/x-markdown` - Alternative markdown
- `text/plain` (low priority) - Can open plain text

**Registration Method**:
- Via .desktop file MimeType field
- User can set as default via system settings

#### 9.2.2 Single Instance Behavior

**Single Instance Goals**:
- Opening file from file manager uses running instance
- New file opens in new tab, not new window
- If app not running, start it

**Implementation Options**:
1. **D-Bus activation**: Register service, receive file open requests
2. **Unix socket**: Check for existing instance, send file path
3. **Lock file**: Detect running instance, use IPC

**Recommended Approach**: D-Bus for best desktop integration on Fedora/COSMIC

**Behavior Matrix**:
| Scenario | Action |
|----------|--------|
| App not running, open file | Start app, open file |
| App running, open file | Focus app, open file in new tab |
| App running, open same file | Focus app, switch to existing tab |

#### 9.2.3 Theme Integration

**System Theme Following**:
- Read COSMIC/GTK theme on startup
- Subscribe to theme change events
- Update colors when theme changes
- Respect accent color setting

**Theme Components**:
| Component | Source |
|-----------|--------|
| Window chrome | System theme |
| Widget colors | System theme |
| Syntax colors | App theme (light/dark variant) |
| Editor background | App theme or system |

**Override Settings**:
- "Follow system theme" toggle (default: on)
- Manual light/dark selection
- Custom accent color (future)

#### 9.2.4 Desktop Entry Details

**Full .desktop Specification**:
```ini
[Desktop Entry]
Type=Application
Name=Cosmic Notebook
GenericName=Markdown Editor
Comment=Lightweight Markdown viewer and editor
Exec=cosmic-notebook %F
Icon=cosmic-notebook
Terminal=false
Categories=Office;TextEditor;Utility;
MimeType=text/markdown;text/x-markdown;
Keywords=markdown;editor;notes;documentation;
StartupNotify=true
StartupWMClass=cosmic-notebook
Actions=new-window;

[Desktop Action new-window]
Name=New Window
Exec=cosmic-notebook --new-window
```

### 9.3 Crash Recovery & Autosave Safety
- **Objective**: Protect user data in the presence of crashes or failures
- **Tasks**:
  - Implement autosave using temporary recovery files separate from main documents
  - On startup, detect and offer recovery of unsaved work from autosave files
  - Define clear behavior when disk write fails (error messaging, retry, safe rollback)
  - Ensure external change conflicts are handled safely together with autosave
- **Success Criteria**: Users can recover most or all unsaved work after unexpected crashes or power loss

#### 9.3.1 Recovery File Strategy

**Recovery vs Auto-Save**:
| Aspect | Recovery Files | Auto-Save to Original |
|--------|----------------|----------------------|
| Location | Separate directory | Same as original |
| User sees | Only on crash recovery | Always current |
| Risk | None to original | Could overwrite |
| Disk usage | Temporary duplicates | No extra space |

**Recommended**: Recovery files (safer)

**Recovery File Location**:
`~/.local/share/cosmic-notebook/recovery/{document-uuid}.md.recovery`

**Recovery Timing**:
- Every 30 seconds if modified
- On focus lost
- On tab switch
- Before potentially dangerous operations

#### 9.3.2 Startup Recovery Flow

**Recovery Detection**:
1. On startup, check recovery directory
2. Load manifest.json listing recovery files
3. For each recovery file:
   - Check if original file exists
   - Compare timestamps and content hash
   - Determine if recovery needed

**Recovery Dialog**:
- Show list of recoverable files
- For each: Original path, last modified, preview snippet
- Options per file: Recover, Discard, Compare (future)
- Global: Recover All, Discard All

**Post-Recovery Cleanup**:
- After user decision, remove recovery files
- If recovered, mark document as modified
- If discarded, just delete recovery file

#### 9.3.3 Disk Write Failure Handling

**Write Failure Scenarios**:
| Scenario | Detection | Response |
|----------|-----------|----------|
| Disk full | Write fails | "Disk full. Free space and retry." |
| Permission denied | Write fails | "Permission denied. Save elsewhere?" |
| File locked | Write fails | "File in use. Retry later." |
| Network drive unavailable | Write fails | "Network unavailable. Save locally?" |

**Retry Logic**:
- Automatic retry: 3 attempts with exponential backoff
- Manual retry: User clicks "Retry" button
- Alternative: "Save As" to different location

**Safe State After Failure**:
- Original file unchanged (atomic write ensures this)
- Recovery file preserved
- User clearly notified
- No data loss

### 9.4 Accessibility
- **Objective**: Make Cosmic Notebook usable for a wide range of users
- **Tasks**:
  - Ensure full keyboard-only navigation for all UI components
  - Provide high-contrast theme option(s)
  - Structure UI in a way that can be surfaced to screen readers where supported by libCosmic
  - Use clear focus indicators and sufficient color contrast for text and controls
- **Success Criteria**: Core workflows are accessible without a mouse; UI meets basic contrast and focus visibility guidelines

#### 9.4.1 Keyboard Accessibility

**Focus Management**:
- Visible focus indicator on all interactive elements
- Focus ring: 2px solid with contrasting color
- Tab order follows logical reading order
- No focus traps (always escapable)

**Keyboard Navigation Coverage**:
| Area | Navigation Method |
|------|-------------------|
| Menu bar | Arrow keys, Enter, Escape |
| Sidebar | Arrow keys, Enter, Tab to editor |
| Tab bar | Arrow keys, Enter, Tab to editor |
| Editor | Full text editing keyboard support |
| Dialogs | Tab between fields, Enter/Escape |
| Command palette | Arrow keys, Enter, Escape |

#### 9.4.2 Visual Accessibility

**Color Contrast Requirements** (WCAG 2.1 AA):
- Normal text: 4.5:1 contrast ratio minimum
- Large text (18pt+): 3:1 contrast ratio minimum
- UI components: 3:1 contrast ratio minimum

**High Contrast Mode**:
- Increase all contrast ratios
- Bolder focus indicators
- Remove or simplify gradients/shadows
- Increase font weight for better readability

**Color Independence**:
- Never convey information by color alone
- Unsaved indicator: Dot + color (not just color)
- Error states: Icon + color + text

#### 9.4.3 Screen Reader Support

**Accessibility Tree** (via libCosmic/iced a11y):
- Label all interactive elements
- Provide roles (button, textbox, listitem, etc.)
- Announce state changes (modified, saved, error)
- Logical reading order

**Accessible Names**:
| Element | Accessible Name |
|---------|-----------------|
| File in sidebar | "filename.md" |
| Tab | "filename.md, modified" or "filename.md" |
| Close button | "Close tab" |
| Save button | "Save file" |
| Editor | "Editor, filename.md, line X column Y" |

#### 9.4.4 Motion and Animation

**Reduced Motion Support**:
- Respect `prefers-reduced-motion` system setting
- Provide setting to disable animations
- Essential animations: Instant instead of animated
- Non-essential: Remove entirely

### 9.5 Internationalization Readiness
- **Objective**: Prepare the app for future localization
- **Tasks**:
  - Centralize all user-facing strings in a single module or resource file
  - Avoid baked-in English strings in layout or logic code
  - Use a simple translation mechanism (e.g., keyed lookup) even if only English is shipped initially
  - Ensure text rendering pipeline handles Unicode correctly (combining characters, RTL where feasible)
- **Success Criteria**: Application can be localized without major code refactors; non-ASCII text in documents renders correctly

#### 9.5.1 String Externalization

**Translation File Format**: Fluent (.ftl)
- Location: `i18n/{locale}/cosmic_notebook.ftl`
- Example: `i18n/en-US/cosmic_notebook.ftl`

**String Organization**:
| Category | Prefix | Examples |
|----------|--------|----------|
| Menu items | menu- | menu-file, menu-edit |
| Button labels | btn- | btn-save, btn-cancel |
| Dialog titles | dlg- | dlg-unsaved-title |
| Dialog messages | msg- | msg-file-not-found |
| Status messages | status- | status-saved |
| Error messages | err- | err-permission-denied |
| Tooltips | tip- | tip-close-tab |

**Fluent Features to Use**:
- Placeholders: `{$filename}`
- Pluralization: `{$count -> [one] file [other] files}`
- Select expressions for gender/variants

#### 9.5.2 Locale Detection

**Locale Detection Priority**:
1. User setting in config (if set)
2. `LANGUAGE` environment variable
3. `LC_MESSAGES` environment variable
4. `LANG` environment variable
5. System locale
6. Fallback: en-US

**Supported Locales** (initial):
- en-US (default, always complete)

**Future Locales** (community contribution):
- es, fr, de, pt-BR, zh-CN, ja, etc.

#### 9.5.3 Unicode Support

**Text Rendering Requirements**:
| Feature | Support Level |
|---------|---------------|
| Basic Latin | Full |
| Extended Latin | Full |
| Cyrillic | Full |
| Greek | Full |
| CJK characters | Full |
| Emoji | Full (with font support) |
| Combining characters | Proper grapheme handling |
| RTL text (Arabic, Hebrew) | Basic (future improvement) |

**Grapheme Handling**:
- Use `unicode-segmentation` crate
- Cursor moves by grapheme, not byte or codepoint
- Selection boundaries on grapheme boundaries
- Character count = grapheme count

#### 9.5.4 Date and Number Formatting

**Locale-Aware Formatting**:
| Data Type | Format |
|-----------|--------|
| Dates | Use system locale format |
| Times | Use system locale format |
| Numbers | Use system locale separators |
| File sizes | Locale-aware (KB, MB, etc.) |

**Implementation**:
- Use `chrono` with locale support
- Or use system formatting via D-Bus

---

## Implementation Priority & Timeline

### Must Have (MVP)
1. **Phase 1**: Core infrastructure
2. **Phase 2**: File management (2.1, 2.2)
3. **Phase 3.1**: Text editor viewport
4. **Phase 3.2**: Tab system
5. **Phase 4.1 & 4.2**: Undo/redo and copy/paste
6. **Phase 6.2**: Essential keyboard shortcuts
7. **Phase 9.3 (core)**: Crash recovery & autosave safety (basic autosave + recovery)

**Estimated: 4-6 weeks**

### Should Have (v1.0)
- Phase 2.3: File watching
- Phase 3.3: File status indicator
- Phase 5.1: Syntax highlighting
- Phase 6.1 & 6.3: Performance & polish
- Phase 7: Settings
- Phase 9.1: Minimal footprint & binary size discipline
- Phase 9.2: Fedora / desktop integration (essentials)

**Estimated: 2-3 weeks**

### Nice to Have (Future)
- Phase 5.2: Preview mode (including PDF export)
- Phase 4.3: Find & replace
- Phase 4.4: Advanced editing (spell checker, frontmatter)
- Phase 8: Full testing & deployment
- Phase 9.4 & 9.5: Advanced accessibility and internationalization
- Advanced features (themes, plugins, etc.)

---

## Development Milestones

### Milestone 1: Hello World (Week 1)
**Goal**: Application launches and shows a window

**Deliverables**:
- Project structure set up per `structure.md`
- Cargo.toml with core dependencies
- Basic cosmic::Application implementation
- Empty window with title "Cosmic Notebook"
- Basic error handling framework

**Acceptance Criteria**:
- `cargo run` opens a window
- No warnings or errors on build
- Window responds to close button

### Milestone 2: Open and View (Week 2-3)
**Goal**: Can open and display a markdown file

**Deliverables**:
- File menu with Open action
- File dialog integration
- TextBuffer with ropey
- Basic text rendering in editor area
- Line numbers gutter
- Scrolling support

**Acceptance Criteria**:
- Can open .md file from dialog
- File contents displayed correctly
- Can scroll through large file
- Line numbers match content

### Milestone 3: Basic Editing (Week 3-4)
**Goal**: Can edit text with cursor and basic operations

**Deliverables**:
- Cursor rendering and positioning
- Text insertion at cursor
- Backspace and delete
- Arrow key navigation
- Home/End, Page Up/Down
- Save functionality (Ctrl+S)

**Acceptance Criteria**:
- Can type text anywhere in document
- Can delete characters
- Can navigate with keyboard
- Changes persist to disk

### Milestone 4: Tabs and Multiple Files (Week 4-5)
**Goal**: VS Code-like tab experience

**Deliverables**:
- Tab bar UI
- Multiple documents open simultaneously
- Tab switching (click and keyboard)
- Tab close with unsaved prompt
- Unsaved indicator (dot)

**Acceptance Criteria**:
- Can open multiple files in tabs
- Switching tabs preserves content
- Unsaved indicator appears on edit
- Confirmation dialog on close unsaved

### Milestone 5: Sidebar and Navigation (Week 5-6)
**Goal**: File browser sidebar

**Deliverables**:
- Sidebar with file tree
- Directory scanning
- Folder expand/collapse
- Click to open file
- Filter/search in sidebar

**Acceptance Criteria**:
- Sidebar shows folder contents
- Clicking file opens in tab
- Folders can be expanded/collapsed
- Filter narrows visible files

### Milestone 6: Undo/Redo and Clipboard (Week 6-7)
**Goal**: Robust editing operations

**Deliverables**:
- Undo stack implementation
- Redo functionality
- Text selection (keyboard and mouse)
- Copy, cut, paste operations
- Selection highlighting

**Acceptance Criteria**:
- Can undo/redo multiple operations
- Can select, copy, paste text
- Clipboard integrates with system
- All operations are undoable

### Milestone 7: Polish and Recovery (Week 7-8)
**Goal**: Production-ready MVP

**Deliverables**:
- Auto-save to recovery files
- Crash recovery on startup
- Essential keyboard shortcuts
- Status bar with stats
- Error handling and dialogs
- Performance optimization pass

**Acceptance Criteria**:
- No data loss on crash
- All documented shortcuts work
- Status bar shows line/column/words
- Handles 1MB+ files smoothly

---

## Technical Stack

- **Language**: Rust
- **GUI Framework**: libCosmic
- **File Operations**: std::fs, notify
- **Text Processing**: ropey or similar
- **Markdown Parsing**: pulldown-cmark or comrak
- **Clipboard**: arboard (cross-platform)
- **Configuration**: serde, toml
- **Internationalization**: i18n-embed with Fluent

### Dependency Decision Matrix

| Need | Options | Chosen | Rationale |
|------|---------|--------|-----------|
| Text buffer | ropey, xi-rope | ropey | Active maintenance, good docs |
| Markdown parser | pulldown-cmark, comrak | pulldown-cmark | Faster, smaller, sufficient features |
| File watching | notify, hotwatch | notify | More mature, better Linux support |
| Clipboard | arboard, clipboard | arboard | Better Linux/Wayland support |
| Serialization | serde_json, toml | toml | Human-readable config files |
| Async runtime | tokio, async-std | tokio | libCosmic integration |

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| libCosmic API instability | Medium | High | Pin versions; abstract API boundaries |
| Performance with large files | Medium | Medium | Lazy loading; rope data structure; profiling |
| File corruption on crash | Low | High | Atomic writes; recovery files; thorough testing |
| Complex undo/redo bugs | Medium | Medium | Simple command pattern; extensive testing |
| Clipboard issues on Wayland | Medium | Low | Test thoroughly; fallback mechanisms |
| Memory leaks | Low | Medium | Regular profiling; Rust ownership helps |
| Theme integration issues | Medium | Low | Test with multiple themes; fallback colors |

### Contingency Plans

**If libCosmic has breaking changes**:
- Maintain abstraction layer over widgets
- Document version compatibility
- Consider pinning to stable release

**If performance targets not met**:
- Profile to identify bottlenecks
- Implement more aggressive lazy loading
- Consider native text rendering optimization
- Reduce feature set if necessary

**If ropey insufficient**:
- Evaluate xi-rope as alternative
- Consider piece table implementation
- Profile specific operations to optimize

---

## Success Metrics

### Quantitative Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Startup time | <500ms | Automated benchmark |
| Input latency | <50ms | Manual testing with timestamp |
| Memory (idle) | <50MB | System monitor |
| Memory (10 files) | <100MB | System monitor |
| Binary size | <15MB | Build output |
| Test coverage | >80% | cargo tarpaulin |

### Qualitative Metrics

| Metric | Criteria |
|--------|----------|
| Usability | First-time user can open, edit, save without help |
| Reliability | No data loss in normal or crash scenarios |
| Responsiveness | No perceived lag during editing |
| Integration | Feels native on COSMIC desktop |
| Accessibility | Basic workflows possible keyboard-only |

### User Feedback Goals

- Conduct usability testing with 5+ users before v1.0
- Gather feedback on:
  - Ease of installation
  - Discoverability of features
  - Performance perception
  - Missing features
  - Bugs encountered

---

## Appendix A: Useful Resources

### libCosmic Resources
- Repository: https://github.com/pop-os/libcosmic
- Examples: https://github.com/pop-os/libcosmic/tree/master/examples
- COSMIC apps for reference: cosmic-edit, cosmic-files, cosmic-term

### Markdown Specifications
- CommonMark: https://commonmark.org/
- GFM: https://github.github.com/gfm/

### Rust Crate Documentation
- ropey: https://docs.rs/ropey
- pulldown-cmark: https://docs.rs/pulldown-cmark
- notify: https://docs.rs/notify
- arboard: https://docs.rs/arboard

### Fedora Packaging
- Rust Packaging Guidelines: https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/
- Desktop Entry Spec: https://specifications.freedesktop.org/desktop-entry-spec/

### Accessibility
- WCAG 2.1: https://www.w3.org/WAI/WCAG21/quickref/
- Accessible Rich Internet Applications: https://www.w3.org/WAI/ARIA/apg/

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| Buffer | In-memory representation of file contents |
| Document | Complete state of an open file (buffer + metadata) |
| Grapheme | User-perceived character (may be multiple codepoints) |
| GFM | GitHub Flavored Markdown |
| LTO | Link Time Optimization |
| Rope | Data structure for efficient text manipulation |
| Recovery file | Temporary backup for crash recovery |
| Viewport | Visible portion of document |
| XDG | Cross-Desktop Group (freedesktop.org standards) |
