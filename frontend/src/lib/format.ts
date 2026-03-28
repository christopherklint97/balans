/**
 * Swedish number formatting utilities.
 * Uses comma as decimal separator and space as thousands separator.
 * Example: 100000.23 → "100 000,23"
 */

/** Format a number to Swedish display format: "100 000,23" */
export function formatSEK(value: number | string): string {
  const num = typeof value === 'string' ? parseFloat(value) : value;
  if (isNaN(num)) return '0,00';

  const fixed = num.toFixed(2);
  const [intPart, decPart] = fixed.split('.');
  const isNegative = intPart.startsWith('-');
  const absInt = isNegative ? intPart.slice(1) : intPart;

  // Add space as thousands separator
  const withSpaces = absInt.replace(/\B(?=(\d{3})+(?!\d))/g, '\u00A0');
  return `${isNegative ? '-' : ''}${withSpaces},${decPart}`;
}

/**
 * Parse a Swedish-formatted number string to a plain number.
 * Handles both comma and dot as decimal separators.
 * Strips spaces (thousands separators).
 */
export function parseSEK(value: string): number {
  if (!value) return 0;
  // Remove all spaces (thin, non-breaking, regular)
  const cleaned = value.replace(/[\s\u00A0]/g, '').replace(',', '.');
  const num = parseFloat(cleaned);
  return isNaN(num) ? 0 : num;
}

/**
 * Format a raw input value for display in an input field.
 * Only adds comma decimal — no thousands separator while editing.
 */
export function formatInputValue(value: string): string {
  if (!value) return '';
  return value.replace('.', ',');
}

/**
 * Normalize input: allow digits, comma, and one decimal separator.
 * Returns the cleaned string suitable for state storage.
 */
export function normalizeAmountInput(value: string): string {
  // Remove spaces
  let cleaned = value.replace(/[\s\u00A0]/g, '');
  // Replace comma with dot for internal storage
  cleaned = cleaned.replace(',', '.');
  // Only allow digits, one dot, and optional leading minus
  const match = cleaned.match(/^-?\d*\.?\d{0,2}/);
  return match ? match[0] : '';
}
