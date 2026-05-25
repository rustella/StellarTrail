export function indexedAppendData<T>(
  path: string,
  startIndex: number,
  items: T[],
): Record<string, T> {
  const data: Record<string, T> = {};
  items.forEach((item, index) => {
    data[`${path}[${startIndex + index}]`] = item;
  });
  return data;
}

export function delayNextTick(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}
