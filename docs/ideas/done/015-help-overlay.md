# Task 015: Help Overlay & Keybinding Documentation

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Implement a dedicated help screen (`?`) that explains all hotkeys, navigation, and UI components, ensuring the tool is self-documenting for new users.

## 1. Research & Strategy
- [ ] **UI Component:** Use `ratatui::widgets::Clear` and a centered `Paragraph` or `Table` to display keybindings.
- [ ] **State Management:** Add `show_help: bool` to `App` struct.
- [ ] **Content:** Document all current hotkeys:
    - `q` / `Esc`: Quit / Back
    - `k`: Kill process
    - `s`: Cycle sort column
    - `/`: Filter by name
    - `Enter`: Process details (Deep-dive)
    - `g`: Graph view
    - `?`: Toggle help

## 2. Implementation (Userspace)
- [ ] **App State:** Add `pub show_help: bool` to `App` in `netmonitor/src/app.rs`.
- [ ] **Keyboard Handling:** 
    - Update `netmonitor/src/main.rs` to handle `?` (or `h`).
    - Ensure `Esc` closes the help overlay if it's open.
- [ ] **Rendering:**
    - Create `ui::render_help_overlay` in `netmonitor/src/ui.rs`.
    - Use a clean table layout for "Key" vs "Action" descriptions.
    - Style the overlay with a distinct border color (e.g., Magenta or Blue).

## 3. Verification
- [ ] **Visual Test:** Press `?` and verify the overlay is centered and correctly displays all keys.
- [ ] **Interaction Test:** Ensure no other hotkeys (like 'k' or 'g') trigger while the help overlay is active, or that `Esc` reliably returns to the main table.

## 4. Documentation
- [ ] Log completion in `devlog.md`.
