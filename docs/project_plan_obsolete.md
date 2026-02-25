dafer-utils/docs/project_plan.md

# Data Science UI Tool: Project Plan & Summary

## Project Aim

The goal of this project is to develop a user-friendly, persistent UI application that empowers data scientists to efficiently work with tabular data formats, specifically CSV and Parquet files. The application will streamline the workflow for loading, previewing, modifying, and visualizing datasets, making common data wrangling and exploration tasks accessible and intuitive. The main focus on the app is to be lightweight, reliable, fast and performant using rust.

---

## Desired Behavior & Scope

### Core Features

1. **Multi-Tab Interface**
   - The UI will be organized into multiple tabs, each dedicated to a specific aspect of the data science workflow.

2. **Tab 1: Data Loading & Preview**
   - Load CSV, Parquet, or other tabular data files.
   - Display a preview of the loaded data (head/tail, column types, basic stats).
   - Support for drag-and-drop and file picker.
   - Show file metadata (size, number of rows/columns, etc.).

3. **Tab 2: Data Modification**
   - Provide tools for common data cleaning and transformation tasks:
     - Fill forward/backward (e.g., for missing values).
     - Drop rows/columns with missing data (dropna).
     - Filter, sort, and select columns.
     - Basic type conversions and renaming.
     - Groupby, multi index...
     - Convert string to datetime and the other way around.
   - Preview changes before applying.

4. **Tab 3: Data Visualization**
   - Enable graphical inspection of the data:
     - Quick plotting (histograms, scatter, line, box, etc.).
     - Column selection for axes.
     - Basic customization (labels, colors, aggregation).
     - Visual filtering, point selection in scatter plots.
     - Proper type handling in axis, like datetime.
     - Exporting of the visuals.
   - Interactive exploration (zoom, pan, select).

---

## Recommended Folder Structure

To keep the project organized and modular, use the following structure for the UI and core functionalities:

```
dafer-utils-ui/
└── src/
    ├── main.rs
    ├── app.rs
    ├── enums.rs
    ├── state.rs
    ├── ui.rs
    └── ui/
        ├── main_ui.rs         # Main workspace layout and tab logic
        ├── load_preview.rs    # Tab 1: Data loading & preview logic/components
        ├── modify.rs          # Tab 2: Data modification logic/components
        ├── visualize.rs       # Tab 3: Data visualization logic/components
        └── widgets/           # (Optional) Reusable UI widgets/components
```

- Each tab gets its own file for clarity and separation of concerns.
- Place reusable UI elements in `widgets/`.
- Keep business/data logic in the corresponding tab file or further split into submodules if needed.

---

## UI/UX Note: Vertical Tabs & Workspace Layout

- **Vertical Tabs:**
  - The main UI should use vertical tabs (on the left) for navigation, not horizontal tabs.
  - The left region will be a vertical tab bar; the main workspace to the right will show the content for the selected tab.
  - This improves usability and leaves more space for data and visualizations.
- **Implementation:**
  - Use `egui::SidePanel::left` for the vertical tab bar.
  - The main workspace should be the central panel (`egui::CentralPanel`).
  - Only the tab bar and workspace should be visible (no separators between them).

---

## Methodologies & Technologies

- **Language:** ALL in Rust since I want to practice and learn.
- **Frontend/UI:** Egui with any other required libraries for desktop native lightweight apps with no complex frameworks.
- **Backend/Data Processing:** Rust (Polars, Arrow) for efficient data handling. Whatever is more performant, better Polars.
- **File Support:** CSV, Parquet (extendable to others, maybe pickle or txt separated by ";", "," or tabs).
- **State Management:** Persistent state for loaded datasets and user actions.
- **Testing:** Unit and integration tests for UI and data operations.
- **Documentation:** Clear user and developer documentation.

---

## Out of Scope (for MVP)

- Real-time collaboration.
- Advanced machine learning or modeling features.
- Cloud storage integration (local files only for MVP).

---

## Summary

This project aims to deliver a robust, extensible, and user-friendly UI for data scientists to load, clean, and visualize tabular data (but also other csv structured files). The focus is on usability, performance, and providing essential tools for everyday data wrangling and exploration tasks. The plan and scope outlined here should be referenced throughout development to ensure alignment with the project's goals.

---
