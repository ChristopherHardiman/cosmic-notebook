# Cosmic Notebook - Scope

Simple Markdown viewer editor using libCosmic for the Fedora Operating System.

## Core Features

- Quickly view/edit md files that are formatted using Markdown format
- Minimal footprint with quick responsive edits
- Sidebar to quickly switch between md files in a folder
- Tabs to have multiple files open at once much like VS Code
- An indicator to track what files have been saved or have pending updates
- Dynamic edits system - a file can be edited while being in use and show changes
- A very robust copy paste undo redo system

## Writing & Editing

- Auto-save functionality to prevent data loss
- Spell checking and word/character count statistics
- Distraction-free mode to focus on writing
- GitHub Flavored Markdown support (tables, task lists)
- Image handling via drag & drop and clipboard paste
- Split view for side-by-side editing and preview

## Navigation & Search

- Global search across all files in the folder
- Session restoration to remember open files and scroll positions
- Command palette for quick access to features
- Recent files quick access menu

## Export & Output

- Export options for HTML and PDF

## User Configuration

- Theme selection (light/dark, follows system theme)
- Font size and tab width settings
- Word wrap toggle
- Persistent settings between sessions

## Data Safety & Recovery

- Crash recovery with automatic backup restoration
- External file conflict resolution (reload/overwrite/diff prompts)
- Atomic file writes to prevent corruption

## Desktop Integration

- Fedora desktop integration with `.md` file associations
- Single-instance behavior when opening files from file manager
- Proper `.desktop` entry with categories, mime types, and icon
- Follows system theme and font defaults

## Accessibility

- Full keyboard-only navigation for all UI components
- High-contrast theme option
- Clear focus indicators and sufficient color contrast
- Keyboard shortcuts help overlay

## Internationalization

- Internationalization-ready architecture (centralized strings)
- Proper Unicode handling for non-ASCII text

## Performance Targets

- Application launches in <500ms
- Smooth scrolling and text editing with <50ms input latency
- Handles files up to 10MB efficiently
- Memory usage <100MB for typical usage
- Undo/redo depth >100 operations without performance degradation

