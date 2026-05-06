export function formatFileSize(bytes: number): string {
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits: 1,
    style: 'unit',
    unit: 'byte',
    unitDisplay: 'short',
  }).format(bytes);
}
