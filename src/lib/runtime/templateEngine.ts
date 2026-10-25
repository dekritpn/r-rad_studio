// src/lib/runtime/templateEngine.ts
// Handlebars R-code template compilation with safe backtick escaping and namespace hygiene

import Handlebars from 'handlebars';

// Helper to escape R variable names (e.g. `Blood Pressure (Systolic)`)
export function escapeRVariable(varName: string): string {
  if (!varName) return '';
  // If already escaped, return as is
  if (varName.startsWith('`') && varName.endsWith('`')) {
    return varName;
  }
  // Check if variable contains non-standard characters
  const needsEscaping = /[^a-zA-Z0-9._]/.test(varName) || /^[0-9_]/.test(varName);
  return needsEscaping ? `\`${varName}\`` : varName;
}

// Register custom Handlebars helpers for R syntax generation
Handlebars.registerHelper('escapeR', function (v: unknown) {
  if (typeof v === 'string') {
    return new Handlebars.SafeString(escapeRVariable(v));
  }
  return v;
});

Handlebars.registerHelper('eq', function (a, b) {
  return a === b;
});

Handlebars.registerHelper('ne', function (a, b) {
  return a !== b;
});

Handlebars.registerHelper('divide', function (a: number, b: number) {
  return a / b;
});

Handlebars.registerHelper('multiply', function (a: number, b: number) {
  return a * b;
});

// Joins an array of variable names with separator, automatically escaping identifiers
Handlebars.registerHelper('joinR', function (arr: unknown[], separator: string = ', ') {
  if (!Array.isArray(arr)) return '';
  return new Handlebars.SafeString(
    arr.map((item) => (typeof item === 'string' ? escapeRVariable(item) : String(item))).join(separator)
  );
});

function formatValueForR(val: unknown): unknown {
  if (typeof val === 'boolean') {
    return val ? 'TRUE' : 'FALSE';
  }
  if (Array.isArray(val)) {
    return val.map(formatValueForR);
  }
  if (val !== null && typeof val === 'object') {
    const res: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(val as Record<string, unknown>)) {
      res[k] = formatValueForR(v);
    }
    return res;
  }
  return val;
}

export function compileRTemplate(templateStr: string, context: Record<string, unknown>): string {
  try {
    const template = Handlebars.compile(templateStr, { noEscape: true });
    // Process context to auto-convert booleans to uppercase TRUE/FALSE for R
    const processedContext = formatValueForR(context) as Record<string, unknown>;

    return template(processedContext).trim();
  } catch (err) {
    console.error('Error compiling R template:', err);
    throw new Error(`Failed to compile R template: ${err instanceof Error ? err.message : String(err)}`);
  }
}
