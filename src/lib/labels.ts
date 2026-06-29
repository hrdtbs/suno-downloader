export function formatErrorMessage(error: unknown): string {
  const message = String(error);
  const replacements: [RegExp, string][] = [
    [/Session not found/i, 'セッションが見つかりません。認証を行ってください。'],
    [/Session expired/i, 'セッションの有効期限が切れました。再度認証してください。'],
    [/Token is invalid or expired/i, 'トークンが無効または期限切れです。'],
  ];
  return replacements.reduce(
    (text, [pattern, replacement]) => text.replace(pattern, replacement),
    message,
  );
}
