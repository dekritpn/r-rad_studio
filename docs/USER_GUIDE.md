# R-RAD Studio User Guide

Welcome to **R-RAD Studio** (R Rapid Application Development & Statistical GUI). R-RAD Studio bridges the familiar, intuitive graphical interface of traditional statistical packages (like IBM SPSS Statistics) with the modern power, flexibility, and reproducibility of the R statistical computing engine.

---

## Table of Contents
1. [System Overview & Architecture](#1-system-overview--architecture)
2. [Getting Started & Opening Data](#2-getting-started--opening-data)
3. [The Dual Data Grid & Spreadsheet Editing](#3-the-dual-data-grid--spreadsheet-editing)
   - [Data View](#data-view)
   - [Variable View](#variable-view)
   - [Renaming Variables & Blank Columns](#renaming-variables--blank-columns)
   - [Spreadsheet Editing, Cut, Copy, Paste & Search](#spreadsheet-editing-cut-copy-paste--search)
4. [Forgiving Undo/Redo & Data Safety Architecture](#4-forgiving-undoredo--data-safety-architecture)
   - [Multi-Level Undo & Redo](#multi-level-undo--redo)
   - [Hardware & Software Crash Resilience](#hardware--software-crash-resilience)
   - [Emergency Checkpoint Recovery](#emergency-checkpoint-recovery)
5. [Data Transformations & Formulas](#5-data-transformations--formulas)
6. [Running Statistical Analyses](#6-running-statistical-analyses)
   - [Dialog Navigation & Selection](#dialog-navigation--selection)
   - [The Action Buttons: OK, Paste, Reset](#the-action-buttons-ok-paste-reset)
7. [The Interactive Output Viewer](#7-the-interactive-output-viewer)
8. [The R Syntax Editor](#8-the-r-syntax-editor)
9. [Project Management (.rrad) & Interoperability](#9-project-management-rrad--interoperability)

---

## 1. System Overview & Architecture

R-RAD Studio is built upon four foundational pillars:
- **Tauri v2 + Rust Core:** Delivers a lightweight desktop experience with native OS dialogs and high-performance asynchronous multi-threaded data handling.
- **Svelte 5 Reactive UI:** Provides an ultra-responsive interface using Runes for zero-lag data grid navigation and dialog interaction.
- **Persistent R Runtime:** A dedicated background R session maintains your dataset (`.active_data`) in memory, avoiding startup overhead and preserving your statistical state.
- **Open Project Specification (`.rrad`):** Stores all data rows, metadata dictionary, syntax history, and rendered output blocks in a single, transparent, human-readable JSON format.

---

## 2. Getting Started & Opening Data

When you launch R-RAD Studio, the application automatically initializes the background R daemon. The status indicator in the top-right corner displays `R Ready` (green dot) when the system is initialized.

### Supported File Formats
You can open, import, and export datasets across all major analytical formats via the **File** menu:
- **SPSS Dataset (`.sav`):** Lossless import and export using `haven`, including variable labels, measurement levels, and value label dictionaries.
- **OpenDocument Spreadsheet (`.ods`):** Open standard spreadsheets parsed and exported via `readODS`.
- **Comma Separated Values (`.csv`):** Standard delimited text tables parsed via `read.csv`.
- **Microsoft Excel (`.xlsx`, `.xls`):** Multi-column spreadsheets parsed via `readxl` and exported via `writexl`.
- **R Data Files (`.rds`):** Native R serialized data frames.

### Opening Datasets
1. Click **File $\rightarrow$ Open / Import Dataset...** in the top menu bar.
2. Choose your `.sav`, `.ods`, `.csv`, `.xlsx`, or `.rds` file in the native file picker.
3. The dataset will load into R's `.active_data` environment and immediately display in the **Dual Data Grid**.

### Built-in Sample Datasets
To quickly explore the application without external files, select any of the pre-loaded datasets under **File $\rightarrow$ Sample Datasets**:
- **Motor Trend Cars (`mtcars`):** 32 observations of 11 automobile design and performance attributes.
- **Fisher's Iris (`iris`):** 150 observations across 3 flower species with sepal and petal dimensions.
- **50,000 Rows Benchmark (`test_50k.sav`):** A high-volume synthetic dataset to demonstrate virtualized grid rendering speed.

---

## 3. The Dual Data Grid

Just like classic analytical suites, R-RAD Studio features a synchronized **Dual Data Grid** with two views accessible via the tabs in the bottom-left corner or the **View** menu:

### Data View
- Displays your raw rectangular data records.
- **Pagination & Virtualization:** Navigate effortlessly through datasets of tens of thousands of rows using the pagination controls in the bottom bar (e.g., jump between pages of 50 rows).
- **Column Header Icons:** Column headers display icons matching variable measurement levels:
  - 📐 **Scale / Continuous:** Numeric measurements (e.g., weight, height, speed).
  - 📊 **Ordinal:** Ordered categories (e.g., Likert scales, education tiers).
  - 🔵 **Nominal:** Categorical identifiers or factors (e.g., gender, treatment group).

### Variable View
- Displays the metadata dictionary for every variable in the active dataset.
- Columns include:
  - **Name:** The internal R variable name (must be unique, alphanumeric).
  - **Type:** Data type (`numeric`, `character`, `factor`, `date`).
  - **Width & Decimals:** Display format specifications.
  - **Label:** Human-readable descriptive title for graphs and tables.
  - **Values:** Categorical value labels (e.g., `1 = Male, 2 = Female`).
  - **Missing:** Missing value codes (e.g., `99 = Refused`).
  - **Measure:** Measurement level (`Scale`, `Ordinal`, `Nominal`).

### Renaming Variables
To rename any variable:
1. Switch to **Variable View**.
2. Click directly on the variable's **Name** cell.
3. Type the new variable name (e.g., `vehicle_weight`).
4. Press **Enter** or click outside the cell.
5. The name will be validated and immediately synchronized with R's `.active_data` environment. The updated name instantly reflects in the **Data View** headers.

### Renaming Variables & Blank Columns
- **Renaming Variables:** In **Variable View**, click directly on the variable's **Name** cell, type the new identifier, and press **Enter**. The updated column name is verified and instantly reflected across all views and dialogs.
- **Inserting Blank Columns:** Select **Edit $\rightarrow$ Insert Variable (Blank Column)...** or click the **`[ ➕ Add Variable ]`** button in the Variable View bottom bar. A new variable (e.g., `VAR00001`) is created and initialized with missing values (`NA`).

### Spreadsheet Editing, Cut, Copy, Paste & Search
- **Inline Cell Editing:** Double-click any data cell in **Data View** to edit its value directly. Press **Enter** to save, **Tab** to edit the next cell, or **Escape** to cancel.
- **Inserting & Deleting Rows:** Select **Edit $\rightarrow$ Insert Row (Case)** or click **➕ Insert Row** on the toolbar. To delete a case, select its row number and click **🗑️ Delete Selected Row** or press the **Delete** key.
- **Copy & Paste (Excel / TSV Interoperability):** Copy any cell or row to clipboard with **Ctrl+C**. Paste single values or multi-row rectangular data copied directly from Excel or Google Sheets with **Ctrl+V** or the **📥 Paste** button.
- **Find & Replace:** Access the quick search dialog via **Ctrl+F** (Find) or **Ctrl+H** (Replace), supporting column scoping, match whole cell, case sensitivity, and Perl-compatible regular expressions.

---

## 4. Forgiving Undo/Redo & Data Safety Architecture

Data integrity and safety are paramount in R-RAD Studio. Users can perform large-scale data modifications, bulk pastes, and variable computations with complete peace of mind.

### Multi-Level Undo & Redo
- **30-Level Deep Snapshot History:** R-RAD Studio preserves an in-memory chronological stack of up to 30 snapshot states of the active dataset.
- **Forgiving Reversibility:** Any major mutation—including formula computations, batch pastes, row deletions, column renaming, and cell replacements—can be undone effortlessly without data loss.
- **Keyboard Shortcuts:**
  - **Undo:** Press `Ctrl+Z` (or click `↶ Undo` on the toolbar / Edit menu).
  - **Redo:** Press `Ctrl+Y` or `Ctrl+Shift+Z` (or click `↷ Redo`).
- **Memory-Efficient Copy-on-Write:** Built on R's native memory architecture, undo snapshots share unedited columns and incur virtually zero memory overhead.

### Hardware & Software Crash Resilience
- **Automated Atomic Disk Checkpoint:** After every data mutation, R-RAD Studio saves a durable snapshot to disk (`~/.config/r-rad-studio/autosave/checkpoint.rds`).
- **Atomic POSIX File Write:** Checkpoints are written to a temporary file (`.tmp.rds`) before being atomically renamed into place. Even in the event of an abrupt power failure, OS freeze, or process termination, your checkpoint file cannot be corrupted.
- **100% Safe Rust Guarantee:** The underlying desktop bridge operates under strict 100% safe Rust guarantees with zero `unsafe` blocks, eliminating memory unsafety risks.

### Emergency Checkpoint Recovery
- If an unexpected hardware failure or operating system shutdown occurs, select **Edit $\rightarrow$ Restore from Emergency Checkpoint...**.
- R-RAD Studio inspects the timestamped checkpoint on disk and restores your dataset to its exact state prior to the interruption.

---

## 5. Data Transformations & Formulas

To derive new variables or compute statistical transformations, use the **Compute Variable** dialog:

1. Select **Edit $\rightarrow$ Compute Variable...** from the menu bar.
2. In the modal dialog:
   - **Target Variable:** Enter the name of the new or existing variable to compute (e.g., `bmi` or `log_mpg`).
   - **Numeric Expression:** Enter an R mathematical or logical formula. You can use standard arithmetic operators (`+`, `-`, `*`, `/`, `^`), parenthetical grouping, or logical conditions.
   - **Variable List:** Double-click or select a variable from the source list and click **[ ⬅ ]** to insert its escaped name into your formula.
   - **Statistical Functions:** Click any function button in the right palette to insert common transforms:
     - `log()` / `exp()`: Natural logarithm and exponential.
     - `sqrt()` / `abs()`: Square root and absolute value.
     - `mean()` / `sd()`: Column mean and standard deviation.
     - `scale()`: Z-score standardization.
     - `round()`: Numerical rounding.
     - `ifelse()`: Conditional logic (e.g., `ifelse(mpg > 20, "High", "Low")`).
3. Click **[ OK ]** to calculate the new variable immediately. The result is appended to `.active_data`, an undo checkpoint is registered, a summary table is logged to the Output Viewer, and the Data View refreshes.
4. Alternatively, click **[ Paste ]** to generate the reproducible R syntax in your Syntax Editor tab.

---

## 5. Running Statistical Analyses

R-RAD Studio ships with an extensible catalog of classic statistical procedures under the **Analyze** menu.

> **Dynamic Folder Organization:** The **Analyze** menu dynamically reflects the exact folder hierarchy in your `modules/` directory. Folder names automatically become the menu category labels, allowing you to organize, categorize, and nest your analytical modules however you prefer.

- **Descriptives:**
  - *Frequencies:* Categorical counts, percentages, cumulative stats, and bar charts.
  - *Crosstabs:* 2-way contingency tables with Pearson $\chi^2$ and Fisher's Exact tests.
- **Compare Means:**
  - *Independent-Samples T Test:* Student's and Welch's t-test with confidence intervals.
  - *One-Way ANOVA:* Analysis of variance with post-hoc tests (Tukey HSD, Bonferroni).
- **Regression:**
  - *Linear Regression:* Ordinary Least Squares (OLS) with model summaries, coefficients, ANOVA table, and diagnostic residual plots.
  - *Binary Logistic Regression:* Generalized linear modeling for dichotomous outcomes.
- **Graphs:**
  - *Histogram & Scatterplot:* Scalable ggplot2 vector plots.

### Dialog Navigation & Selection
Every statistical dialog is rendered dynamically from an open schema (`.rform.json`):
1. **Source Variable List (Left):** Displays variables filtered by allowed measurement levels.
2. **Transfer Buttons (`>`):** Move selected variables into specific target slots.
3. **Target Slots (Right):** Accepts single or multiple variables according to statistical constraints.
4. **Options Sub-Dialogs:** Click buttons like `Options...` or `Plots...` to open modal secondary dialogs (e.g., setting confidence interval percentages or requesting histograms).

### The Action Buttons
- **[ OK ]:** Validates form inputs, compiles the R template, executes the analysis on the R daemon, appends the output to the Output Viewer, and closes the modal.
- **[ Paste ]:** Compiles the exact R code template and appends it directly to the **Syntax Editor** tab, allowing you to customize, batch, or save scripts.
- **[ Reset ]:** Restores all controls, checkboxes, and target slots in the dialog to their default blank state.
- **[ Cancel ]:** Dismisses the dialog without executing or pasting code.

---

## 6. The Interactive Output Viewer

Access the Output Viewer anytime by clicking the **Output** tab in the bottom bar or the **View $\rightarrow$ Output Viewer** menu.

### Key Output Features
- **Collapsible Document Outline (Left Pane):** Displays a hierarchical tree of all analyses performed during the session. Click any node to instantly jump to that result card.
- **Rich Result Cards (Right Pane):**
  - **Syntax Block:** Shows the exact, reproducible R code executed for the analysis.
  - **Publication-Ready Tables:** Rendered via R's `gt` engine with academic APA-style borders, clean typography, and formatted decimal places.
  - **High-Resolution Vector Graphics:** Statistical charts rendered as scalable SVGs (`svglite` / `ggplot2`).
- **Export & Action Tools:**
  - **📋 Copy Table:** Copies table HTML formatted for direct pasting into Microsoft Word or Excel.
  - **💾 Export HTML Log:** Exports the entire session notebook as a self-contained, shareable `.html` report.
  - **🗑️ Clear Log:** Empties the current output stream.

---

## 7. The R Syntax Editor

For users who want script-level control alongside the GUI:
- Click the **Syntax** tab to view the code editor.
- Code generated from the **[ Paste ]** button in any statistical dialog appears here.
- You can write arbitrary R commands, highlight sections, and click **Run** to execute them against the active dataset.
- All code executed from the editor produces corresponding output cards in the Output Viewer.

---

## 8. Project Management (.rrad) & Interoperability

R-RAD Studio introduces an open project file standard (`.rrad`) designed for total reproducibility and transparency:

### Saving a Project
Click **File $\rightarrow$ Save Project (.rrad)...**:
- Saves the entire workspace into a human-readable JSON file.
- Preserves:
  1. The complete active dataset (all rows and typed columns).
  2. The variable dictionary (labels, measurement levels, value mappings).
  3. The syntax history of all executed analyses.
  4. The complete output log (HTML tables, SVG plots, and syntax blocks).

### Opening a Project
Click **File $\rightarrow$ Open Project (.rrad)...**:
- Select any `.rrad` or `.rrad.json` file.
- The dataset is immediately reconstructed in R's `.active_data`, the data grid is populated, the syntax history is restored, and all historical output cards reappear in the Output Viewer.

---

*Need to create your own custom statistical dialogs or modify existing menus? Consult the **RAD Developer Guide** accessible via `Help -> RAD Developer Guide` or `docs/RAD_GUIDE.md`.*
