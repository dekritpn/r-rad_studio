# R-RAD Studio — RAD Module Developer & AI Authoring Guide

Welcome to the **R-RAD Studio RAD Developer Guide**. This document is the authoritative engineering specification for designing, authoring, and deploying custom statistical dialogs into R-RAD Studio.

Whether you are building modules manually using a code editor, using the visual in-app **RAD Designer**, or utilizing an **AI Assistant (LLM)** to generate statistical tools, this guide provides all visual design standards, JSON schema specifications, R templating rules, and file conventions.

---

## Table of Contents
1. [Core Architectural Philosophy](#1-core-architectural-philosophy)
2. [Directory Layout & File Conventions](#2-directory-layout--file-conventions)
3. [The `.rform.json` Schema Specification](#3-the-rformjson-schema-specification)
4. [Visual & Layout Guidelines (12-Column Grid)](#4-visual--layout-guidelines-12-column-grid)
5. [The Widget Library Reference](#5-the-widget-library-reference)
   - [VariableSourceList](#variablesourcelist)
   - [VariableTargetSlot](#variabletargetslot)
   - [Standard Controls (Checkbox, Dropdown, NumericInput, etc.)](#standard-controls)
   - [GroupBox](#groupbox)
   - [SubDialogButton & Child Dialogs](#subdialogbutton--child-dialogs)
6. [R Template Engine & Handlebars Rules](#6-r-template-engine--handlebars-rules)
   - [Namespace Isolation: `local({ ... })`](#namespace-isolation-local--)
   - [Output Protocol: Tables & SVG Plots](#output-protocol-tables--svg-plots)
   - [Escaping & Safe Variable Names](#escaping--safe-variable-names)
7. [Step-by-Step AI Prompting Template](#7-step-by-step-ai-prompting-template)
8. [Hot-Reloading & Live Testing](#8-hot-reloading--live-testing)

---

## 1. Core Architectural Philosophy

In R-RAD Studio:
> **"Everything is a Form Schema."**

100% of the statistical dialogs shipped with the application—from basic Frequencies to complex Logistic Regression models—are defined as declarative JSON files (`.rform.json`). 

- There are **no hardcoded statistical dialogs** in the application codebase.
- The Form Renderer dynamically loads any valid schema file, builds the reactive UI widgets, manages form validation, and interpolates values into an R script template.
- When you add or modify a `.rform.json` file in the forms catalog, the desktop application's top menu bar **hot-reloads instantly** without requiring recompilation or application restarts.

---

## 2. Directory Layout & File Conventions

All statistical forms reside within the `forms-catalog/` directory in the repository:

```plaintext
forms-catalog/
├── descriptives/
│   ├── frequencies.rform.json
│   └── crosstabs.rform.json
├── compare-means/
│   ├── independent-t-test.rform.json
│   └── one-way-anova.rform.json
├── regression/
│   ├── linear.rform.json
│   └── binary-logistic.rform.json
└── custom/                   # User-created or imported modules
    └── your-module.rform.json
```

### Naming Conventions:
- **Directory Names:** Use kebab-case categorizing the statistical family (e.g., `descriptives`, `non-parametric`, `survival`).
- **File Names:** Lowercase kebab-case matching the procedure with the extension `.rform.json` (e.g., `mann-whitney.rform.json`, `cox-regression.rform.json`).

---

## 3. The `.rform.json` Schema Specification

Every form file must be valid JSON conforming to the `rform.json` specification:

```json
{
  "$schema": "https://r-rad-studio.local/schemas/v1/rform.json",
  "id": "stat.compare_means.ind_t_test",
  "version": "1.0.0",
  "menu_path": ["Analyze", "Compare Means", "Independent-Samples T Test..."],
  "window": {
    "title": "Independent-Samples T Test",
    "width": 640,
    "height": 450
  },
  "layout": [
    ...
  ],
  "sub_dialogs": [
    ...
  ],
  "r_template": "local({ ... })"
}
```

### Top-Level Properties:
| Property | Type | Description |
| :--- | :--- | :--- |
| `id` | `string` | Unique reverse-DNS identifier (e.g., `stat.regression.linear`). |
| `version` | `string` | Semantic version string (e.g., `1.0.0`). |
| `menu_path` | `string[]` | Hierarchy in top menu bar. First element is usually `"Analyze"`. |
| `window` | `object` | Modal dimensions: `title`, `width` (pixels), and `height` (pixels). |
| `layout` | `Widget[]` | Array of UI widgets positioned on the 12-column grid. |
| `sub_dialogs` | `SubDialog[]` | Array of child modal dialog definitions (e.g., Options, Plots). |
| `r_template` | `string` | Handlebars-interpolated R code template to be executed. |

---

## 4. Visual & Layout Guidelines (12-Column Grid)

The Form Renderer positions widgets using a **12-column CSS Grid** coordinate system.

```plaintext
Col 1      Col 5   Col 6               Col 12
┌──────────────┐   ┌────────────────────────┐
│              │   │ [ Target Slot 1 ]      │
│  Variable    │   ├────────────────────────┤
│  Source      │ > │ [ Target Slot 2 ]      │
│  List        │   ├────────────────────────┤
│              │   │ [ GroupBox / Options ] │
└──────────────┘   └────────────────────────┘
```

### Grid Coordinates:
Each widget in `layout` has a `grid` object:
```json
"grid": {
  "x": 1,   // 1-based start column (1 to 12)
  "y": 1,   // 1-based start row
  "w": 5,   // column span width (1 to 12)
  "h": 10   // row span height
}
```

### Recommended Golden Layouts:
1. **Classic SPSS Split (Left / Right):**
   - **Left Column (`x: 1, w: 5`):** Reserved for `VariableSourceList`. Height typically `h: 8` to `h: 12`.
   - **Right Column (`x: 6, w: 7`):** Stack target slots, checkboxes, and sub-dialog option buttons.
2. **Standard Window Dimensions:**
   - Simple Dialog (Frequencies, Descriptives): `width: 580`, `height: 420`
   - Medium Dual-Slot (T-Test, Crosstabs): `width: 640`, `height: 460`
   - Complex Multi-Block (Linear Regression, ANOVA): `width: 720`, `height: 520`

---

## 5. The Widget Library Reference

### VariableSourceList
Pulls available columns from the active dataset and displays them with measurement icons:
```json
{
  "id": "src_vars",
  "type": "VariableSourceList",
  "grid": { "x": 1, "y": 1, "w": 5, "h": 10 },
  "filter_types": ["scale", "ordinal", "nominal"]
}
```
- `filter_types`: Array limiting which column types to show. Omit or pass `["scale", "ordinal", "nominal"]` to show all.

### VariableTargetSlot
Receives variables transferred from the source list. Validates data types and enforces selection requirements:
```json
{
  "id": "dep_var",
  "type": "VariableTargetSlot",
  "label": "Dependent Variable:",
  "grid": { "x": 6, "y": 1, "w": 6, "h": 3 },
  "accepts": {
    "types": ["scale"],
    "multiple": false,
    "required": true
  }
}
```
- `accepts.types`: Allowed measurement types (`["scale"]`, `["nominal", "ordinal"]`, etc.).
- `accepts.multiple`: `true` allows multiple variables; `false` accepts only one.
- `accepts.required`: If `true`, the form blocks clicking **[ OK ]** until filled.

### Standard Controls

#### Checkbox
```json
{
  "id": "chk_posthoc",
  "type": "Checkbox",
  "label": "Calculate Tukey HSD Post-Hoc Test",
  "grid": { "x": 6, "y": 7, "w": 6, "h": 1 },
  "default": true
}
```

#### Dropdown
```json
{
  "id": "method_select",
  "type": "Dropdown",
  "label": "Estimation Method:",
  "grid": { "x": 6, "y": 8, "w": 6, "h": 1 },
  "options": [
    { "label": "Pearson Correlation", "value": "pearson" },
    { "label": "Spearman Rank", "value": "spearman" },
    { "label": "Kendall Tau", "value": "kendall" }
  ],
  "default": "pearson"
}
```

#### NumericInput & TextInput
```json
{
  "id": "ci_level",
  "type": "NumericInput",
  "label": "Confidence Level (%):",
  "grid": { "x": 6, "y": 9, "w": 6, "h": 1 },
  "default": 95,
  "min": 50,
  "max": 99.9,
  "step": 1
}
```

### GroupBox
Groups related controls visually with a bordered title container:
```json
{
  "id": "grp_statistics",
  "type": "GroupBox",
  "label": "Display Options",
  "grid": { "x": 6, "y": 5, "w": 6, "h": 3 },
  "children": [ ... ]
}
```

### SubDialogButton & Child Dialogs
Triggers a modal secondary dialog (e.g. for Options, Plots, or Bootstrapping):
```json
{
  "id": "btn_options",
  "type": "SubDialogButton",
  "label": "Options...",
  "grid": { "x": 10, "y": 10, "w": 2, "h": 1 },
  "sub_dialog_id": "dlg_options"
}
```
Define the matching child modal in `sub_dialogs`:
```json
"sub_dialogs": [
  {
    "id": "dlg_options",
    "label": "Independent T-Test: Options",
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
]
```
In your R template, values from sub-dialogs are accessed using dot notation:
`{{sub_dialogs.dlg_options.conf_level}}`

---

## 6. R Template Engine & Handlebars Rules

R-RAD Studio compiles the `r_template` string using the Handlebars templating engine before dispatching the payload to the persistent R session.

### Namespace Isolation: `local({ ... })`
> **CRITICAL RULE:** All generated R code MUST be wrapped inside `local({ ... })`.

Wrapping prevents temporary analysis variables (e.g., `model`, `res`, `fit`) from polluting R's global workspace or corrupting `.active_data`.

```handlebars
local({
  fit <- lm({{dep_var}} ~ {{#each indep_vars}}{{#if @first}}{{this}}{{else}} + {{this}}{{/if}}{{/each}}, data = .active_data)
  rrad_emit_table(broom::tidy(fit), title = "Model Coefficients")
})
```

### Output Protocol: Tables & SVG Plots
The R runtime environment exposes two primary helper functions to deliver structured outputs to the user's Output Viewer:

1. **`rrad_emit_table(x, title = NULL)`**
   - Accepts a data frame, matrix, or `broom::tidy()` object.
   - Formats the table using R's `gt` package into a clean, publication-quality HTML card.
2. **`rrad_emit_plot(p = last_plot(), title = NULL, width = 7, height = 4.5)`**
   - Captures any `ggplot2` object or base R graphic and renders it as a crisp, scalable SVG vector graphic via `svglite`.

### Escaping & Safe Variable Names
Column names may contain spaces or reserved symbols. When interpolating variable names in templates, wrap them in backticks if they are passed as raw symbols:
```handlebars
t.test(`{{dep_var}}` ~ `{{group_var}}`, data = .active_data)
```

For iterating through multiple variables:
```handlebars
{{#each test_vars}}
local({
  res <- t.test(`{{this}}` ~ `{{../group_var}}`, data = .active_data)
  rrad_emit_table(broom::tidy(res), title = paste("T-Test for", "{{this}}"))
})
{{/each}}
```

---

## 7. Step-by-Step AI Prompting Template

You can instruct any modern Large Language Model (such as Gemini, Claude, or GPT-4) to generate fully functional `.rform.json` modules for R-RAD Studio.

### Standard AI Prompt:
```markdown
Please author an R-RAD Studio statistical module for [INSERT STATISTICAL PROCEDURE, e.g., Paired Samples T-Test / Chi-Square Test of Independence].

The module must be formatted as a single, valid `.rform.json` file adhering to the R-RAD Studio specification:
1. `id`: "stat.[category].[name]"
2. `menu_path`: ["Analyze", "[Category Name]", "[Item Name]..."]
3. `window`: Width between 600-720, Height between 440-520
4. `layout`:
   - Left side (x: 1, w: 5): `VariableSourceList`
   - Right side (x: 6, w: 6 or 7): `VariableTargetSlot` instances with clear labels, measurement level filtering (`accepts.types`), and `required: true` where necessary.
   - Options, checkboxes, or dropdowns positioned cleanly without overlap.
5. `r_template`:
   - Wrapped completely in `local({ ... })`
   - Accesses data from `.active_data`
   - Emits results using `rrad_emit_table()` (wrapping `broom::tidy()` or standard summaries) and optionally `rrad_emit_plot()`
   - Properly handles Handlebars loops for multi-variable inputs with backtick escaping.

Provide only the valid raw JSON object.
```

---

## 8. Hot-Reloading & Live Testing

Once your `.rform.json` file is written:
1. Save the file into `forms-catalog/[category]/[name].rform.json` (or your user custom directory).
2. Look at R-RAD Studio: the top menu bar automatically hot-reloads and mounts your new entry in the menu path defined by `menu_path`.
3. Click your menu item to open the dialog.
4. Test with a sample dataset (e.g. `mtcars` or `iris`):
   - Select variables and check validation.
   - Click **[ Paste ]** to verify the Handlebars-interpolated R syntax in the Syntax tab.
   - Click **[ OK ]** to verify table and plot rendering in the Output Viewer.

If you ever need to inspect or modify an existing dialog visually, click the **"Customize Dialog (🔧)"** button on the dialog title bar to open it directly in the in-app **RAD Designer**.
