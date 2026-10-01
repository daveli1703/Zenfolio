export function parseGoalValues(target: string, current: string) {
  const scale = Math.max(decimalPlaces(target), decimalPlaces(current));
  if (scale > 3) throw new Error('Use at most three decimal places.');
  return {
    targetValue: parseAtScale(target, scale),
    currentValue: parseAtScale(current, scale),
    decimalScale: scale,
  };
}

export function parseAtScale(value: string, scale: number): string {
  const match = value.trim().match(/^(\d+)(?:\.(\d+))?$/);
  if (!match) throw new Error('Enter a nonnegative number.');
  const fraction = match[2] ?? '';
  if (fraction.length > scale)
    throw new Error(`Use at most ${scale} decimal places.`);
  return (
    BigInt(match[1]) * 10n ** BigInt(scale) +
    BigInt((fraction + '0'.repeat(scale)).slice(0, scale) || '0')
  ).toString();
}

export function formatScaled(value: string, scale: number): string {
  const digits = value.padStart(scale + 1, '0');
  if (scale === 0) return digits;
  const result = `${digits.slice(0, -scale)}.${digits.slice(-scale)}`;
  return result.replace(/\.0+$|(?<=\.[0-9]*?)0+$/g, '').replace(/\.$/, '');
}

function decimalPlaces(value: string) {
  const match = value.trim().match(/^\d+(?:\.(\d+))?$/);
  if (!match) throw new Error('Enter a nonnegative number.');
  return match[1]?.length ?? 0;
}
