<!-- src/lib/components/runtime/widgets/StandardWidgets.svelte -->
<!-- Standard Statistical Form Controls: Checkbox, NumericInput, TextInput, Dropdown, RadioGroup -->
<script lang="ts">
  import type { FormWidget } from '../../../schemas/rform';

  interface Props {
    widget: FormWidget;
    value: unknown;
    onChange: (val: unknown) => void;
    onOpenSubDialog?: (subDialogId: string) => void;
  }

  let { widget, value, onChange, onOpenSubDialog }: Props = $props();
</script>

{#if widget.type === 'Checkbox'}
  <label class="flex items-center gap-2 cursor-pointer text-xs text-slate-200 select-none py-1">
    <input
      type="checkbox"
      checked={Boolean(value)}
      onchange={(e) => onChange((e.target as HTMLInputElement).checked)}
      class="w-4 h-4 rounded border-slate-700 bg-slate-900 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-slate-900"
    />
    <span>{widget.label}</span>
  </label>

{:else if widget.type === 'NumericInput'}
  <div class="flex flex-col gap-1 py-1">
    {#if widget.label}
      <label class="text-xs font-semibold text-slate-300">{widget.label}</label>
    {/if}
    <input
      type="number"
      value={value !== undefined ? value : widget.default ?? 0}
      min={widget.min}
      max={widget.max}
      step={widget.step ?? 1}
      oninput={(e) => onChange(parseFloat((e.target as HTMLInputElement).value))}
      class="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500 font-mono"
    />
  </div>

{:else if widget.type === 'TextInput'}
  <div class="flex flex-col gap-1 py-1">
    {#if widget.label}
      <label class="text-xs font-semibold text-slate-300">{widget.label}</label>
    {/if}
    <input
      type="text"
      value={String(value ?? widget.default ?? '')}
      oninput={(e) => onChange((e.target as HTMLInputElement).value)}
      class="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
    />
  </div>

{:else if widget.type === 'Dropdown'}
  <div class="flex flex-col gap-1 py-1">
    {#if widget.label}
      <label class="text-xs font-semibold text-slate-300">{widget.label}</label>
    {/if}
    <select
      value={String(value ?? widget.default ?? '')}
      onchange={(e) => onChange((e.target as HTMLSelectElement).value)}
      class="bg-slate-900 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
    >
      {#each widget.options || [] as opt}
        <option value={String(opt.value)}>{opt.label}</option>
      {/each}
    </select>
  </div>

{:else if widget.type === 'RadioGroup'}
  <div class="flex flex-col gap-1.5 py-1">
    {#if widget.label}
      <label class="text-xs font-semibold text-slate-300">{widget.label}</label>
    {/if}
    <div class="flex flex-col gap-1">
      {#each widget.options || [] as opt}
        <label class="flex items-center gap-2 cursor-pointer text-xs text-slate-200 select-none">
          <input
            type="radio"
            name={widget.id}
            value={String(opt.value)}
            checked={String(value ?? widget.default) === String(opt.value)}
            onchange={() => onChange(opt.value)}
            class="w-3.5 h-3.5 border-slate-700 bg-slate-900 text-indigo-600 focus:ring-indigo-500"
          />
          <span>{opt.label}</span>
        </label>
      {/each}
    </div>
  </div>

{:else if widget.type === 'SubDialogButton'}
  <div class="py-1">
    <button
      type="button"
      class="w-full px-3 py-1.5 bg-slate-800 hover:bg-slate-750 border border-slate-700 text-slate-200 rounded text-xs font-medium transition-colors shadow-sm text-center"
      onclick={() => onOpenSubDialog && onOpenSubDialog(widget.sub_dialog_id || widget.id)}
    >
      {widget.label || 'Options...'}
    </button>
  </div>
{/if}
