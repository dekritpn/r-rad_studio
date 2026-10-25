# Master Engineering Specification & Multi-Agent Directives

**Project Codename:** R-RAD Studio (R Rapid Application Development & Statistical GUI)

**Target Architecture:** Tauri v2 (Rust Backend) + Svelte 5 (Frontend UI & Form Designer) + Persistent R Process + Integrated WebView Output Engine

---

## 1. System Vision & Core Tenets

1. **Everything is a Form Schema:** 100% of the statistical dialogs shipped with the application (emulating SPSS menus) MUST be built using the application's own JSON Form Schema and rendered dynamically at runtime. No hardcoded statistical dialogs.
2. **RAD Extensibility (VB6 / Qt Designer Paradigm):** Users can switch any dialog into "Design Mode," drag-and-drop statistical widgets onto a canvas, bind them to an R code template, and mount the saved `.rform.json` file directly into the application's top menu bar.
3. **Browser-Native Output Stream:** All R output (tables, plots, text logs) is captured, transformed into clean HTML/CSS/SVG, and appended to a simplified, interactive HTML document viewer with live Outline navigation.
4. **Reproducible Transparency:** Every GUI action generates clean, human-readable R syntax (`tidyverse` / `broom` / `gt` preferred) that is logged alongside the visual output and can be exported as a standalone `.R` or `.qmd` script.
5. **Open Project & Multi-Format Interoperability:** Seamless opening, importing, and exporting to CSV, SPSS (`.sav`), and Excel (`.xlsx`). Entire workspace state (active data, variable dictionary, syntax log, and rendered outputs) can be preserved in an open, fully documented JSON project specification (`.rrad`).
6. **Interactive Data Wrangling & Transformations:** Users can directly manipulate the data schema (renaming variables, adding blank columns, and deriving new columns via formula expressions) with immediate bidirectional synchronization with the underlying R runtime.
7. **Spreadsheet-Style Grid Manipulation & Find/Replace:** In-place cell editing, arbitrary row insertion/deletion, tab-separated clipboard interoperability (Excel/Google Sheets copy-paste), and a dedicated Find & Replace dialog with scope and precision matching filters.

---

## 2. Agent Team Topology & Role Boundaries

When executing tasks on this repository, adopt the specific persona required for the current module and strictly respect interface contracts between boundaries:

| Agent Role | Responsibility Scope | Primary Tech Stack |
| :--- | :--- | :--- |
| `@Lead-Architect` | Repository scaffolding, IPC message contracts, JSON schema validation, state synchronization, build pipelines. | Rust, TypeScript, JSON Schema, Git |
| `@Systems-Rust` | Tauri backend, persistent R daemon lifecycle (`R --vanilla`), stdin/stdout/socket IPC, Apache Arrow data serialization, file I/O. | Rust (`tokio`, `serde`, `arrow`, `tauri`) |
| `@Frontend-Core` | Main application shell, dynamic menu bar, SPSS-style Dual Data Grid ("Data View" & "Variable View"), HTML Output Viewer. | Svelte 5 (Runes), TypeScript, TailwindCSS, Canvas Grid |
| `@RAD-Engine` | The Visual Form Builder (drag-and-drop canvas, property inspector), Dynamic Form Renderer, and Handlebars/Mustache R-code interpolation engine. | Svelte 5, TypeScript, `handlebars` |
| `@R-Stat-Integration` | R runtime wrapper package (`rradbridge`), HTML table/plot capture (`gt`, `svglite`), and authoring the built-in SPSS-equivalent `.rform.json` catalog. | R (`jsonlite`, `broom`, `gt`, `ggplot2`, `haven`) |
| `@QA-Verifier` | Unit tests, IPC stress tests (100k+ row datasets), schema validation tests, and end-to-end form execution verification. | Cargo Test, Vitest, Playwright, `testthat` |

---

## 3. Repository Structure

All agents must adhere to this exact directory layout:

```plaintext
r-rad-studio/
├── src-tauri/                  # @Systems-Rust domain
│   ├── src/
│   │   ├── main.rs
│   │   ├── r_daemon.rs         # Spawns and manages persistent R session
│   │   ├── data_bridge.rs      # Arrow / JSON serialization between R and UI
│   │   ├── form_registry.rs    # Scans built-in and user custom .rform.json files
│   │   └── ipc_commands.rs     # Tauri #[tauri::command] handlers
│   └── Cargo.toml
├── src/                        # @Frontend-Core & @RAD-Engine domain
│   ├── lib/
│   │   ├── components/
│   │   │   ├── grid/           # Virtualized Data View & Variable View
│   │   │   ├── output/         # Simplified Browser Output Canvas & Outline
│   │   │   ├── runtime/        # Dynamic Form Renderer & Statistical Widgets
│   │   │   └── designer/       # VB6/Qt-style Drag-and-Drop Form Builder
│   │   ├── schemas/            # TypeScript interfaces & JSON Schema validators
│   │   └── stores/             # Reactive state (active dataset, menu tree, output log)
│   └── App.svelte
├── r-runtime/                  # @R-Stat-Integration domain
│   ├── bridge.R                # Persistent socket/stdio listener loop in R
│   ├── formatters.R            # Converts R objects/plots to HTML/SVG payloads
│   └── metadata.R              # Extracts & mutates variable labels, levels, types
└── forms-catalog/              # Shipped SPSS-emulating forms (.rform.json)
    ├── descriptives/
    │   ├── frequencies.rform.json
    │   └── crosstabs.rform.json
    ├── compare-means/
    │   ├── independent-t-test.rform.json
    │   └── one-way-anova.rform.json
    └── regression/
        ├── linear.rform.json
        └── binary-logistic.rform.json
```

---

## 4. Core Technical Contracts

### 4.1. The `.rform.json` Declarative Schema

Every statistical dialog—built-in or user-created—MUST conform to this JSON specification:

```json
{
  "$schema": "https://r-rad-studio.local/schemas/v1/rform.json",
  "id": "stat.compare_means.ind_t_test",
  "version": "1.0.0",
  "menu_path": ["Analyze", "Compare Means", "Independent-Samples T Test..."],
  "window": { "title": "Independent-Samples T Test", "width": 620, "height": 440 },
  "layout": [
    {
      "id": "src_vars",
      "type": "VariableSourceList",
      "grid": { "x": 1, "y": 1, "w": 5, "h": 10 },
      "filter_types": ["scale", "ordinal", "nominal"]
    },
    {
      "id": "test_vars",
      "type": "VariableTargetSlot",
      "label": "Test Variable(s):",
      "grid": { "x": 6, "y": 1, "w": 6, "h": 5 },
      "accepts": { "types": ["scale"], "multiple": true, "required": true }
    },
    {
      "id": "group_var",
      "type": "VariableTargetSlot",
      "label": "Grouping Variable:",
      "grid": { "x": 6, "y": 6, "w": 6, "h": 2 },
      "accepts": { "types": ["nominal", "ordinal"], "multiple": false, "required": true }
    },
    {
      "id": "chk_welch",
      "type": "Checkbox",
      "label": "Assume equal variances (Student's t)",
      "grid": { "x": 6, "y": 8, "w": 6, "h": 1 },
      "default": false
    }
  ],
  "sub_dialogs": [
    {
      "id": "btn_options",
      "label": "Options...",
      "layout": [
        {
          "id": "conf_level",
          "type": "NumericInput",
          "label": "Confidence Interval Percentage:",
          "default": 95,
          "min": 1,
          "max": 99.9
        }
      ]
    }
  ],
  "r_template": "local({\n  {{#each test_vars}}\n  res <- t.test({{this}} ~ {{../group_var}}, data = .active_data, var.equal = {{../chk_welch}}, conf.level = {{../sub_dialogs.btn_options.conf_level}} / 100)\n  rrad_emit_table(broom::tidy(res), title = 'T-Test: {{this}} by {{../group_var}}')\n  {{/each}}\n})"
}
```

### 4.2. The Statistical Widget Library

`@RAD-Engine` must implement the following draggable components for the Form Builder:

1. **`VariableSourceList`:** Automatically binds to the active dataset's column metadata; displays icons for Scale (ruler), Ordinal (bars), and Nominal (circles).
2. **`VariableTargetSlot`:** Receives variables from `VariableSourceList` via drag-and-drop or directional arrow buttons `[ > ]`. Validates measurement level constraints and required flags before enabling the form's **OK** and **Paste** buttons.
3. **`FormulaBuilder`:** Allows combining selected variables with interaction terms (`*`, `:`, `+`).
4. **Standard Controls:** `Checkbox`, `RadioGroup`, `Dropdown`, `NumericInput`, `TextInput`, `GroupBox`, and `SubDialogButton` (spawns a child modal and merges its state dictionary into the parent namespace).

### 4.3. R Daemon & Output Protocol

`@Systems-Rust` and `@R-Stat-Integration` must communicate over a local asynchronous TCP socket or named pipe:

* **Request Payload (Rust -> R):**

```json
{
  "request_id": "uuid-1234",
  "command_type": "EXECUTE_ANALYSIS",
  "r_code": "..."
}
```

* **Response Stream (R -> Rust -> Frontend Output Browser):**
  R captures execution using `evaluate` or custom `tryCatch` wrappers and emits structured blocks:

```json
{
  "request_id": "uuid-1234",
  "blocks": [
    { "type": "syntax", "content": "t.test(mpg ~ am, data = .active_data)" },
    { "type": "html_table", "title": "Independent Samples Test", "html": "<table>...</table>" },
    { "type": "svg_plot", "title": "Distribution Plot", "svg": "<svg>...</svg>" },
    { "type": "warning", "content": "Cannot compute exact p-value with ties" }
  ]
}
```

### 4.4. Multi-Format Data I/O Protocol (CSV, SAV, XLSX)

The application provides native, lossless data exchange across three primary statistical formats:

* **Import Formats:**
  * **CSV (`.csv`):** Parsed via `read.csv(..., stringsAsFactors = FALSE, check.names = FALSE)`.
  * **SPSS (`.sav`):** Parsed via `haven::read_sav(...)`, extracting variable labels, value label dictionaries, and missing value indicators.
  * **Excel (`.xlsx`, `.xls`):** Parsed via `readxl::read_excel(...)` with header detection and column type casting.
* **Export Formats:**
  * **CSV (`.csv`):** `write.csv(.active_data, file, row.names = FALSE)`.
  * **SPSS (`.sav`):** `haven::write_sav(.active_data, file)`.
  * **Excel (`.xlsx`):** `writexl::write_xlsx(.active_data, path = file)`.
* **IPC Contract:**
  * `load_dataset(file_path: String) -> DatasetMetadata`
  * `export_dataset(file_path: String, format: "csv" | "sav" | "xlsx") -> ()`

### 4.5. Dynamic Data Schema Mutation & Transformation

Users can perform interactive data wrangling synchronized in real time with the persistent R session:

1. **Variable Renaming:**
   * In Variable View, the **Name** field is editable.
   * Enforces R naming constraints (unique, no illegal symbols).
   * Synchronizes with R via `names(.active_data)[which(names(.active_data) == old_name)] <- new_name`.
2. **Add Blank Column:**
   * Appends a new variable with default name (e.g. `VAR00001` or user-defined), default measurement level (`scale`), and initialized `NA` values across all rows.
   * Immediately updates Data View columns and Variable View row count.
3. **Compute Variable (Formula Transformation):**
   * Accessible via **Transform $\rightarrow$ Compute Variable...**.
   * Allows specifying a **Target Variable** and an arbitrary R **Numeric Expression** (e.g., `log(mpg) * 2`, `weight / (height / 100)^2`, `ifelse(hp > 150, 1, 0)`).
   * Evaluated securely within `.active_data` scope: `.active_data[[target_var]] <- with(.active_data, eval(parse(text = expr)))`.
   * Automatically logs reproducible R syntax to the Output Viewer and refreshes the data grid.

### 4.6. Open Project File Specification (`.rrad`)

R-RAD Studio defines an open, human-readable JSON project standard (`.rrad` / `.rrad.json`) that saves and restores complete analytical workspace sessions:

```json
{
  "$schema": "https://r-rad-studio.local/schemas/v1/project.json",
  "version": "1.0.0",
  "project_meta": {
    "title": "Cardiovascular Analysis 2026",
    "saved_at": "2026-10-25T11:45:00Z",
    "r_version": "R version 4.5.3"
  },
  "dataset": {
    "dataset_name": "clinical_trial.sav",
    "row_count": 1500,
    "column_count": 14,
    "variables": [
      {
        "name": "patient_id",
        "type": "numeric",
        "measure": "scale",
        "label": "Patient ID Number",
        "values": [],
        "missing": "None"
      }
    ],
    "rows": [
      { "patient_id": 1001, "age": 45, "treatment": 1 }
    ]
  },
  "syntax_history": "# Past analysis syntax...",
  "output_logs": [
    {
      "id": "out_01",
      "timestamp": "11:30:15",
      "title": "Independent-Samples T Test",
      "blocks": [
        { "type": "syntax", "content": "t.test(bp ~ group, data = .active_data)" },
        { "type": "html_table", "title": "Group Statistics", "html": "<table>...</table>" },
        { "type": "svg_plot", "title": "Boxplot", "svg": "<svg>...</svg>" }
      ]
    }
  ]
}
```

* **IPC Contract:**
  * `save_project(file_path: String, project_data: Value) -> ()`
  * `open_project(file_path: String) -> Value`

---

## 5. Phased Execution Roadmap

Agents must execute this build sequentially by phase. Do not advance to Phase $N+1$ until all Verification Gates for Phase $N$ pass.

```plaintext
[Phase 1: Core IPC & R Daemon]
       │
       ▼
[Phase 2: SPSS Data/Variable Grid, File I/O & Output View]
       │
       ▼
[Phase 3: Dynamic Form Renderer & R Template Engine]
       │
       ▼
[Phase 4: Visual Form Designer (VB6/Qt-Style Builder)]
       │
       ▼
[Phase 5: Standard SPSS Dialog Catalog & Packaging]
```

### Phase 1: Foundation, R Daemon & IPC Bridge

* **Tasks:**
  1. Initialize Tauri v2 + Svelte 5 + TypeScript workspace.
  2. Implement `r_daemon.rs` to locate the system R binary (or bundled portable R), launch `Rscript r-runtime/bridge.R`, and establish a persistent bidirectional JSON-RPC channel.
  3. In `r-runtime/`, implement `.active_data` environment management and `haven` data loaders (`.sav`, `.csv`, `.rds`, `.xlsx`).
* **Verification Gate 1:** Rust test suite successfully spawns R, loads a 50,000-row `.sav` file, queries variable names/types/labels in under 150ms, and cleanly terminates the R child process on drop.

### Phase 2: Dual Data Grid, File I/O, Data Wrangling & Browser Output

* **Tasks:**
  1. Build the **Data View** (virtualized canvas grid supporting 100k+ rows via paginated Arrow/IPC slices) and **Variable View** (editable metadata table for Name, Type, Width, Decimals, Label, Values, Missing, Measure).
  2. Implement **Multi-Format Data I/O**: Open and import `.csv`, `.sav`, and `.xlsx` files; export active dataset to `.csv`, `.sav`, and `.xlsx`.
  3. Implement **Data Schema Mutation**: Inline variable name editing in Variable View, Add Blank Column, and **Transform $\rightarrow$ Compute Variable...** formula evaluator.
  4. Implement **Spreadsheet-Style Grid Operations**: Inline data cell editing in Data View (double-click/F2), row insertion and deletion (context menu & toolbar), and tab-separated (TSV) clipboard copy & paste interoperable with Microsoft Excel / Google Sheets.
  5. Implement **Find & Replace**: Dedicated modal dialog (`Ctrl+F` / `Ctrl+H`) with target variable scoping, "Match entire cell contents" (safe default), "Match case", and optional Regular Expression (PCRE) support.
  6. Implement **Open Project File (`.rrad`)**: Complete workspace save/open (active data, variable dictionary, syntax log, and full output stream).
  7. Build the **Output Viewer**: a split-pane component with a left-hand collapsible Document Outline tree and a right-hand sandboxed HTML/SVG canvas (iframe or Shadow DOM with clean academic CSS styling for tables).
  8. Add export handlers on every output card (Copy HTML table to clipboard for Word/Excel, Save Plot as SVG/PNG, Export entire Output Log as standalone `.html`).
* **Verification Gate 2:**
  * Editing a variable label, name, or measurement level in "Variable View" immediately updates R's underlying `.active_data` and reflects in "Data View" column headers and tooltips.
  * Adding a blank column or formula-computed column updates both frontend grid and R session without desynchronization.
  * Modifying cells, inserting rows, and deleting rows synchronously mutates the R session dataset; Find & Replace correctly replaces occurrences across targeted or all columns and updates the grid.
  * Saving and reopening a `.rrad` project file fully restores the active dataset and all previous output cards (tables and SVG charts).
  * Importing and exporting `.csv`, `.sav`, and `.xlsx` files passes roundtrip verification.

### Phase 3: Dynamic Form Runtime & Code Generator

* **Tasks:**
  1. Implement the JSON Schema validator for `.rform.json`.
  2. Build the Dynamic Form Renderer modal that reads any `.rform.json` file, lays out the 12-column CSS grid, populates `VariableSourceList` from active dataset metadata, and enforces validation rules on `VariableTargetSlot`.
  3. Integrate the Handlebars template engine. Wire the three classic SPSS action buttons on every rendered dialog:
     * **[ OK ]:** Compiles the R template, sends `EXECUTE_ANALYSIS` to the R daemon, closes the dialog, and focuses the Output Viewer.
     * **[ Paste ]:** Compiles the R template and opens/appends it to the built-in R Syntax Editor tab without running it.
     * **[ Reset ]:** Restores all form widgets to their schema defaults.
* **Verification Gate 3:** Passing a mock `independent-t-test.rform.json` renders a functional dialog, blocks clicking **OK** until required slots are filled, and outputs both the R syntax and formatted result table to the Output Viewer.

### Phase 4: The Visual Form Builder (RAD Studio Mode)

* **Tasks:**
  1. Build the Form Designer IDE view consisting of four panes:
     * **Left Pane:** Widget Toolbox (`VariableSourceList`, `VariableTargetSlot`, inputs, sub-dialog buttons).
     * **Center Pane:** Interactive WYSIWYG Grid Canvas with snap-to-grid, resize handles, and tab-order overlays.
     * **Right Pane:** Property Inspector (edits `id`, `label`, accepted variable types, default values) & Menu Path selector (`Analyze -> Custom -> ...`).
     * **Bottom Pane:** R Code Template Editor with syntax highlighting and autocomplete for bound widget `{{id}}` tokens, plus a Live Preview toggle.
  2. Add a **"Customize Dialog (🔧)"** button to the title bar of every runtime dialog. Clicking it clones the dialog's `.rform.json` into the user's local workspace and opens it immediately in the Form Designer.
  3. Implement `form_registry.rs` to hot-reload the application's top menu bar whenever a user saves a new or modified `.rform.json` file.
* **Verification Gate 4:** An agent or user can visually create a new form from scratch in the Designer, bind two variable slots to an R `cor.test()` template, save it to `Analyze -> Correlate -> Custom Bivariate`, and immediately execute it from the main menu bar without restarting the app.

### Phase 5: The SPSS Emulation Catalog & Packaging

* **Tasks:**
  1. Author the standard library of `.rform.json` files inside `forms-catalog/` using exclusively the schema and widgets built in Phases 3–4:
     * **Descriptives:** Frequencies, Descriptives, Explore, Crosstabs (with $\chi^2$ and Fisher's Exact).
     * **Compare Means:** One-Sample T-Test, Independent-Samples T-Test, Paired-Samples T-Test, One-Way ANOVA (with Tukey/Bonferroni Post-Hoc sub-dialogs), Non-parametric equivalents (Mann-Whitney, Wilcoxon, Kruskal-Wallis).
     * **Regression & Correlation:** Bivariate Correlation, Linear Regression (with diagnostic plots sub-dialog), Binary Logistic Regression.
     * **Graphs:** Bar, Line, Boxplot, Scatterplot, and Histogram builders wrapping `ggplot2`.
  2. Configure platform-specific Tauri bundlers (`.AppImage` / `.deb` / `pacman` `PKGBUILD` for Linux, `.dmg` for macOS, `.msi` for Windows).
* **Verification Gate 5:** Full end-to-end automated test suite runs all shipped `.rform.json` dialogs against the standard `mtcars` and `iris` datasets, verifying zero R runtime errors and valid HTML/SVG output blocks for every menu item.

---

## 6. Strict Coding & Quality Guardrails for Agents

* **Zero UI Thread Blocking:** Never run synchronous R evaluations or large dataset parsing on the main Tauri thread or UI event loop. All R execution must be asynchronous with a visible "Running Analysis..." status indicator and an interrupt/cancel button (`SIGINT` to R child process).
* **Namespace Hygiene in R:** Generated R code templates MUST wrap intermediate variables in `local({ ... })` so user analyses do not pollute the global `.GlobalEnv` or overwrite `.active_data`.
* **Safe Variable Escaping:** Dataset column names with spaces or special characters must be automatically backtick-escaped (e.g., `` `Blood Pressure (Systolic)` ``) by the template interpolation engine before R execution.
* **Self-Dogfooding Rule:** If an SPSS dialog cannot be expressed using the current `.rform.json` capabilities during Phase 5, do not write a custom hardcoded component. Instead, escalate to `@RAD-Engine` to extend the declarative JSON schema and widget set so the Form Builder gains that capability for end-users as well.
