export function formatErrorMessage(error: unknown): string {
  const message = String(error);
  const replacements: [RegExp, string][] = [
    [/Session not found/i, 'セッションが見つかりません。認証を行ってください。'],
    [/Session expired/i, 'セッションの有効期限が切れました。再度認証してください。'],
    [/Chrome extension files not found/i, 'Chrome 拡張機能のファイルが見つかりません。'],
  ];
  return replacements.reduce(
    (text, [pattern, replacement]) => text.replace(pattern, replacement),
    message,
  );
}
