# Task 024: Adaptive Theming (System Integration)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Enhance the theming engine to support terminal-native colors (transparency/ANSI) and automatically detect system-wide dark/light mode on startup.

## 1. Research & Strategy
- [ ] **Terminal Color Support:**
    - Investigate `Color::Reset` in Ratatui for background transparency.
    - Research how to use 16-color ANSI palettes for better compatibility with terminal-wide themes.
- [ ] **System Theme Detection:**
    - Research crates like `dark-light` or `auto-theme` for cross-platform (Linux focused) theme detection.
    - Plan how to map system "light" vs "dark" to our internal theme presets.

## 2. Implementation (Userspace)
- [ ] **Theme Engine Update:**
    - Add `ThemeType::Terminal` to `theme.rs`.
    - Implement `ThemeType::Terminal` using `Color::Reset` for backgrounds and standard ANSI colors for foregrounds.
- [ ] **Auto-Detection Logic:**
    - Integrate a theme detection library in `main.rs`.
    - On startup, if no theme is specified (or if "Auto" is selected), set `current_theme` based on system state.
- [ ] **App State Update:**
    - Add `ThemeType::Auto` to the theme selector.
    - Update `App::new()` to perform initial detection.

## 3. Verification
- [ ] **Transparency:** Confirm that the "Terminal" theme respects the terminal emulator's background transparency/image.
- [ ] **Detection:** Verify that changing the system theme (if possible) or starting in different environments selects the correct default.
- [ ] **Consistency:** Ensure all widgets correctly inherit the "Reset" or "Default" colors when the Terminal theme is active.

## 4. Documentation
- [ ] Update `devlog.md` with progress.
- [ ] Mark as complete in `ROADMAP.md` once finished.
