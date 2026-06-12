export interface HealthResponse {
  status: "ok";
}

export interface MetaResponse {
  name: "StellarTrail";
  env: string;
  database_kind: "sqlite" | "postgres" | "mysql";
}

export type ClientKey =
  | "wechat_miniprogram"
  | "web"
  | "android"
  | "ios"
  | "macos";

export type ClientVersionStatus = "draft" | "published";

export type ClientVersionReleaseNoteSectionKey =
  | "feature"
  | "bug_fix"
  | "notes";

export interface ClientVersionReleaseNoteSection {
  key: ClientVersionReleaseNoteSectionKey;
  title: string;
  items: string[];
}

export interface ClientVersion {
  id: string;
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes: string[];
  release_note_sections: ClientVersionReleaseNoteSection[];
  status: ClientVersionStatus;
  commit_hash?: string | null;
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListClientVersionsRequest {
  client_key?: ClientKey;
  status?: ClientVersionStatus;
  limit?: number;
  cursor?: string;
}

export interface ListClientVersionsResponse {
  items: ClientVersion[];
  next_cursor?: string | null;
}

export interface ClientVersionRequest {
  client_key: ClientKey;
  version: string;
  title: string;
  release_notes?: string[];
  release_note_sections?: ClientVersionReleaseNoteSection[];
  status: ClientVersionStatus;
  commit_hash?: string | null;
}

export type RoadmapStatus =
  | "planned"
  | "designing"
  | "building"
  | "preview"
  | "shipped";

export type RoadmapCategory =
  | "gear"
  | "skills"
  | "routes"
  | "offline"
  | "safety"
  | "community";

export interface RoadmapItem {
  id: string;
  client_key: ClientKey;
  title: string;
  summary: string;
  details?: string | null;
  category: RoadmapCategory | string;
  status: RoadmapStatus | string;
  priority: number;
  sort_order: number;
  is_published: boolean;
  vote_count: number;
  subscription_count: number;
  is_voted: boolean;
  is_subscribed: boolean;
  published_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListRoadmapRequest {
  client_key?: ClientKey;
  status?: RoadmapStatus;
  limit?: number;
  cursor?: string;
}

export interface ListRoadmapResponse {
  items: RoadmapItem[];
  next_cursor?: string | null;
}

export interface RoadmapItemRequest {
  client_key: ClientKey;
  title: string;
  summary: string;
  details?: string | null;
  category: RoadmapCategory;
  status: RoadmapStatus;
  priority: number;
  sort_order: number;
  is_published: boolean;
}

export type RoadmapInteractionStatusResponse = RoadmapItem;

export interface ContentListResponse<T> {
  items: T[];
}

export type AppLocale = "zh-CN" | "en";
export type SkillLocale = AppLocale;

export interface SkillCategorySummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  item_count: number;
  href: string;
}

export interface SkillCategoriesResponse {
  items: SkillCategorySummary[];
}

export interface PageInfo {
  limit: number;
  offset: number;
  next_offset?: number | null;
}

export interface KnotTaxonomyItem {
  id: string;
  slug: string;
  title: string;
}

export type KnotMediaAssetId =
  | "thumbnail"
  | "preview"
  | "draw_gif"
  | "turntable_gif"
  | "draw_mp4"
  | "turntable_mp4";

export type KnotMediaType = KnotMediaAssetId;

export interface KnotMediaAsset {
  id: KnotMediaAssetId | string;
  media_type: KnotMediaType | string;
  url: string;
  mime_type: string;
  width?: number | null;
  height?: number | null;
  size_bytes: number;
  attribution?: string | null;
  license_note?: string | null;
}

export interface KnotMediaUploadResponse {
  status: "uploaded";
  knot_id: string;
  media: KnotMediaAsset;
}

export interface KnotSummary {
  id: string;
  slug: string;
  title: string;
  summary: string;
  aliases: string[];
  categories: KnotTaxonomyItem[];
  types: KnotTaxonomyItem[];
  media: KnotMediaAsset[];
  href: string;
}

export interface KnotFilterOption {
  id: string;
  slug?: string | null;
  title: string;
  count: number;
}

export interface KnotFiltersResponse {
  locale: SkillLocale;
  categories: KnotFilterOption[];
}

export interface KnotListResponse {
  locale: SkillLocale;
  items: KnotSummary[];
  page: PageInfo;
}

export interface KnotOfflineManifestResponse {
  locale: SkillLocale;
  item_count: number;
  media_count: number;
  estimated_bytes: number;
  items: KnotDetail[];
}

export interface KnotDetail extends Omit<KnotSummary, "href"> {
  href?: string;
  description?: string | null;
  steps: string[];
  locale: SkillLocale;
}

export interface ListKnotsRequest {
  offset?: number;
  limit?: number;
  category?: string;
  q?: string;
}

export interface KnotDisclaimerResponse {
  key: string;
  version: string;
  title: string;
  content: string;
  accepted: boolean;
  accepted_at?: string | null;
}

export interface AcceptKnotDisclaimerRequest {
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
}

export type ApiUsageMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ApiUsageListRequest {
  from?: string;
  to?: string;
  user_id?: string;
  method?: ApiUsageMethod;
  route?: string;
  limit?: number;
  offset?: number;
}

export interface ApiUsageSummary {
  bucket_date: string;
  user_id?: string | null;
  method: ApiUsageMethod | string;
  route_pattern: string;
  status_code: number;
  call_count: number;
  first_called_at: string;
  last_called_at: string;
}

export interface ApiUsageListResponse {
  items: ApiUsageSummary[];
  page: PageInfo;
}

export type AdminRole = "admin" | "super_admin";

export interface AdminUserSelector {
  username?: string | null;
  user_id?: string | null;
}

export interface AdminRoleResponse {
  user_id: string;
  role: AdminRole;
}

export type DeletedFilter = "active" | "deleted" | "all";

export interface UploadImageInfo {
  id: string;
  purpose: string;
  original_filename: string;
  image_type: string;
  content_type: string;
  size_bytes: number;
  sha256: string;
  download_url: string;
  is_deleted: boolean;
  created_at: string;
}

export interface AdminFeedbackUser {
  id: string;
  username?: string | null;
  email?: string | null;
  nickname?: string | null;
  avatar_url?: string | null;
}

export interface AdminFeedbackItem {
  id: string;
  user: AdminFeedbackUser;
  category: string;
  content: string;
  contact?: string | null;
  page?: string | null;
  client_platform?: string | null;
  client_version?: string | null;
  device_model?: string | null;
  status: string;
  images: UploadImageInfo[];
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface ListAdminFeedbackRequest {
  status?: string;
  deleted?: DeletedFilter;
  limit?: number;
  cursor?: string;
}

export interface ListAdminFeedbackResponse {
  items: AdminFeedbackItem[];
  next_cursor?: string | null;
}

export interface GearTemplateCategory {
  id: string;
  name: string;
  items: string[];
}

export interface GearTemplate {
  id: string;
  title: string;
  categories: GearTemplateCategory[];
}

export type GearCategory =
  | "backpack_system"
  | "sleep_system"
  | "kitchen_system"
  | "walking_system"
  | "clothing_system"
  | "lighting_system"
  | "first_aid_system"
  | "electronics_system"
  | "technical_gear"
  | "other_gear"
  | "consumable";

export type GearStatus =
  | "available"
  | "in_use"
  | "maintenance"
  | "damaged"
  | "lost"
  | "retired"
  | "sold"
  | "idle";

export type GearShareStatus =
  | "not_shared"
  | "pending"
  | "approved"
  | "rejected"
  | "withdrawn";

export type GearTab = "available" | "history";

export type GearSort =
  | "created_at_desc"
  | "created_at_asc"
  | "purchase_date_desc"
  | "name_asc"
  | "weight_desc"
  | "price_desc";

export type GearCurrency = "CNY" | "USD" | "EUR" | "JPY" | "HKD";
export type GearSpecs = Record<string, string>;
export interface GearVariant {
  key: string;
  label: string;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  weight_g?: number | null;
}
export type GearTagColor =
  | "teal"
  | "blue"
  | "violet"
  | "rose"
  | "orange"
  | "amber"
  | "green"
  | "slate";
export type GearTagColorMap = Record<string, GearTagColor | string>;

export interface GearItem {
  id: string;
  user_id: string;
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_location?: string | null;
  status: GearStatus;
  storage_location?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity: number;
  specs?: GearSpecs | null;
  tags: string[];
  tag_colors?: GearTagColorMap | null;
  share_enabled: boolean;
  share_status: GearShareStatus;
  notes?: string | null;
  archived_at?: string | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface GearSummary {
  id: string;
  category: GearCategory;
  category_label: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  status: GearStatus;
  status_label: string;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity: number;
  specs?: GearSpecs | null;
  tags: string[];
  tag_colors?: GearTagColorMap | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateGearRequest {
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  purchase_date?: string | null;
  purchase_price_cents?: number | null;
  purchase_price_currency?: GearCurrency | string | null;
  purchase_location?: string | null;
  status?: GearStatus | null;
  storage_location?: string | null;
  atlas_item_id?: string | null;
  selected_variant_key?: string | null;
  selected_variant_label?: string | null;
  quantity?: number | null;
  specs?: GearSpecs | null;
  tags?: string[];
  tag_colors?: GearTagColorMap | null;
  share_enabled?: boolean;
  notes?: string | null;
}

export type UpdateGearRequest = Partial<CreateGearRequest>;

export interface GearCategoryCount {
  category: GearCategory;
  label: string;
  count: number;
  total_weight_g: number;
  total_value_cents: number;
}

export interface GearStatusCount {
  status: GearStatus;
  label: string;
  count: number;
  total_weight_g: number;
  total_value_cents: number;
}

export interface GearStatsResponse {
  current_count: number;
  archived_count: number;
  total_value_cents: number;
  total_weight_g: number;
  by_category: GearCategoryCount[];
  by_status: GearStatusCount[];
}

export interface GearCategoryFilter {
  id: "all" | GearCategory;
  label: string;
  count: number;
}

export interface GearCategoriesResponse {
  items: GearCategoryFilter[];
}

export interface GearSpecKeyRankingsResponse {
  keys: string[];
}

export interface GearTagSuggestion {
  tag: string;
  color?: GearTagColor | string | null;
}

export interface GearTagSuggestionsResponse {
  items: GearTagSuggestion[];
}

export interface ListGearsRequest {
  tab?: GearTab;
  category?: GearCategory;
  status?: GearStatus;
  deleted?: DeletedFilter;
  q?: string;
  sort?: GearSort;
  limit?: number;
  cursor?: string;
}

export interface ListGearsResponse {
  items: GearSummary[];
  next_cursor?: string | null;
}

export interface GearOverviewRequest {
  tab?: GearTab;
  limit?: number;
  sort?: GearSort;
}

export interface GearOverviewResponse {
  categories: GearCategoriesResponse;
  stats: GearStatsResponse;
  list: ListGearsResponse;
}

export interface GearPackingListStats {
  item_count: number;
  packed_count: number;
  total_weight_g: number;
}

export interface GearPackingListSummary extends GearPackingListStats {
  id: string;
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ListGearPackingListsRequest {
  limit?: number;
  cursor?: string;
}

export interface ListGearPackingListsResponse {
  items: GearPackingListSummary[];
  next_cursor?: string | null;
}

export interface CreateGearPackingListRequest {
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
}

export type UpdateGearPackingListRequest = CreateGearPackingListRequest;

export interface AddGearPackingItemsRequest {
  gear_ids: string[];
}

export interface UpdateGearPackingItemRequest {
  packed?: boolean;
  planned_quantity?: number | null;
  packed_quantity?: number | null;
}

export type GearPackingUnavailableReason = "deleted" | string;

export interface GearPackingListItem {
  id: string;
  gear_id: string;
  planned_quantity: number;
  packed_quantity: number;
  packed: boolean;
  unavailable: boolean;
  unavailable_reason?: GearPackingUnavailableReason | null;
  gear: GearSummary;
  created_at: string;
  updated_at: string;
}

export interface GearPackingListDetail {
  id: string;
  name: string;
  route_name?: string | null;
  duration_label?: string | null;
  stats: GearPackingListStats;
  items: GearPackingListItem[];
  created_at: string;
  updated_at: string;
}

export type FieldVersions = Record<string, number>;

export type TripSectionKey =
  | "members"
  | "personal_gear"
  | "itinerary"
  | "shared_gear"
  | "food_plan"
  | "medical_kit"
  | "safety_plan"
  | "rescue_info"
  | "budget"
  | "goals";

export interface TripFieldConflict {
  field: string;
  client_value: unknown;
  server_value: unknown;
  server_version: number;
}

export interface TripConflictResponse {
  code: "edit_conflict";
  message: string;
  conflicts: TripFieldConflict[];
}

export type TripType = "solo" | "team";
export type TripTimeBucket = "ongoing" | "upcoming" | "past" | "undated";

export interface TripReadiness {
  missing_count: number;
  missing_labels: string[];
  completion_percent: number;
}

export interface Trip {
  id: string;
  owner_user_id: string;
  trip_type: TripType;
  title: string;
  description?: string | null;
  start_date?: string | null;
  end_date?: string | null;
  enabled_sections: TripSectionKey[];
  route_use_slope_adjustment: boolean;
  route_use_high_altitude_adjustment: boolean;
  route_start_altitude_m?: number | null;
  day_count: number;
  field_versions: FieldVersions;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface TripSummary extends Trip {
  time_bucket: TripTimeBucket;
  days_until_start?: number | null;
  days_until_end?: number | null;
  member_count: number;
  readiness: TripReadiness;
  outdoor_experience_id?: string | null;
}

export interface ListTripsRequest {
  limit?: number;
  cursor?: string;
  bucket?: TripTimeBucket | "all";
  trip_type?: TripType | "all";
  today?: string;
}

export interface ListTripsResponse {
  items: TripSummary[];
  next_cursor?: string | null;
}

export type TripHomeHighlightStatus = "ongoing" | "upcoming";

export interface TripHomeHighlightItem {
  trip: TripSummary;
  status: TripHomeHighlightStatus;
  days_until_start: number;
  days_until_end: number;
}

export interface TripHomeHighlightResponse {
  item?: TripHomeHighlightItem | null;
}

export interface CreateTripRequest {
  trip_type: TripType;
  title: string;
  description?: string | null;
  start_date?: string | null;
  end_date?: string | null;
  route_use_slope_adjustment?: boolean;
  route_use_high_altitude_adjustment?: boolean;
  route_start_altitude_m?: number | null;
}

export interface TripPatchMeta {
  base_field_versions?: FieldVersions;
  force_fields?: string[];
}

export type UpdateTripRequest = Partial<Omit<CreateTripRequest, "trip_type">> &
  TripPatchMeta;

export interface UpdateTripSectionsRequest extends TripPatchMeta {
  enabled_sections: TripSectionKey[];
}

export interface TripMemberProfile {
  display_name: string;
  outdoor_id?: string | null;
  real_name?: string | null;
  gender?: string | null;
  age?: number | null;
  height_cm?: number | null;
  phone?: string | null;
  emergency_contact?: string | null;
  emergency_contact_relationship?: string | null;
  emergency_phone?: string | null;
  blood_type?: string | null;
  medical_history?: string | null;
  allergy_history?: string | null;
  medical_response_note?: string | null;
  diet_preference?: string | null;
  insurance_policy_no?: string | null;
  insurance_company_phone?: string | null;
  experience_note?: string | null;
  role_label?: string | null;
}

export interface TripMember {
  id: string;
  plan_id: string;
  user_id: string;
  is_owner: boolean;
  profile: TripMemberProfile;
  field_versions: FieldVersions;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface TripGearSnapshotBase {
  id: string;
  category: GearCategory;
  category_label: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  planned_quantity: number;
  packed_quantity: number;
  unit_weight_g?: number | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripPersonalGearItem extends TripGearSnapshotBase {
  member_id: string;
  source_packing_list_id?: string | null;
  source_packing_item_id?: string | null;
  source_gear_id?: string | null;
}

export interface TripSharedGearDemand extends TripGearSnapshotBase {
  source_member_id?: string | null;
  source_gear_id?: string | null;
  responsible_member_id: string;
  created_by_user_id?: string | null;
  template_key?: string | null;
  demand_name?: string | null;
  concrete_name?: string | null;
}

export interface SharedGearDemandTemplate {
  template_key: string;
  demand_name: string;
  group_label: string;
  category: GearCategory;
  category_label: string;
  planned_quantity: number;
  sort_order: number;
}

export interface TripRouteSegment {
  id: string;
  name: string;
  start_point?: string | null;
  end_point?: string | null;
  checkpoint?: string | null;
  leader_member_id?: string | null;
  bailout_route?: string | null;
  trail_condition?: string | null;
  distance_km: number;
  ascent_m: number;
  descent_m: number;
  descent_profile: "none" | "gentle" | "steep" | string;
  technical_factor: number;
  rest_factor: number;
  pack_factor: number;
  formula_estimate_minutes: number;
  final_estimate_minutes: number;
  manual_estimate_minutes?: number | null;
  estimated_start_altitude_m?: number | null;
  estimated_end_altitude_m?: number | null;
  estimated_highest_altitude_m?: number | null;
  high_altitude_factor?: number | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripItineraryTimeSlot {
  id: string;
  day_id: string;
  slot_key: "morning" | "afternoon" | "evening" | string;
  route_segment_id?: string | null;
  route_description?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripItineraryDay {
  id: string;
  day_index: number;
  date_label?: string | null;
  title?: string | null;
  notes?: string | null;
  weather?: string | null;
  high_temperature_c?: number | null;
  low_temperature_c?: number | null;
  weather_summary?: string | null;
  weather_notes?: string | null;
  camp_name?: string | null;
  camp_altitude_m?: number | null;
  camp_terrain?: string | null;
  camp_slope?: string | null;
  camp_area?: string | null;
  camp_water_source?: string | null;
  camp_notes?: string | null;
  estimate_minutes: number;
  time_slots: TripItineraryTimeSlot[];
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripFoodItem {
  id: string;
  food_meal_id: string;
  name: string;
  amount_g?: number | null;
  per_person_amount_g?: number | null;
  total_price_cents?: number | null;
  responsible_member_id?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripFoodMeal {
  id: string;
  itinerary_day_id: string;
  meal_key: "breakfast" | "lunch" | "dinner" | string;
  meal_type?: string | null;
  skipped: boolean;
  dish_name?: string | null;
  responsible_member_id?: string | null;
  notes?: string | null;
  items: TripFoodItem[];
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripFoodSupply {
  id: string;
  name: string;
  supply_type?: string | null;
  amount_g?: number | null;
  per_person_amount_g?: number | null;
  total_price_cents?: number | null;
  responsible_member_id?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripMedicalItem {
  id: string;
  name: string;
  item_type?: string | null;
  scope?: string | null;
  suggested_quantity?: number | null;
  required_quantity: number;
  packed_quantity: number;
  responsible_member_id?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripSegmentAssignment {
  id: string;
  route_segment_id?: string | null;
  checkpoint?: string | null;
  leader_record_member_id?: string | null;
  navigator_safety_member_id?: string | null;
  collaborator_member_id?: string | null;
  photographer_member_id?: string | null;
  safety_member_id?: string | null;
  environment_member_id?: string | null;
  sweeper_member_id?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripSafetyRisk {
  id: string;
  risk_type: string;
  prevention?: string | null;
  response?: string | null;
  responsible_member_id?: string | null;
  itinerary_day_id?: string | null;
  route_segment_id?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripRescueContact {
  id: string;
  organization: string;
  address?: string | null;
  phone?: string | null;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripBudgetItem {
  id: string;
  category?: string | null;
  name: string;
  quantity: number;
  unit_price_cents?: number | null;
  total_price_cents?: number | null;
  split_member_count?: number | null;
  notes?: string | null;
  linked_shared_gear_id?: string | null;
  linked_shared_gear_deleted: boolean;
  linked_shared_gear_name?: string | null;
  linked_shared_gear_responsible_member_id?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripGoalItem {
  id: string;
  scope: "team" | "member" | string;
  member_id?: string | null;
  content: string;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripMemberGearWeightSummary {
  member_id: string;
  all_weight_g: number;
  actual_weight_g: number;
}

export interface TripMemberGearViewItem {
  id: string;
  source: "personal" | "shared" | string;
  name: string;
  category: GearCategory;
  category_label: string;
  planned_quantity: number;
  packed_quantity: number;
  unit_weight_g?: number | null;
  labels: string[];
  counts_weight: boolean;
}

export interface TripMemberGearView {
  member_id: string;
  all_weight_g: number;
  actual_weight_g: number;
  items: TripMemberGearViewItem[];
}

export interface TripDetail {
  trip: Trip;
  sections: TripSectionKey[];
  my_member_id: string;
  members: TripMember[];
  personal_gear: TripPersonalGearItem[];
  shared_gear_demands: TripSharedGearDemand[];
  itinerary_days: TripItineraryDay[];
  route_segments: TripRouteSegment[];
  food_meals: TripFoodMeal[];
  food_supplies: TripFoodSupply[];
  medical_items: TripMedicalItem[];
  segment_assignments: TripSegmentAssignment[];
  safety_risks: TripSafetyRisk[];
  rescue_contacts: TripRescueContact[];
  budget_items: TripBudgetItem[];
  goals: TripGoalItem[];
  weight_summaries: TripMemberGearWeightSummary[];
  member_gear_views: TripMemberGearView[];
}

export interface TripInvitation {
  id: string;
  plan_id: string;
  token: string;
  created_by_user_id: string;
  revoked_at?: string | null;
  created_at: string;
}

export interface CreateTripInvitationResponse {
  invitation: TripInvitation;
}

export interface ImportTripPackingListRequest {
  packing_list_id: string;
}

export type TripRecordCreateRequest = Record<string, unknown> & {
  parent_id?: string | null;
  sort_order?: number;
};

export type TripRecordPatchRequest = Record<string, unknown> & TripPatchMeta;

export type TrailSourceFormat = "gpx" | "kml" | "fit";

export interface TrailPoint {
  lng: number;
  lat: number;
  elevation_m?: number | null;
  time?: string | null;
}

export interface TrailBounds {
  min_lng: number;
  min_lat: number;
  max_lng: number;
  max_lat: number;
}

export interface TrailSummary {
  id: string;
  owner_user_id: string;
  display_name: string;
  description?: string | null;
  source_format: TrailSourceFormat;
  original_filename: string;
  content_type: string;
  size_bytes: number;
  sha256_hex: string;
  bounds?: TrailBounds | null;
  distance_m: number;
  ascent_m: number;
  descent_m: number;
  min_elevation_m?: number | null;
  max_elevation_m?: number | null;
  start_elevation_m?: number | null;
  end_elevation_m?: number | null;
  start_time?: string | null;
  end_time?: string | null;
  point_count: number;
  created_at: string;
  updated_at: string;
}

export interface Trail extends TrailSummary {
  bucket: string;
  object_key: string;
  normalized_points: TrailPoint[];
  simplified_geojson: unknown;
  is_deleted: boolean;
}

export interface ListTrailsResponse {
  items: TrailSummary[];
}

export interface UpdateTrailRequest {
  display_name?: string;
  description?: string | null;
}

export interface TrailLink {
  trail_id: string;
  linked_by_user_id: string;
  role: "route" | string;
  sort_order: number;
  notes?: string | null;
  created_at: string;
  updated_at: string;
  trail: TrailSummary;
}

export interface MapTrailLink extends TrailLink {
  simplified_geojson: unknown;
}

export interface TripOverviewMapTrail extends MapTrailLink {
  trip_id: string;
  trip_title: string;
  trip_start_date?: string | null;
  trip_end_date?: string | null;
}

export interface TripsMapOverviewStats {
  trip_count: number;
  trail_count: number;
  rendered_point_count: number;
  total_distance_m: number;
  total_ascent_m: number;
  total_descent_m: number;
}

export interface TripsMapOverviewResponse {
  map: MapConfigResponse;
  trails: TripOverviewMapTrail[];
  bounds?: TrailBounds | null;
  stats: TripsMapOverviewStats;
  truncated: boolean;
}

export interface TrailLinkRequest {
  trail_id: string;
}

export interface MapAnnotation {
  id: string;
  owner_user_id: string;
  trail_id?: string | null;
  lng: number;
  lat: number;
  elevation_m?: number | null;
  trail_point_index?: number | null;
  annotation_type: string;
  title?: string | null;
  note?: string | null;
  field_versions: FieldVersions;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface MapAnnotationRequest {
  trail_id?: string | null;
  lng: number;
  lat: number;
  elevation_m?: number | null;
  trail_point_index?: number | null;
  annotation_type: string;
  title?: string | null;
  note?: string | null;
}

export type UpdateMapAnnotationRequest = Partial<
  Pick<
    MapAnnotationRequest,
    "annotation_type" | "title" | "note" | "elevation_m"
  >
> &
  TripPatchMeta;

export interface MapStyleOption {
  id: "outdoor" | "streets" | "satellite" | string;
  label: string;
  style_url: string;
}

export interface MapConfigResponse {
  provider: "maptiler" | string;
  style_url: string;
  public_key?: string | null;
  coordinate_system: "WGS84";
  enabled: boolean;
  styles?: MapStyleOption[];
  default_style_id?: string;
}

export interface TripMapStateResponse {
  map: MapConfigResponse;
  trails: MapTrailLink[];
  annotations: MapAnnotation[];
}

export interface OutdoorExperienceMapStateResponse {
  map: MapConfigResponse;
  trails: MapTrailLink[];
  annotations: MapAnnotation[];
}

export type GearAtlasStatus = "pending" | "approved" | "rejected";

export type GearAtlasSourceType = "manual" | "user_gear" | "external_import";

export type GearAtlasSort =
  | "approved_at_desc"
  | "name_asc"
  | "weight_desc"
  | "official_price_desc";

export interface GearAtlasPublicItem {
  id: string;
  category: GearCategory;
  category_label: string;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  variants?: GearVariant[] | null;
  specs?: GearSpecs | null;
  approved_at?: string | null;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface GearAtlasReviewChange {
  field: string;
  label: string;
  before?: string | null;
  after?: string | null;
}

export interface GearAtlasSubmission extends GearAtlasPublicItem {
  source_type: GearAtlasSourceType;
  source_user_gear_id?: string | null;
  source_name?: string | null;
  source_url?: string | null;
  source_rating_score?: number | null;
  source_rating_count?: number | null;
  status: GearAtlasStatus;
  rejection_reason?: string | null;
  review_changes?: GearAtlasReviewChange[] | null;
  reviewed_at?: string | null;
}

export interface CreateGearAtlasSubmissionRequest {
  category: GearCategory;
  name: string;
  brand?: string | null;
  model?: string | null;
  description?: string | null;
  weight_g?: number | null;
  official_price_cents?: number | null;
  official_price_currency?: GearCurrency | string | null;
  variants?: GearVariant[] | null;
  specs?: GearSpecs | null;
}

export type UpdateGearAtlasSubmissionRequest = CreateGearAtlasSubmissionRequest;

export interface ListGearAtlasRequest {
  category?: GearCategory;
  q?: string;
  sort?: GearAtlasSort;
  limit?: number;
  cursor?: string;
}

export interface ListGearAtlasResponse {
  items: GearAtlasPublicItem[];
  next_cursor?: string | null;
}

export interface ListGearAtlasSubmissionsRequest {
  status?: GearAtlasStatus;
  category?: GearCategory;
  deleted?: DeletedFilter;
  q?: string;
  limit?: number;
  cursor?: string;
}

export interface ListGearAtlasSubmissionsResponse {
  items: GearAtlasSubmission[];
  next_cursor?: string | null;
}

export interface RejectGearAtlasSubmissionRequest {
  reason: string;
}

export interface WechatLoginRequest {
  code: string;
  profile?: {
    nickname?: string | null;
    avatar_url?: string | null;
  };
}

export interface EmailVerificationCodeRequest {
  email: string;
}

export interface EmailVerificationCodeResponse {
  email: string;
  expires_at: string;
  debug_code?: string;
}

export interface EmailLoginCodeRequest {
  email: string;
}

export interface EmailLoginRequest {
  email: string;
  email_verification_code: string;
}

export interface PasswordResetCodeRequest {
  email: string;
}

export interface PasswordResetRequest {
  email: string;
  email_verification_code: string;
  password: string;
  confirm_password: string;
}

export interface BindEmailCodeRequest {
  email: string;
}

export interface BindEmailRequest {
  email: string;
  email_verification_code: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  confirm_password: string;
  email_verification_code: string;
}

export interface CaptchaChallengeRequest {
  account: string;
}

export interface CaptchaChallengeResponse {
  captcha_ticket: string;
  captcha_type: "image";
  image_svg: string;
  expires_at: string;
  debug_answer?: string;
}

export interface PasswordLoginRequest {
  account: string;
  password: string;
  captcha_ticket?: string | null;
  captcha_answer?: string | null;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface AuthUserResponse {
  id: string;
  username?: string | null;
  email?: string | null;
  nickname?: string | null;
  avatar_url?: string | null;
}

export interface WechatLoginResponse {
  access_token: string;
  expires_at: string;
  refresh_token: string;
  refresh_expires_at: string;
  user: AuthUserResponse;
}

export type LoginResponse = WechatLoginResponse;

export interface BindEmailResponse {
  user: AuthUserResponse;
}

export interface ProfileUserResponse {
  user: AuthUserResponse;
}

export interface OutdoorExperience {
  id: string;
  user_id: string;
  source_trip_id?: string | null;
  trip_type: TripType;
  title: string;
  start_date?: string | null;
  end_date?: string | null;
  day_count?: number | null;
  companion_count?: number | null;
  route_summary?: string | null;
  gear_summary?: string | null;
  food_summary?: string | null;
  budget_summary?: string | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
}

export type OutdoorExperienceRequest = Pick<OutdoorExperience, "title"> &
  Partial<
    Pick<
      OutdoorExperience,
      | "start_date"
      | "end_date"
      | "day_count"
      | "companion_count"
      | "route_summary"
      | "gear_summary"
      | "food_summary"
      | "budget_summary"
      | "notes"
    >
  >;

export interface ListOutdoorExperiencesResponse {
  items: OutdoorExperience[];
}

export interface OutdoorProfile {
  user_id: string;
  outdoor_id?: string | null;
  real_name?: string | null;
  gender?: string | null;
  birth_date?: string | null;
  height_cm?: number | null;
  phone?: string | null;
  emergency_contact?: string | null;
  emergency_contact_relationship?: string | null;
  emergency_phone?: string | null;
  blood_type?: string | null;
  medical_history?: string | null;
  allergy_history?: string | null;
  medical_response_note?: string | null;
  diet_preference?: string | null;
  insurance_policy_no?: string | null;
  insurance_company_phone?: string | null;
  experience_note?: string | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export type UpdateOutdoorProfileRequest = Partial<
  Omit<OutdoorProfile, "user_id" | "created_at" | "updated_at">
>;

export interface OutdoorProfileResponse {
  profile: OutdoorProfile;
}

export interface ImportGearsRequest {
  dry_run?: boolean;
  items: CreateGearRequest[];
}

export interface ImportGearError {
  row: number;
  field: string;
  message: string;
}

export interface ImportGearsResponse {
  created_count: number;
  updated_count: number;
  failed_count: number;
  errors: ImportGearError[];
}
