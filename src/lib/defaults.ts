export const DEFAULT_SINCE_FILTER = '2w';

export const SINCE_FILTER_OPTIONS = [
  { value: '', label: '制限なし' },
  { value: '12h', label: '12時間以内' },
  { value: '1d', label: '1日以内' },
  { value: '2d', label: '2日以内' },
  { value: '3d', label: '3日以内' },
  { value: '5d', label: '5日以内' },
  { value: '1w', label: '1週間以内' },
  { value: '2w', label: '2週間以内' },
  { value: '3w', label: '3週間以内' },
  { value: '1m', label: '1か月以内' },
  { value: '2m', label: '2か月以内' },
  { value: '3m', label: '3か月以内' },
  { value: '6m', label: '6か月以内' },
  { value: '12m', label: '1年以内' },
] as const;

export function normalizeSinceFilter(since: string | null | undefined): string {
  return since === '' ? '' : (since ?? DEFAULT_SINCE_FILTER);
}

export function sinceFilterForApi(since: string): string | undefined {
  return since === '' ? '' : since.trim() || undefined;
}
