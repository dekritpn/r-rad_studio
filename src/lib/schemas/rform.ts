// src/lib/schemas/rform.ts
// JSON schema types and validator for .rform.json statistical dialogs

import type { MeasureType } from './dataset';

export type WidgetType =
  | 'VariableSourceList'
  | 'VariableTargetSlot'
  | 'FormulaBuilder'
  | 'Checkbox'
  | 'RadioGroup'
  | 'Dropdown'
  | 'NumericInput'
  | 'TextInput'
  | 'GroupBox'
  | 'SubDialogButton';

export interface GridPos {
  x: number; // 1 to 12
  y: number; // Row index (1-based)
  w: number; // Col span (1 to 12)
  h: number; // Row span
}

export interface TargetSlotAccepts {
  types?: MeasureType[];
  multiple?: boolean;
  required?: boolean;
}

export interface RadioOption {
  label: string;
  value: string | number | boolean;
}

export interface FormWidget {
  id: string;
  type: WidgetType;
  label?: string;
  grid?: GridPos;
  filter_types?: MeasureType[];
  accepts?: TargetSlotAccepts;
  default?: unknown;
  min?: number;
  max?: number;
  step?: number;
  options?: RadioOption[];
  sub_dialog_id?: string;
}

export interface SubDialogDefinition {
  id: string;
  label: string;
  layout: FormWidget[];
}

export interface RFormSchema {
  $schema?: string;
  id: string;
  version: string;
  menu_path: string[];
  window: {
    title: string;
    width: number;
    height: number;
  };
  layout: FormWidget[];
  sub_dialogs?: SubDialogDefinition[];
  r_template: string;
}

export interface MenuNode {
  label: string;
  children?: MenuNode[];
  form_id?: string;
  is_custom?: boolean;
}

export function validateRFormSchema(obj: unknown): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  if (!obj || typeof obj !== 'object') {
    return { valid: false, errors: ['Schema must be an object'] };
  }

  const form = obj as Partial<RFormSchema>;
  if (!form.id || typeof form.id !== 'string') {
    errors.push('Missing or invalid "id" field');
  }
  if (!form.version || typeof form.version !== 'string') {
    errors.push('Missing or invalid "version" field');
  }
  if (!Array.isArray(form.menu_path) || form.menu_path.length === 0) {
    errors.push('Missing or empty "menu_path" array');
  }
  if (!form.window || typeof form.window.title !== 'string') {
    errors.push('Missing or invalid "window.title"');
  }
  if (!Array.isArray(form.layout)) {
    errors.push('Missing or invalid "layout" widget array');
  }
  if (!form.r_template || typeof form.r_template !== 'string') {
    errors.push('Missing or invalid "r_template" code string');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
