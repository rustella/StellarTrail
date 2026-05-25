import { getGearSpecKeyRankings, getGearTagSuggestions } from "./api-gears";
import type {
  GearCategory,
  GearSpecKeyRankingsResponse,
  GearTagSuggestionsResponse,
} from "./gear-utils";

const CACHE_TTL_MS = 5 * 60 * 1000;

const specRankingsByCategory = new Map<
  GearCategory,
  { expiresAt: number; response: GearSpecKeyRankingsResponse }
>();
const tagSuggestionsByLimit = new Map<
  number,
  { expiresAt: number; response: GearTagSuggestionsResponse }
>();

export async function getCachedGearSpecKeyRankings(
  category: GearCategory,
): Promise<GearSpecKeyRankingsResponse> {
  const cached = specRankingsByCategory.get(category);
  if (cached && cached.expiresAt > Date.now()) {
    return cached.response;
  }
  const response = await getGearSpecKeyRankings(category);
  specRankingsByCategory.set(category, {
    expiresAt: Date.now() + CACHE_TTL_MS,
    response,
  });
  return response;
}

export async function getCachedGearTagSuggestions(
  limit = 20,
): Promise<GearTagSuggestionsResponse> {
  const cached = tagSuggestionsByLimit.get(limit);
  if (cached && cached.expiresAt > Date.now()) {
    return cached.response;
  }
  const response = await getGearTagSuggestions(limit);
  tagSuggestionsByLimit.set(limit, {
    expiresAt: Date.now() + CACHE_TTL_MS,
    response,
  });
  return response;
}

export function clearGearFormSuggestionCaches(): void {
  tagSuggestionsByLimit.clear();
}
