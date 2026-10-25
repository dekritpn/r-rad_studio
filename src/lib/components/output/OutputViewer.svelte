<!-- src/lib/components/output/OutputViewer.svelte -->
<!-- Browser Output Stream & Document Outline Navigation Tree -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import type { OutputBlock, OutputLogItem } from '../../schemas/output';

  let selectedItemId = $state<string | null>(null);
  let copyFeedback = $state<string | null>(null);

  // Sync selected item
  $effect(() => {
    if (appState.outputLogs.length > 0 && !selectedItemId) {
      selectedItemId = appState.outputLogs[appState.outputLogs.length - 1].id;
    }
  });

  function scrollToBlock(id: string) {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  async function copyHtmlToClipboard(html: string) {
    try {
      const blob = new Blob([html], { type: 'text/html' });
      const textBlob = new Blob([html.replace(/<[^>]+>/g, '')], { type: 'text/plain' });
      await navigator.clipboard.write([
        new ClipboardItem({
          'text/html': blob,
          'text/plain': textBlob,
        }),
      ]);
      showToast('Table copied to clipboard (Word/Excel compatible)');
    } catch {
      // Fallback
      await navigator.clipboard.writeText(html);
      showToast('HTML code copied to clipboard');
    }
  }

  async function copyTextToClipboard(text: string) {
    await navigator.clipboard.writeText(text);
    showToast('Copied to clipboard');
  }

  function saveSvgAsFile(svgContent: string, filename: string = 'plot.svg') {
    const blob = new Blob([svgContent], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    showToast('Plot downloaded as SVG');
  }

  function exportFullHtmlReport() {
    if (appState.outputLogs.length === 0) return;
    const reportHtml = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>R-RAD Studio Output Report</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; margin: 40px auto; max-width: 900px; line-height: 1.5; color: #1e293b; }
    h1 { border-bottom: 2px solid #0f172a; padding-bottom: 8px; font-size: 24px; }
    .report-card { border: 1px solid #e2e8f0; border-radius: 8px; padding: 24px; margin-bottom: 32px; box-shadow: 0 1px 3px rgba(0,0,0,0.05); }
    .syntax-pre { background: #0f172a; color: #f8fafc; padding: 12px; border-radius: 6px; font-family: monospace; font-size: 13px; overflow-x: auto; }
    .rrad-table { border-collapse: collapse; width: 100%; margin: 16px 0; font-size: 13px; font-variant-numeric: tabular-nums; }
    .rrad-table th { border-top: 2px solid #0f172a; border-bottom: 1px solid #0f172a; padding: 6px 12px; text-align: left; }
    .rrad-table td { padding: 6px 12px; border-bottom: 1px solid #e2e8f0; }
    .rrad-table tr:last-child td { border-bottom: 2px solid #0f172a; }
    .rrad-table-title { font-weight: bold; font-size: 14px; margin-bottom: 4px; }
    .svg-container svg { max-width: 100%; height: auto; }
  </style>
</head>
<body>
  <h1>R-RAD Studio Analysis Output</h1>
  <p style="color: #64748b; font-size: 13px;">Generated on: ${new Date().toLocaleString()}</p>
  ${appState.outputLogs.map(log => `
    <div class="report-card">
      <h2 style="font-size: 18px; margin-top: 0;">${log.title}</h2>
      ${log.blocks.map(b => {
        if (b.type === 'syntax') return `<pre class="syntax-pre"><code>${b.content || ''}</code></pre>`;
        if (b.type === 'html_table') return b.html || '';
        if (b.type === 'svg_plot') return `<div class="svg-container">${b.svg || ''}</div>`;
        if (b.type === 'warning') return `<div style="background:#fffbeb; color:#92400e; padding:10px; border-left:4px solid #f59e0b; margin:8px 0;">Warning: ${b.content}</div>`;
        if (b.type === 'error') return `<div style="background:#fef2f2; color:#b91c1c; padding:10px; border-left:4px solid #ef4444; margin:8px 0;">Error: ${b.content}</div>`;
        return `<p>${b.content || ''}</p>`;
      }).join('\n')}
    </div>
  `).join('\n')}
</body>
</html>`;

    const blob = new Blob([reportHtml], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `r_rad_output_${Date.now()}.html`;
    a.click();
    URL.revokeObjectURL(url);
    showToast('Report exported as standalone HTML file');
  }

  function showToast(msg: string) {
    copyFeedback = msg;
    setTimeout(() => {
      copyFeedback = null;
    }, 2500);
  }

  function clearAllLogs() {
    appState.outputLogs = [];
    selectedItemId = null;
  }
</script>

<div class="flex flex-col h-full bg-slate-950 text-slate-100 select-none font-output-custom">
  <!-- Top Output Toolbar -->
  <div class="h-10 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 z-10">
    <div class="flex items-center gap-2">
      <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <span class="text-xs font-semibold text-slate-200">Output Viewer</span>
      <span class="text-[11px] text-slate-400 bg-slate-800 px-2 py-0.5 rounded-full font-mono">
        {appState.outputLogs.length} logs
      </span>
    </div>

    <div class="flex items-center gap-2">
      <button
        class="px-2.5 py-1 text-xs rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors flex items-center gap-1.5 disabled:opacity-40"
        onclick={exportFullHtmlReport}
        disabled={appState.outputLogs.length === 0}
        title="Export full output log as standalone HTML"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        Export HTML
      </button>

      <button
        class="px-2.5 py-1 text-xs rounded bg-rose-950/40 border border-rose-800/40 hover:bg-rose-900/50 text-rose-300 transition-colors flex items-center gap-1 disabled:opacity-40"
        onclick={clearAllLogs}
        disabled={appState.outputLogs.length === 0}
        title="Clear all output history"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        Clear Log
      </button>
    </div>
  </div>

  <!-- Toast Notification -->
  {#if copyFeedback}
    <div class="absolute top-12 right-6 bg-indigo-600 text-white px-3.5 py-1.5 rounded-lg text-xs font-medium shadow-lg z-50 animate-bounce">
      {copyFeedback}
    </div>
  {/if}

  <!-- Split Pane Main Body -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Left Pane: Collapsible Document Outline Tree (SPSS Style) -->
    <div class="w-64 bg-slate-900 border-r border-slate-800 flex flex-col overflow-y-auto">
      <div class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wider text-slate-400 bg-slate-950/60 border-b border-slate-800/60">
        Document Outline
      </div>

      {#if appState.outputLogs.length === 0}
        <div class="p-4 text-xs text-slate-500 italic">No output items</div>
      {:else}
        <div class="p-2 space-y-1">
          {#each appState.outputLogs as log}
            <div class="rounded overflow-hidden">
              <!-- Log Header Node -->
              <div
                class="px-2.5 py-1.5 text-xs rounded font-medium flex items-center justify-between cursor-pointer transition-colors {selectedItemId === log.id ? 'bg-indigo-950/60 text-indigo-300 font-semibold' : 'text-slate-300 hover:bg-slate-800'}"
                onclick={() => { selectedItemId = log.id; scrollToBlock(log.id); }}
              >
                <div class="flex items-center gap-1.5 truncate">
                  <svg class="w-3.5 h-3.5 text-slate-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                  </svg>
                  <span class="truncate">{log.title}</span>
                </div>
                <span class="text-[10px] text-slate-500 shrink-0 font-mono">{log.timestamp}</span>
              </div>

              <!-- Nested Child Blocks -->
              <div class="pl-5 pr-1 py-0.5 space-y-0.5 border-l border-slate-800 ml-3 mt-0.5">
                {#each log.blocks as b, bIdx}
                  <div
                    class="px-2 py-0.5 text-[11px] text-slate-400 hover:text-slate-100 hover:bg-slate-800/50 rounded cursor-pointer truncate flex items-center gap-1.5 transition-colors"
                    onclick={() => scrollToBlock(`${log.id}_block_${bIdx}`)}
                  >
                    {#if b.type === 'syntax'}
                      <span class="text-blue-400 font-mono text-[9px] font-bold">R</span>
                      <span class="truncate">Syntax</span>
                    {:else if b.type === 'html_table'}
                      <span class="text-emerald-400">📊</span>
                      <span class="truncate">{b.title || 'Table'}</span>
                    {:else if b.type === 'svg_plot'}
                      <span class="text-purple-400">📈</span>
                      <span class="truncate">{b.title || 'Plot'}</span>
                    {:else if b.type === 'warning'}
                      <span class="text-amber-400">⚠️</span>
                      <span class="truncate">Warning</span>
                    {:else if b.type === 'error'}
                      <span class="text-rose-400">❌</span>
                      <span class="truncate">Error</span>
                    {:else}
                      <span class="text-slate-400">📄</span>
                      <span class="truncate">Log</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Right Pane: Interactive HTML/SVG Output Canvas -->
    <div class="flex-1 overflow-y-auto p-6 space-y-6 bg-slate-950">
      {#if appState.outputLogs.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-slate-500">
          <svg class="w-16 h-16 mb-4 text-slate-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-base font-medium text-slate-400">Ready for Statistical Analysis</p>
          <p class="text-xs text-slate-600 mt-1">Select an analysis from the Analyze menu to produce tables and charts</p>
        </div>
      {:else}
        {#each appState.outputLogs as log}
          <div
            id={log.id}
            class="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-lg transition-all"
          >
            <!-- Card Header -->
            <div class="bg-slate-900 px-5 py-3 border-b border-slate-800 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="w-2.5 h-2.5 rounded-full bg-indigo-500"></span>
                <h3 class="text-sm font-bold text-white tracking-wide">{log.title}</h3>
              </div>
              <span class="text-xs text-slate-400 font-mono">{log.timestamp}</span>
            </div>

            <!-- Card Blocks -->
            <div class="p-5 space-y-5">
              {#each log.blocks as b, bIdx}
                <div id={`${log.id}_block_${bIdx}`} class="relative group">
                  <!-- SYNTAX BLOCK -->
                  {#if b.type === 'syntax'}
                    <div class="relative bg-slate-950 border border-slate-800 rounded-lg p-3 font-mono text-xs text-emerald-400 overflow-x-auto">
                      <div class="absolute top-2 right-2 flex items-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
                        <button
                          class="px-2 py-0.5 text-[10px] bg-slate-800 hover:bg-slate-700 text-slate-300 rounded"
                          onclick={() => copyTextToClipboard(b.content || '')}
                        >
                          Copy Syntax
                        </button>
                      </div>
                      <pre><code>{b.content}</code></pre>
                    </div>

                  <!-- HTML TABLE BLOCK -->
                  {:else if b.type === 'html_table'}
                    <div class="bg-white text-slate-900 rounded-lg p-5 shadow-sm overflow-x-auto relative">
                      <div class="absolute top-3 right-3 opacity-90 group-hover:opacity-100 transition-opacity flex items-center gap-2">
                        <button
                          class="px-2.5 py-1 text-xs bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-300 rounded font-medium shadow-sm transition-colors"
                          onclick={() => copyHtmlToClipboard(b.html || '')}
                          title="Copy table to paste into Word or Excel"
                        >
                          📋 Copy Table
                        </button>
                      </div>
                      <!-- Raw HTML rendered table -->
                      <div class="academic-table-render font-serif text-sm">
                        {@html b.html}
                      </div>
                    </div>

                  <!-- SVG PLOT BLOCK -->
                  {:else if b.type === 'svg_plot'}
                    <div class="bg-white rounded-lg p-4 shadow-sm relative flex flex-col items-center">
                      <div class="absolute top-3 right-3 opacity-90 group-hover:opacity-100 transition-opacity flex items-center gap-2">
                        <button
                          class="px-2.5 py-1 text-xs bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-300 rounded font-medium shadow-sm transition-colors"
                          onclick={() => saveSvgAsFile(b.svg || '', `${(b.title || 'plot').toLowerCase().replace(/\s+/g, '_')}.svg`)}
                        >
                          💾 Save SVG
                        </button>
                      </div>
                      {#if b.title}
                        <h4 class="text-xs font-semibold text-slate-700 mb-2">{b.title}</h4>
                      {/if}
                      <div class="w-full flex justify-center max-w-full overflow-x-auto">
                        {@html b.svg}
                      </div>
                    </div>

                  <!-- WARNING BLOCK -->
                  {:else if b.type === 'warning'}
                    <div class="bg-amber-950/40 border border-amber-800/60 text-amber-200 rounded-lg p-3 text-xs flex items-start gap-2.5">
                      <span class="text-base leading-none">⚠️</span>
                      <div>
                        <strong>Warning:</strong> {b.content}
                      </div>
                    </div>

                  <!-- ERROR BLOCK -->
                  {:else if b.type === 'error'}
                    <div class="bg-rose-950/40 border border-rose-800/60 text-rose-200 rounded-lg p-3 text-xs flex items-start gap-2.5">
                      <span class="text-base leading-none">❌</span>
                      <div>
                        <strong>Error:</strong> {b.content}
                      </div>
                    </div>

                  <!-- PLAIN TEXT / MESSAGE BLOCK -->
                  {:else}
                    <div class="bg-slate-800/50 rounded-lg p-3 text-xs text-slate-300 font-mono">
                      {#if b.title}
                        <div class="font-bold text-slate-200 mb-1">{b.title}</div>
                      {/if}
                      <p>{b.content}</p>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  /* Academic APA / SPSS Table Styling for @html injected content */
  :global(.academic-table-render table) {
    width: 100%;
    border-collapse: collapse;
    margin: 8px 0;
    font-size: 13px;
  }
  :global(.academic-table-render table thead tr th) {
    border-top: 2px solid #0f172a;
    border-bottom: 1px solid #0f172a;
    padding: 6px 12px;
    font-weight: 600;
  }
  :global(.academic-table-render table tbody tr td) {
    padding: 6px 12px;
    border-bottom: 1px solid #e2e8f0;
  }
  :global(.academic-table-render table tbody tr:last-child td) {
    border-bottom: 2px solid #0f172a;
  }
  :global(.academic-table-render .num-cell) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }
  :global(.academic-table-render .rrad-table-title) {
    font-size: 14px;
    font-weight: bold;
    margin-bottom: 4px;
  }
</style>
