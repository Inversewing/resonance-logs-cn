export const NO_DAMAGE_INFO = "-" as const;

export const DAMAGE_PROPERTY_LABELS: Readonly<Record<number, string>> = {
  0: "通用",
  1: "火",
  2: "水",
  3: "雷",
  4: "木",
  5: "风",
  6: "岩",
  7: "光",
  8: "暗",
};

export const DAMAGE_MODE_LABELS: Readonly<Record<number, string>> = {
  1: "物理",
  2: "魔法",
};

export function propertyLabel(value: number | null | undefined): string {
  if (value == null) return NO_DAMAGE_INFO;
  return DAMAGE_PROPERTY_LABELS[value] ?? NO_DAMAGE_INFO;
}

export function damageModeLabel(value: number | null | undefined): string {
  if (value == null) return NO_DAMAGE_INFO;
  return DAMAGE_MODE_LABELS[value] ?? NO_DAMAGE_INFO;
}
