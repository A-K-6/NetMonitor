# Task 017: Custom Themes & Theme Selector

**Status:** Completed
**Phase:** 3 (Advanced)
**Objective:** Implement a theme engine and a theme selector dialog to allow users to customize the color scheme of the TUI.

## 1. Research & Strategy
- [x] **Data Model:** 
    - Define a `Theme` struct in `netmonitor/src/ui.rs` or a new module `netmonitor/src/theme.rs`.
    - `Theme` fields: `header_fg`, `row_fg`, `highlight_bg`, `border_fg`, `alert_fg`, etc.
    - Store available themes (e.g., `Default`, `Dracula`, `Solarized`, `Monokai`).
    - Add `current_theme: ThemeType` to `App`.
- [x] **UI Component:**
    - Create a dialog to select a theme (hotkey `t`).
    - Ensure all rendering functions use the colors defined in the current theme.
- [x] **Logic:**
    - Add logic to cycle or select from available themes.
    - (Optional) Persist theme selection in a simple config file or in the existing DB.

## 2. Implementation (Userspace)
- [x] **Theme Module:** Create `netmonitor/src/theme.rs` to define the theme structure and preset themes.
- [x] **App State:** Update `netmonitor/src/app.rs` to include `current_theme`.
- [x] **Keyboard Handling:** 
    - Update `netmonitor/src/main.rs` to handle `t` (Toggle/Select Theme).
- [x] **Rendering:**
    - Update all rendering calls in `netmonitor/src/ui.rs` to reference the theme colors.
    - Create `ui::render_theme_dialog` to allow users to choose from a list of themes.

## 3. Verification
- [x] **Functional Test:** Verify that changing the theme immediately updates all UI components (header, table, borders, alerts).
- [x] **UI Test:** Ensure the theme selector dialog is intuitive and all themes are legible.

## 4. Documentation
- [x] Log completion in `devlog.md`.
- [x] Update `ROADMAP.md`.
