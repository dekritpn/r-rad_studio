// src/lib/schemas/output.ts
// Structured output blocks emitted by R Daemon for the Output Viewer

export type BlockType = 'syntax' | 'html_table' | 'svg_plot' | 'warning' | 'error' | 'text';

export interface OutputBlock {
  type: BlockType;
  title?: string;
  content?: string;
  html?: string;
  svg?: string;
}

export interface AnalysisResult {
  request_id: string;
  status: 'ok' | 'error';
  blocks?: OutputBlock[];
  error?: string;
}

export interface OutputLogItem {
  id: string;
  timestamp: string;
  title: string;
  blocks: OutputBlock[];
}
