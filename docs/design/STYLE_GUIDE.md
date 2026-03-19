# Design & Style Guide: NetMonitor TUI

This document defines the visual identity and interaction principles for the NetMonitor Terminal User Interface.

## 1. Design Philosophy
- **High Information Density:** Maximize the data visible on a single screen without overwhelming the user.
- **Color with Purpose:** Use colors to highlight critical information (e.g., high bandwidth, active status, errors) rather than for pure decoration.
- **Modern & Secure:** The UI should look like a professional, modern Linux utility, similar to `btop` or `lazygit`.

## 2. Color Palette (ANSI)
NetMonitor uses standard ANSI colors to ensure compatibility across different terminal emulators and themes.

| Element | Color | Purpose |
| :--- | :--- | :--- |
| **Accent** | Blue / Cyan | Headers, Borders, and Active Selections. |
| **Upload** | Green | Outgoing data (Traffic Leaving). |
| **Download** | Yellow / Amber | Incoming data (Traffic Entering). |
| **Danger** | Red | "Kill-Switch" confirmation, Errors, and High Usage Thresholds. |
| **Dimmed** | Gray / Dark Gray | Secondary info (PIDs, inactive rows, and helper text). |

## 3. UI Components & Layout

### 3.1. Tables
- **Active Row:** Highlighted with a background color (e.g., Blue) or a prefix character (`>`).
- **Column Sorting:** The currently sorted column header should be underlined or prefixed with an arrow (e.g., `↓ UP`).
- **Alignment:** 
    - PIDs: Left-aligned.
    - Names: Left-aligned.
    - Bandwidth (KB/s): Right-aligned for easy comparison.

### 3.2. Borders & Spacing
- Use **Block Borders** (`symbols::border::PLAIN` or `THICK`) to separate functional areas.
- Maintain a **1-character padding** inside widgets to prevent text from touching borders.

## 4. Interaction Patterns
- **Confirmation Dialogs:** Destructive actions (like killing a process) MUST show a centered pop-up window requiring explicit `y/n` confirmation.
- **Dynamic Resizing:** Widgets must adapt to terminal size; if the terminal is too small, hide secondary columns (e.g., "Total Data") before hiding primary ones.
- **Smooth Refreshing:** Use double-buffering (provided by Ratatui) to prevent screen flickering during high-frequency updates.

## 5. Typography
- Assume a **monospaced font**.
- Use **bold** sparingly for column headers and critical alerts.
- Use **italic** (if supported) for help hints in the footer.
