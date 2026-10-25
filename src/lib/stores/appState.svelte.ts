// src/lib/stores/appState.svelte.ts
// Central reactive state store for R-RAD Studio using Svelte 5 Runes

import { invoke } from '@tauri-apps/api/core';
import type { DatasetMetadata, VariableMeta } from '../schemas/dataset';
import type { OutputLogItem, OutputBlock } from '../schemas/output';
import type { RFormSchema, MenuNode } from '../schemas/rform';

export type ViewMode = 'data' | 'variable' | 'output' | 'designer';
export type DaemonStatus = 'connected' | 'running' | 'idle' | 'disconnected' | 'error';

class AppStateManager {
  // Navigation & View
  viewMode = $state<ViewMode>('data');
  daemonStatus = $state<DaemonStatus>('idle');
  rVersion = $state<string>('');

  // Undo / Redo & Data Safety State
  canUndo = $state<boolean>(false);
  canRedo = $state<boolean>(false);
  undoCount = $state<number>(0);
  redoCount = $state<number>(0);
  lastAction = $state<string>('');

  // Dataset State
  activeDataset = $state<DatasetMetadata | null>(null);
  dataRows = $state<Record<string, unknown>[]>([]);
  dataOffset = $state<number>(0);
  dataLimit = $state<number>(100);
  totalRows = $state<number>(0);
  isLoadingData = $state<boolean>(false);

  // Variable View selection
  selectedVarIndex = $state<number | null>(null);

  // Menu Registry
  menuTree = $state<MenuNode[]>([]);

  // Runtime Dialog
  activeDialog = $state<{
    isOpen: boolean;
    schema: RFormSchema | null;
    values: Record<string, unknown>;
    subDialogId: string | null;
  }>({
    isOpen: false,
    schema: null,
    values: {},
    subDialogId: null,
  });

  // Compute Variable Dialog
  isComputeDialogOpen = $state<boolean>(false);

  // Preferences Dialog
  isPreferencesOpen = $state<boolean>(false);

  openPreferences() {
    this.isPreferencesOpen = true;
  }

  closePreferences() {
    this.isPreferencesOpen = false;
  }

  // Find & Replace Dialog
  isFindReplaceOpen = $state<boolean>(false);
  findReplaceInitialTab = $state<'find' | 'replace'>('find');

  openFindReplace(tab: 'find' | 'replace' = 'find') {
    this.findReplaceInitialTab = tab;
    this.isFindReplaceOpen = true;
  }

  closeFindReplace() {
    this.isFindReplaceOpen = false;
  }

  // Data View Selection State
  selectedRowIndex = $state<number | null>(null);
  activeDataCell = $state<{ row: number; colName: string } | null>(null);
  findHighlightQuery = $state<string | null>(null);

  // Help & About Modal
  isHelpModalOpen = $state<boolean>(false);
  helpModalTab = $state<'user' | 'rad' | 'about'>('user');

  openHelp(tab: 'user' | 'rad' | 'about' = 'user') {
    this.helpModalTab = tab;
    this.isHelpModalOpen = true;
  }

  closeHelp() {
    this.isHelpModalOpen = false;
  }

  // Global Toast Notification
  toastMessage = $state<string | null>(null);

  showToast(msg: string) {
    this.toastMessage = msg;
    setTimeout(() => {
      if (this.toastMessage === msg) {
        this.toastMessage = null;
      }
    }, 3000);
  }

  // Output Log Stream
  outputLogs = $state<OutputLogItem[]>([]);
  selectedLogId = $state<string | null>(null);

  // Syntax Editor Tab
  syntaxCode = $state<string>('# R-RAD Studio Syntax Editor\n# Paste analysis syntax here or run directly\n\n');

  // Form Designer (RAD Mode)
  designer = $state<{
    schema: RFormSchema;
    selectedWidgetId: string | null;
    previewMode: boolean;
  }>({
    schema: {
      id: 'custom.analysis',
      version: '1.0.0',
      menu_path: ['Analyze', 'Custom', 'My Custom Dialog'],
      window: { title: 'Custom Analysis Dialog', width: 620, height: 440 },
      layout: [],
      sub_dialogs: [],
      r_template: 'local({\n  # Write R code template here\n})\n',
    },
    selectedWidgetId: null,
    previewMode: false,
  });

  // Helper Methods
  setView(mode: ViewMode) {
    this.viewMode = mode;
  }

  setDataset(meta: DatasetMetadata) {
    this.activeDataset = meta;
    this.totalRows = meta.row_count;
  }

  updateVariables(vars: VariableMeta[]) {
    if (this.activeDataset) {
      this.activeDataset.variables = vars;
      this.activeDataset.column_count = vars.length;
    }
  }

  setDataPage(rows: Record<string, unknown>[], total: number, offset: number) {
    this.dataRows = rows;
    this.totalRows = total;
    this.dataOffset = offset;
  }

  appendOutput(title: string, blocks: OutputBlock[]) {
    const id = 'out_' + Date.now() + '_' + Math.random().toString(36).substring(2, 6);
    const item: OutputLogItem = {
      id,
      timestamp: new Date().toLocaleTimeString(),
      title,
      blocks,
    };
    this.outputLogs.push(item);
    this.selectedLogId = id;
    this.viewMode = 'output';
  }

  openDialog(schema: RFormSchema) {
    // Initialize default values from schema
    const initialValues: Record<string, unknown> = {};
    for (const w of schema.layout) {
      if (w.default !== undefined) {
        initialValues[w.id] = w.default;
      } else if (w.type === 'VariableTargetSlot') {
        initialValues[w.id] = w.accepts?.multiple ? [] : null;
      }
    }
    this.activeDialog = {
      isOpen: true,
      schema,
      values: initialValues,
      subDialogId: null,
    };
  }

  closeDialog() {
    this.activeDialog.isOpen = false;
    this.activeDialog.subDialogId = null;
  }

  openDesignerWith(schema: RFormSchema) {
    // Clone schema for customization
    const cloned = JSON.parse(JSON.stringify(schema));
    if (!cloned.id.startsWith('custom.')) {
      cloned.id = 'custom.' + cloned.id;
      cloned.menu_path = ['Analyze', 'Custom', cloned.window.title + ' (Custom)'];
      cloned.window.title += ' (Customized)';
    }
    this.designer.schema = cloned;
    this.designer.selectedWidgetId = null;
    this.designer.previewMode = false;
    this.viewMode = 'designer';
  }

  // Undo / Redo & Data Safety Engine
  async refreshUndoStatus() {
    try {
      const status: any = await invoke('get_undo_status');
      this.canUndo = !!status.can_undo;
      this.canRedo = !!status.can_redo;
      this.undoCount = status.undo_count || 0;
      this.redoCount = status.redo_count || 0;
      this.lastAction = status.last_action || '';
    } catch (e) {
      console.warn('Failed to get undo status:', e);
    }
  }

  async undo() {
    if (!this.canUndo) return;
    try {
      this.isLoadingData = true;
      const meta: any = await invoke('undo_action');
      if (meta && meta.variables) {
        this.setDataset(meta);
        const slice: any = await invoke('get_data_page', {
          offset: this.dataOffset,
          limit: this.dataLimit
        });
        this.setDataPage(slice.rows, slice.total_rows, slice.offset);
        this.showToast('Undo berhasil');
      }
      await this.refreshUndoStatus();
    } catch (err: any) {
      console.error('Undo failed:', err);
      this.showToast('Gagal melakukan Undo: ' + (err.message || err));
    } finally {
      this.isLoadingData = false;
    }
  }

  async redo() {
    if (!this.canRedo) return;
    try {
      this.isLoadingData = true;
      const meta: any = await invoke('redo_action');
      if (meta && meta.variables) {
        this.setDataset(meta);
        const slice: any = await invoke('get_data_page', {
          offset: this.dataOffset,
          limit: this.dataLimit
        });
        this.setDataPage(slice.rows, slice.total_rows, slice.offset);
        this.showToast('Redo berhasil');
      }
      await this.refreshUndoStatus();
    } catch (err: any) {
      console.error('Redo failed:', err);
      this.showToast('Gagal melakukan Redo: ' + (err.message || err));
    } finally {
      this.isLoadingData = false;
    }
  }

  async restoreEmergencyCheckpoint() {
    try {
      this.isLoadingData = true;
      const meta: any = await invoke('restore_emergency_checkpoint');
      if (meta && meta.variables) {
        this.setDataset(meta);
        const slice: any = await invoke('get_data_page', {
          offset: 0,
          limit: this.dataLimit
        });
        this.setDataPage(slice.rows, slice.total_rows, slice.offset);
        await this.refreshUndoStatus();
        this.showToast('Data berhasil dipulihkan dari emergency checkpoint!');
      }
    } catch (err: any) {
      console.error('Restore emergency checkpoint failed:', err);
      alert('Gagal memulihkan dari emergency checkpoint: ' + (err.message || err));
    } finally {
      this.isLoadingData = false;
    }
  }

  async checkEmergencyCheckpoint(): Promise<{ exists: boolean; path?: string; timestamp?: string }> {
    try {
      const res: any = await invoke('check_emergency_checkpoint');
      return res;
    } catch {
      return { exists: false };
    }
  }
}

export const appState = new AppStateManager();
