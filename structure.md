```
cosmic-notebook/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── assets/
│   ├── icons/
│   │   ├── cosmic-notebook.svg          # Application icon (scalable)
│   │   ├── cosmic-notebook-16.png
│   │   ├── cosmic-notebook-32.png
│   │   ├── cosmic-notebook-48.png
│   │   ├── cosmic-notebook-128.png
│   │   ├── cosmic-notebook-256.png
│   │   ├── file-markdown.svg            # File type icon
│   │   ├── folder-open.svg
│   │   ├── folder-closed.svg
│   │   ├── tab-close.svg
│   │   ├── tab-unsaved.svg
│   │   ├── search.svg
│   │   ├── settings.svg
│   │   ├── preview.svg
│   │   ├── split-view.svg
│   │   └── distraction-free.svg
│   └── themes/
│       ├── syntax-light.toml
│       └── syntax-dark.toml
├── i18n/
│   ├── en-US/
│   │   └── cosmic_notebook.ftl          # Fluent translation file
│   └── messages.ftl                      # Fallback/reference strings
├── src/
│   ├── main.rs                           # Application entry point
│   ├── lib.rs                            # Library root (optional, for testing)
│   ├── app.rs                            # Main application struct & cosmic::Application impl
│   ├── config.rs                         # Configuration management
│   ├── localization.rs                   # i18n string loading
│   ├── error.rs                          # Custom error types
│   ├── message.rs                        # Application message enum
│   ├── state/
│   │   ├── mod.rs
│   │   ├── app_state.rs                  # Global application state
│   │   ├── editor_state.rs               # Per-editor state (cursor, selection, undo)
│   │   ├── tab_state.rs                  # Tab management state
│   │   ├── sidebar_state.rs              # File tree state
│   │   └── session_state.rs              # Session persistence
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs                # Primary window layout
│   │   ├── sidebar/
│   │   │   ├── mod.rs
│   │   │   ├── file_tree.rs              # File tree widget
│   │   │   ├── file_item.rs              # Individual file/folder item
│   │   │   └── search_bar.rs             # Sidebar search filter
│   │   ├── editor/
│   │   │   ├── mod.rs
│   │   │   ├── text_editor.rs            # Main text editing widget
│   │   │   ├── line_numbers.rs           # Line number gutter
│   │   │   ├── cursor.rs                 # Cursor rendering & blinking
│   │   │   ├── selection.rs              # Selection highlighting
│   │   │   └── syntax_highlight.rs       # Markdown syntax coloring
│   │   ├── tabs/
│   │   │   ├── mod.rs
│   │   │   ├── tab_bar.rs                # Tab bar container
│   │   │   └── tab_item.rs               # Individual tab widget
│   │   ├── preview/
│   │   │   ├── mod.rs
│   │   │   ├── markdown_renderer.rs      # Markdown to widget conversion
│   │   │   └── split_view.rs             # Side-by-side layout
│   │   ├── dialogs/
│   │   │   ├── mod.rs
│   │   │   ├── save_dialog.rs            # Unsaved changes confirmation
│   │   │   ├── conflict_dialog.rs        # External file change conflict
│   │   │   ├── find_replace.rs           # Find & replace dialog
│   │   │   ├── settings_dialog.rs        # Settings panel
│   │   │   └── command_palette.rs        # Quick command access
│   │   ├── status_bar.rs                 # Bottom status bar
│   │   └── toolbar.rs                    # Optional toolbar (if needed)
│   ├── editor/
│   │   ├── mod.rs
│   │   ├── buffer.rs                     # Text buffer using ropey
│   │   ├── cursor.rs                     # Cursor position logic
│   │   ├── selection.rs                  # Selection range logic
│   │   ├── operations.rs                 # Text operations (insert, delete, etc.)
│   │   ├── undo_redo.rs                  # Undo/redo stack implementation
│   │   └── clipboard.rs                  # System clipboard integration
│   ├── file_handler/
│   │   ├── mod.rs
│   │   ├── file_io.rs                    # Read/write operations
│   │   ├── file_tree.rs                  # Directory scanning
│   │   ├── file_watcher.rs               # notify integration
│   │   ├── atomic_write.rs               # Safe file writing
│   │   ├── backup.rs                     # Backup/recovery file management
│   │   └── recent_files.rs               # Recent file tracking
│   ├── markdown/
│   │   ├── mod.rs
│   │   ├── parser.rs                     # Markdown parsing (pulldown-cmark)
│   │   ├── tokenizer.rs                  # Syntax token extraction
│   │   ├── renderer.rs                   # HTML/widget rendering
│   │   └── export.rs                     # HTML/PDF export
│   ├── search/
│   │   ├── mod.rs
│   │   ├── text_search.rs                # In-file search
│   │   └── global_search.rs              # Cross-file search
│   └── utils/
│       ├── mod.rs
│       ├── debounce.rs                   # Event debouncing utility
│       ├── paths.rs                      # Path manipulation helpers
│       └── platform.rs                   # Platform-specific utilities
├── tests/
│   ├── integration/
│   │   ├── file_operations_test.rs
│   │   ├── editor_test.rs
│   │   └── undo_redo_test.rs
│   └── unit/
│       ├── buffer_test.rs
│       ├── markdown_test.rs
│       └── search_test.rs
├── packaging/
│   ├── cosmic-notebook.desktop           # Desktop entry file
│   ├── cosmic-notebook.metainfo.xml      # AppStream metadata
│   ├── cosmic-notebook.spec              # RPM spec file
│   └── flatpak/
│       └── com.cosmic.Notebook.json      # Flatpak manifest
└── docs/
    ├── USER_GUIDE.md
    ├── KEYBOARD_SHORTCUTS.md
    ├── CONTRIBUTING.md
    └── ARCHITECTURE.md
```