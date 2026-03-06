import buffNameRaw from "./BuffName.json";

export type BuffAliasMap = Record<string, string>;

export type BuffDefinition = {
  baseId: number;
  name: string;
  spriteFile: string;
  searchKeywords: string[];
};

export type BuffNameInfo = {
  baseId: number;
  name: string;
  hasSpriteFile: boolean;
};

export type BuffMeta = {
  baseId: number;
  defaultName: string;
  hasSpriteFile: boolean;
  spriteFile: string | null;
  searchKeywords: string[];
};

type RawBuffEntry = {
  Id: number;
  Icon?: string | null;
  NameDesign?: string | null;
  SpriteFile?: string | null;
};

const rawBuffEntries = buffNameRaw as RawBuffEntry[];

const BUFF_META_MAP = new Map<number, BuffMeta>();
const AVAILABLE_BUFF_DEFINITIONS: BuffDefinition[] = [];

for (const entry of rawBuffEntries) {
  const defaultName = entry.NameDesign?.trim() ?? "";
  if (!defaultName) continue;

  const spriteFile = entry.SpriteFile?.trim() || null;
  const searchKeywords = [defaultName];
  const meta: BuffMeta = {
    baseId: entry.Id,
    defaultName,
    hasSpriteFile: Boolean(spriteFile),
    spriteFile,
    searchKeywords,
  };
  BUFF_META_MAP.set(entry.Id, meta);

  if (spriteFile) {
    AVAILABLE_BUFF_DEFINITIONS.push({
      baseId: entry.Id,
      name: defaultName,
      spriteFile,
      searchKeywords,
    });
  }
}

AVAILABLE_BUFF_DEFINITIONS.sort((a, b) => a.baseId - b.baseId);

function normalizeText(value: string): string {
  return value.trim().toLowerCase();
}

function normalizeAliasMap(aliases?: BuffAliasMap): BuffAliasMap {
  if (!aliases) return {};
  const next: BuffAliasMap = {};
  for (const [baseId, alias] of Object.entries(aliases)) {
    const trimmed = alias.trim();
    if (!trimmed) continue;
    next[baseId] = trimmed;
  }
  return next;
}

function getAlias(baseId: number, aliases?: BuffAliasMap): string | null {
  const normalizedAliases = normalizeAliasMap(aliases);
  const alias = normalizedAliases[String(baseId)]?.trim();
  return alias ? alias : null;
}

function getMatchRank(
  text: string | null | undefined,
  normalizedKeyword: string,
  exactRank: number,
  containsRank: number,
): number | null {
  if (!text) return null;
  const normalizedText = normalizeText(text);
  if (!normalizedText) return null;
  if (normalizedText === normalizedKeyword) return exactRank;
  if (normalizedText.includes(normalizedKeyword)) return containsRank;
  return null;
}

export function lookupBuffMeta(baseId: number): BuffMeta | undefined {
  return BUFF_META_MAP.get(baseId);
}

export function lookupDefaultBuffName(baseId: number): string | undefined {
  return lookupBuffMeta(baseId)?.defaultName;
}

export function getAvailableBuffDefinitions(): BuffDefinition[] {
  return AVAILABLE_BUFF_DEFINITIONS;
}

export function resolveBuffDisplayName(
  baseId: number,
  aliases?: BuffAliasMap,
): string {
  const alias = getAlias(baseId, aliases);
  if (alias) return alias;
  return lookupDefaultBuffName(baseId) ?? `#${baseId}`;
}

export function resolveBuffNameInfo(
  baseId: number,
  aliases?: BuffAliasMap,
): BuffNameInfo {
  const meta = lookupBuffMeta(baseId);
  return {
    baseId,
    name: resolveBuffDisplayName(baseId, aliases),
    hasSpriteFile: meta?.hasSpriteFile ?? false,
  };
}

export function searchBuffsByName(
  keyword: string,
  aliases?: BuffAliasMap,
  limit = 120,
): BuffNameInfo[] {
  const normalizedKeyword = normalizeText(keyword);
  if (!normalizedKeyword) return [];

  const normalizedAliases = normalizeAliasMap(aliases);
  const matches: Array<{ baseId: number; rank: number }> = [];

  for (const meta of BUFF_META_MAP.values()) {
    const alias = normalizedAliases[String(meta.baseId)] ?? null;
    const aliasRank = getMatchRank(alias, normalizedKeyword, 1, 2);
    const defaultRank = getMatchRank(meta.defaultName, normalizedKeyword, 3, 4);
    const rank = Math.min(aliasRank ?? Number.POSITIVE_INFINITY, defaultRank ?? Number.POSITIVE_INFINITY);
    if (!Number.isFinite(rank)) continue;
    matches.push({ baseId: meta.baseId, rank });
  }

  matches.sort((a, b) => a.rank - b.rank || a.baseId - b.baseId);

  return matches.slice(0, Math.max(1, limit)).map((match) =>
    resolveBuffNameInfo(match.baseId, normalizedAliases)
  );
}
