// src/lib/schemas/dataset.ts
// Data structures for dataset metadata, variable view, and data pagination

export type MeasureType = 'scale' | 'ordinal' | 'nominal';
export type VariableType = 'Numeric' | 'String' | 'Date';
export type AlignmentType = 'left' | 'right' | 'center';

export interface ValueLabel {
  value: string;
  label: string;
}

export interface VariableMeta {
  name: string;
  type: VariableType;
  width: number;
  decimals: number;
  label: string;
  values: ValueLabel[];
  missing: string;
  columns: number;
  align: AlignmentType;
  measure: MeasureType;
  storage_type?: string;
}

export interface DatasetMetadata {
  dataset_name: string;
  row_count: number;
  column_count: number;
  variables: VariableMeta[];
}

export interface DataSlice {
  rows: Record<string, unknown>[];
  total_rows: number;
  offset: number;
  limit: number;
}
