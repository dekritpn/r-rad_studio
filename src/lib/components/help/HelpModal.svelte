<!-- src/lib/components/help/HelpModal.svelte -->
<!-- Comprehensive In-App Documentation & About Dialog for R-RAD Studio -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';

  function close() {
    appState.closeHelp();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if appState.isHelpModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div
      class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col w-full max-w-4xl h-[85vh] overflow-hidden text-slate-100"
      role="dialog"
      aria-modal="true"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-slate-800 bg-slate-950 flex items-center justify-between select-none">
        <div class="flex items-center gap-3">
          <img src="/icon.png" alt="R-RAD Studio" class="w-7 h-7 rounded-lg shadow-sm object-contain" />
          <div>
            <h2 class="text-sm font-semibold text-white tracking-wide">Help & Reference</h2>
            <p class="text-[11px] text-slate-400">Documentation, Guides, and System Information</p>
          </div>
        </div>

        <!-- Navigation Tabs -->
        <div class="flex items-center bg-slate-900 border border-slate-800 rounded-lg p-0.5 text-xs">
          <button
            class="px-3 py-1.5 rounded-md font-medium transition-all {appState.helpModalTab === 'user' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => appState.helpModalTab = 'user'}
          >
            📖 User Guide
          </button>
          <button
            class="px-3 py-1.5 rounded-md font-medium transition-all {appState.helpModalTab === 'rad' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => appState.helpModalTab = 'rad'}
          >
            🛠️ RAD Developer Guide
          </button>
          <button
            class="px-3 py-1.5 rounded-md font-medium transition-all {appState.helpModalTab === 'about' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => appState.helpModalTab = 'about'}
          >
            ℹ️ About
          </button>
        </div>

        <!-- Close Button -->
        <button
          class="text-slate-400 hover:text-white p-1 rounded-md hover:bg-slate-800 transition-colors"
          onclick={close}
          title="Close (Esc)"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-6 text-sm text-slate-300 leading-relaxed space-y-6">
        {#if appState.helpModalTab === 'user'}
          <!-- USER GUIDE CONTENT -->
          <div class="space-y-6 max-w-3xl mx-auto">
            <div class="border-b border-slate-800 pb-4">
              <h1 class="text-xl font-bold text-white mb-2">User Guide: Getting Started with R-RAD Studio</h1>
              <p class="text-slate-400 text-xs">
                Learn how to open datasets, navigate the dual data grid, transform variables, run statistical analyses, and export publication-ready reports.
              </p>
            </div>

            <!-- Section 1 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">1. Data Import & File Formats</h2>
              <p>R-RAD Studio provides lossless support for key analytical formats via the <span class="text-white font-medium">File</span> menu:</p>
              <ul class="list-disc pl-5 space-y-1 text-xs">
                <li><strong class="text-slate-200">SPSS (.sav):</strong> Full metadata extraction including variable labels, measurement levels (Scale, Ordinal, Nominal), and value dictionaries.</li>
                <li><strong class="text-slate-200">Comma Separated Values (.csv):</strong> Standard delimited data.</li>
                <li><strong class="text-slate-200">Excel (.xlsx, .xls):</strong> Modern multi-sheet spreadsheets.</li>
                <li><strong class="text-slate-200">Sample Datasets:</strong> Quickly load <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">mtcars</code>, <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">iris</code>, or high-volume 50,000-row benchmarks directly from the menu.</li>
              </ul>
            </section>

            <!-- Section 2 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">2. The Dual Data Grid: Data View & Variable View</h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                <div class="bg-slate-950 p-3 rounded-lg border border-slate-800">
                  <h3 class="font-bold text-white mb-1">Data View</h3>
                  <p class="text-slate-400">Virtual view for raw data rows with paginated browsing. Measurement icons in column headers denote Scale (📐), Ordinal (📊), and Nominal (🔵).</p>
                </div>
                <div class="bg-slate-950 p-3 rounded-lg border border-slate-800">
                  <h3 class="font-bold text-white mb-1">Variable View</h3>
                  <p class="text-slate-400">Metadata dictionary defining variable properties. Click directly on any cell in the <strong class="text-slate-200">Name</strong> column to rename it with real-time R synchronization.</p>
                </div>
              </div>
            </section>

            <!-- Section 3 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">3. Data Wrangling & Transformations</h2>
              <p>Under the <span class="text-white font-medium">Data</span> menu:</p>
              <ul class="list-disc pl-5 space-y-1 text-xs">
                <li><strong class="text-slate-200">Insert Variable (Blank Column):</strong> Appends a new variable initialized with <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">NA</code> values.</li>
                <li><strong class="text-slate-200">Compute Variable...:</strong> Enter a target variable and an R expression (e.g. <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">log(mpg) * 2</code> or <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">ifelse(hp &gt; 150, 1, 0)</code>). Supports built-in statistical functions like <code class="text-indigo-300 font-mono">scale()</code>, <code class="text-indigo-300 font-mono">mean()</code>, and <code class="text-indigo-300 font-mono">round()</code>.</li>
              </ul>
            </section>

            <!-- Section 4 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">4. Running Statistical Analyses</h2>
              <p>Select any procedure from the <span class="text-white font-medium">Analyze</span> menu:</p>
              <ul class="list-disc pl-5 space-y-1 text-xs">
                <li><strong>Source Variables:</strong> Select variables on the left and use the <code class="bg-slate-800 px-1 rounded">&gt;</code> arrow to transfer them into target slots.</li>
                <li><strong>Target Slots:</strong> Enforce measurement rules and required fields before enabling action buttons.</li>
                <li><strong>[ OK ]:</strong> Executes analysis on the persistent R daemon and streams HTML tables and vector graphics to the Output Viewer.</li>
                <li><strong>[ Paste ]:</strong> Compiles the reproducible R syntax and appends it to the Syntax tab.</li>
                <li><strong>[ Reset ]:</strong> Restores all dialog controls to their default state.</li>
              </ul>
            </section>

            <!-- Section 5 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">5. Output Viewer & Open Projects (.rrad)</h2>
              <p>Review and export your analytical results:</p>
              <ul class="list-disc pl-5 space-y-1 text-xs">
                <li><strong>Document Outline:</strong> Live sidebar tree to jump directly to any analysis card.</li>
                <li><strong>Copy Table:</strong> Formats HTML tables for direct pasting into Microsoft Word, Excel, or Google Docs.</li>
                <li><strong>Save Project (.rrad):</strong> Preserves the active dataset, variable dictionary, syntax log, and full output stream into an open JSON file format.</li>
              </ul>
            </section>
          </div>

        {:else if appState.helpModalTab === 'rad'}
          <!-- RAD DEVELOPER GUIDE CONTENT -->
          <div class="space-y-6 max-w-3xl mx-auto">
            <div class="border-b border-slate-800 pb-4">
              <h1 class="text-xl font-bold text-white mb-2">RAD Developer Guide: Authoring Custom Modules</h1>
              <p class="text-slate-400 text-xs">
                Engineering guidelines for authoring <code class="text-amber-300 font-mono">.rform.json</code> modules either manually, in the visual Form Designer, or using an AI assistant.
              </p>
            </div>

            <!-- Core Tenet -->
            <div class="p-3 bg-indigo-950/40 border border-indigo-700/50 rounded-lg text-xs text-indigo-200">
              <strong>Core Architecture:</strong> 100% of all statistical dialogs in R-RAD Studio are declarative JSON form schemas. The <strong>Analyze</strong> menu automatically follows the folder hierarchy of the <code class="font-mono text-indigo-300">modules/</code> folder, where folder names automatically become menu category and submenu labels.
            </div>

            <!-- Section 1 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">1. Schema Structure Overview</h2>
              <pre class="bg-slate-950 p-3 rounded-lg border border-slate-800 font-mono text-[11px] text-slate-200 overflow-x-auto leading-relaxed">&#123;
  "$schema": "https://r-rad-studio.local/schemas/v1/rform.json",
  "id": "stat.compare_means.ind_t_test",
  "version": "1.0.0",
  "menu_path": ["Analyze", "Compare Means", "Independent-Samples T Test..."],
  "window": &#123; "title": "Independent-Samples T Test", "width": 640, "height": 450 &#125;,
  "layout": [ ... ],
  "sub_dialogs": [ ... ],
  "r_template": "local(&#123; ... &#125;)"
&#125;</pre>
              <p class="text-xs text-slate-400">
                <strong class="text-indigo-300">Folder Hierarchy Mapping:</strong> The <code class="text-white font-mono">Analyze</code> menu path is dynamically determined by the folder hierarchy inside <code class="text-indigo-300 font-mono">modules/</code>. Folder and subfolder names automatically become menu category and submenu labels.
              </p>
            </section>

            <!-- Section 2 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">2. The 12-Column Grid Layout Standard</h2>
              <p class="text-xs">
                All widgets specify <code class="bg-slate-800 px-1 py-0.5 rounded text-amber-300">grid: &#123; x, y, w, h &#125;</code> coordinates on a 12-column grid:
              </p>
              <div class="bg-slate-950 p-3 rounded-lg border border-slate-800 text-xs space-y-1 font-mono">
                <div>• Left Side (x: 1, w: 5): VariableSourceList (displays columns with measurement icons)</div>
                <div>• Right Side (x: 6, w: 7): VariableTargetSlot instances, Checkboxes, Dropdowns, SubDialogButtons</div>
              </div>
            </section>

            <!-- Section 3 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">3. R Template Guidelines & Output Helpers</h2>
              <ul class="list-disc pl-5 space-y-1 text-xs">
                <li><strong class="text-slate-200">Namespace Hygiene:</strong> Always wrap code templates in <code class="bg-slate-800 px-1 rounded text-amber-300">local(&#123; ... &#125;)</code> to keep <code class="text-indigo-300 font-mono">.GlobalEnv</code> clean.</li>
                <li><strong class="text-slate-200">rrad_emit_table(df, title):</strong> Emits a publication-ready HTML table styled with R's <code class="text-indigo-300 font-mono">gt</code> package.</li>
                <li><strong class="text-slate-200">rrad_emit_plot(p, title):</strong> Emits a scalable vector SVG chart rendered via <code class="text-indigo-300 font-mono">svglite</code>.</li>
                <li><strong class="text-slate-200">Backtick Escaping:</strong> Wrap variables in backticks (e.g. <code class="bg-slate-800 px-1 rounded text-amber-300">`&#123;&#123;dep_var&#125;&#125;`</code>) to safely handle spaces or special characters.</li>
              </ul>
            </section>

            <!-- Section 4 -->
            <section class="space-y-2">
              <h2 class="text-base font-semibold text-indigo-400">4. AI Prompting Template</h2>
              <p class="text-xs">Copy and paste this prompt to any LLM (Gemini, Claude, GPT-4) to generate a complete module:</p>
              <div class="bg-slate-950 p-3 rounded-lg border border-slate-800 text-xs font-mono text-slate-300 select-all leading-relaxed">
                "Author an R-RAD Studio statistical module for [PROCEDURE NAME]. Format as a valid .rform.json adhering to the schema: 12-column grid layout with VariableSourceList on left (w: 5), VariableTargetSlot on right, clean checkboxes/options, and an r_template wrapped in local(&#123;&#125;) using rrad_emit_table() and rrad_emit_plot(). Output raw JSON only."
              </div>
            </section>
          </div>

        {:else}
          <!-- ABOUT DIALOG CONTENT -->
          <div class="space-y-6 max-w-2xl mx-auto py-2">
            <div class="text-center space-y-2">
              <img src="/icon.png" alt="R-RAD Studio" class="w-16 h-16 rounded-2xl shadow-xl shadow-red-500/10 mx-auto mb-2 object-contain" />
              <h1 class="text-2xl font-bold text-white tracking-tight"><span class="text-red-500 font-mono font-black">R-RAD</span> Studio</h1>
              <p class="text-xs font-medium text-slate-400 font-mono">Version 0.1.0 • Release Preview</p>
              <p class="text-xs text-slate-400 max-w-md mx-auto">
                R Rapid Application Development & Modern Statistical GUI combining classic SPSS-style workflows with the reproducible power of R.
              </p>
            </div>

            <!-- Architecture Grid -->
            <div class="bg-slate-950 rounded-xl border border-slate-800 p-4 divide-y divide-slate-800/80 text-xs">
              <div class="py-2.5 flex items-center justify-between">
                <span class="text-slate-400 font-medium">Desktop Engine</span>
                <span class="text-slate-200 font-mono">Tauri v2 + Rust Core</span>
              </div>
              <div class="py-2.5 flex items-center justify-between">
                <span class="text-slate-400 font-medium">User Interface</span>
                <span class="text-slate-200 font-mono">Svelte 5 (Runes) + TailwindCSS</span>
              </div>
              <div class="py-2.5 flex items-center justify-between">
                <span class="text-slate-400 font-medium">Statistical Runtime</span>
                <span class="text-slate-200 font-mono">Persistent R Daemon (haven, broom, gt, ggplot2)</span>
              </div>
              <div class="py-2.5 flex items-center justify-between">
                <span class="text-slate-400 font-medium">Project Standard</span>
                <span class="text-slate-200 font-mono">Open JSON Schema (.rrad)</span>
              </div>
              <div class="py-2.5 flex items-center justify-between">
                <span class="text-slate-400 font-medium">Form Extensibility</span>
                <span class="text-slate-200 font-mono">Declarative .rform.json + Handlebars</span>
              </div>
            </div>

            <!-- License & Transparency -->
            <div class="text-center text-xs text-slate-500 space-y-1">
              <p>Designed for researchers, students, and analysts demanding open, reproducible science.</p>
              <p>Built with open-source technologies under permissive licensing.</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-slate-800 bg-slate-950 flex items-center justify-between text-xs text-slate-400">
        <div>
          {#if appState.helpModalTab === 'user'}
            <span>Full documentation saved to <code class="text-indigo-400 font-mono">docs/USER_GUIDE.md</code></span>
          {:else if appState.helpModalTab === 'rad'}
            <span>Full developer guide saved to <code class="text-indigo-400 font-mono">docs/RAD_GUIDE.md</code></span>
          {:else}
            <span>R-RAD Studio • Open Statistical Computing</span>
          {/if}
        </div>
        <button
          class="px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-sm"
          onclick={close}
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
