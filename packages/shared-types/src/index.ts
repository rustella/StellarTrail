export interface HealthResponse {
  status: 'ok';
}

export interface MetaResponse {
  name: 'StellarTrail';
  env: string;
  database_kind: 'sqlite' | 'postgres' | 'mysql';
}

export type DifficultyLevel =
  | 'leisure'
  | 'beginner'
  | 'intermediate'
  | 'advanced'
  | 'technical';

export interface RouteSummary {
  id: string;
  title: string;
  province: string;
  difficulty_level: DifficultyLevel;
  distance_m?: number;
  ascent_m?: number;
  duration_min?: number;
  best_seasons: string[];
  summary: string;
}
