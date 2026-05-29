import type { GearCategory } from "./gear-utils";

const TRIP_INVITATION_TOKEN_PATTERN =
  /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i;

export interface SharedGearDemandTemplate {
  template_key: string;
  demand_name: string;
  group_label: string;
  category: GearCategory;
  category_label: string;
  planned_quantity: number;
  sort_order: number;
  slot_key: string;
  slot_name: string;
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

export interface TripConflictResponse {
  code: "edit_conflict";
  message: string;
  conflicts: Array<{
    field: string;
    client_value: unknown;
    server_value: unknown;
    server_version: number;
  }>;
}

export type TripType = "solo" | "team";
export type TripTimeBucket = "ongoing" | "upcoming" | "past" | "undated";

export interface TripReadiness {
  missing_count: number;
  missing_labels: string[];
  completion_percent: number;
}

export interface TripSummary {
  id: string;
  owner_user_id: string;
  trip_type: TripType;
  title: string;
  name: string;
  description?: string | null;
  start_date?: string | null;
  end_date?: string | null;
  enabled_sections: TripSectionKey[];
  route_use_slope_adjustment: boolean;
  route_use_high_altitude_adjustment: boolean;
  route_start_altitude_m?: number | null;
  day_count: number;
  itinerary_day_count: number;
  time_bucket: TripTimeBucket;
  days_until_start?: number | null;
  days_until_end?: number | null;
  member_count: number;
  readiness: TripReadiness;
  outdoor_experience_id?: string | null;
  field_versions: FieldVersions;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface ListTripsResponse {
  items: TripSummary[];
  next_cursor?: string | null;
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

export type TripHomeHighlightStatus = "ongoing" | "upcoming";

export interface TripHomeHighlightItem {
  trip: TripSummary;
  plan: TripSummary;
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

export type UpdateTripRequest = Partial<CreateTripRequest> & {
  base_field_versions?: FieldVersions;
  force_fields?: string[];
};

export interface UpdateTripSectionsRequest {
  enabled_sections: TripSectionKey[];
  base_field_versions?: FieldVersions;
  force_fields?: string[];
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
  trip_id: string;
  plan_id?: string;
  user_id: string;
  is_owner: boolean;
  profile: TripMemberProfile;
  field_versions: FieldVersions;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

interface TripGearSnapshotBase {
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
  slot_key?: string | null;
  slot_name?: string | null;
  concrete_name?: string | null;
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
  descent_profile: string;
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
  slot_key: string;
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
  meal_key: string;
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
  scope: string;
  member_id?: string | null;
  content: string;
  notes?: string | null;
  field_versions: FieldVersions;
  created_at: string;
  updated_at: string;
}

export interface TripMemberGearViewItem {
  id: string;
  source: string;
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
  trip: TripSummary;
  plan: TripSummary;
  sections: TripSectionKey[];
  my_member_id: string;
  members: TripMember[];
  personal_gear: TripPersonalGearItem[];
  personal_gear_items: TripPersonalGearItem[];
  shared_gear_demands: TripSharedGearDemand[];
  shared_gear_items: TripSharedGearDemand[];
  shared_gear_demand_templates: SharedGearDemandTemplate[];
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
  weight_summaries: Array<{
    member_id: string;
    all_weight_g: number;
    actual_weight_g: number;
  }>;
  gear_weight_summaries: Array<{
    member_id: string;
    all_weight_g: number;
    actual_weight_g: number;
  }>;
  member_gear_views: TripMemberGearView[];
}

export interface TripInvitation {
  id: string;
  trip_id: string;
  plan_id?: string;
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

export type TripRecordPatchRequest = Record<string, unknown> & {
  base_field_versions?: FieldVersions;
  force_fields?: string[];
};

export function formatTripDurationText(
  plan: Pick<
    TripSummary,
    "day_count" | "itinerary_day_count" | "start_date" | "end_date"
  >,
): string {
  const days =
    positiveIntegerOrNull(plan.day_count ?? plan.itinerary_day_count) ??
    inclusiveDateRangeDays(plan.start_date, plan.end_date);
  if (!days) {
    return "未设置天数";
  }
  if (days === 1) {
    return "1天";
  }
  return `${days}天${days - 1}夜`;
}

function positiveIntegerOrNull(
  value: number | null | undefined,
): number | null {
  if (!Number.isFinite(value) || value === undefined || value === null) {
    return null;
  }
  const days = Math.floor(value);
  return days > 0 ? days : null;
}

function inclusiveDateRangeDays(
  startDate: string | null | undefined,
  endDate: string | null | undefined,
): number | null {
  const start = parseDateOnlyUtc(startDate);
  const end = parseDateOnlyUtc(endDate);
  if (start === null || end === null || end < start) {
    return null;
  }
  return Math.floor((end - start) / 86400000) + 1;
}

function parseDateOnlyUtc(value: string | null | undefined): number | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value || "");
  if (!match) {
    return null;
  }
  const year = Number(match[1]);
  const monthIndex = Number(match[2]) - 1;
  const day = Number(match[3]);
  const timestamp = Date.UTC(year, monthIndex, day);
  const date = new Date(timestamp);
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== monthIndex ||
    date.getUTCDate() !== day
  ) {
    return null;
  }
  return timestamp;
}

export const TRIP_SECTION_LABELS: Record<string, string> = {
  members: "成员信息",
  personal_gear: "个人装备",
  itinerary: "行程安排",
  shared_gear: "公共装备",
  food_plan: "食品计划",
  medical_kit: "医药包",
  safety_plan: "安全预案",
  rescue_info: "救援信息",
  budget: "财务预算",
  goals: "目标",
};

export const OPTIONAL_TRIP_SECTIONS = [
  "itinerary",
  "shared_gear",
  "food_plan",
  "medical_kit",
  "safety_plan",
  "rescue_info",
  "budget",
  "goals",
] as const;

export function formatTripWeight(weightG?: number | null): string {
  const value = Number(weightG ?? 0);
  if (value >= 1000) {
    return `${(value / 1000).toFixed(value >= 10_000 ? 1 : 2)} kg`;
  }
  return `${Math.round(value)} g`;
}

export function sectionLabel(section: string): string {
  return TRIP_SECTION_LABELS[section] ?? section;
}

export function extractTripInvitationToken(value?: string | null): string {
  const match = (value || "").match(TRIP_INVITATION_TOKEN_PATTERN);
  return match ? match[0].toLowerCase() : "";
}

export function buildTripJoinPath(token: string): string {
  return `/pages/trips/join/index?token=${encodeURIComponent(token)}`;
}

export function buildTripInvitationShareTitle(tripTitle: string): string {
  return `邀请你加入「${tripTitle || "多人行程"}」多人行程`;
}

export function buildTripInvitationText(
  tripTitle: string,
  token: string,
): string {
  return [
    buildTripInvitationShareTitle(tripTitle),
    `邀请口令：${token}`,
    "打开小程序「寻径星野」- 行程 - 加入，粘贴口令加入。",
  ].join("\n");
}
