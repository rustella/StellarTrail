export function formatLocalDate(date = new Date()): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function calculateAge(
  birthDate: string | null | undefined,
  today = new Date(),
): number | null {
  const parsed = parseDateOnly(birthDate);
  if (!parsed) {
    return null;
  }
  let age = today.getFullYear() - parsed.year;
  const currentMonth = today.getMonth() + 1;
  const currentDay = today.getDate();
  if (
    currentMonth < parsed.month ||
    (currentMonth === parsed.month && currentDay < parsed.day)
  ) {
    age -= 1;
  }
  return age >= 0 ? age : null;
}

export function formatAgeText(birthDate: string | null | undefined): string {
  const age = calculateAge(birthDate);
  return age === null ? "" : `${age} 岁`;
}

function parseDateOnly(value: string | null | undefined): {
  year: number;
  month: number;
  day: number;
} | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value || "");
  if (!match) {
    return null;
  }
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(year, month - 1, day);
  if (
    date.getFullYear() !== year ||
    date.getMonth() !== month - 1 ||
    date.getDate() !== day
  ) {
    return null;
  }
  return { year, month, day };
}
