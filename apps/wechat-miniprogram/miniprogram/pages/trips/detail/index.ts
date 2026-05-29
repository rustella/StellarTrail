import { getThemeViewData, syncPageTheme } from "../../../utils/theme";
import {
  consumeOfflineCacheNotice,
  createTripBudgetItem,
  createTripFoodItem,
  createTripFoodSupply,
  createTripGoalItem,
  createTripPersonalGearItem,
  createTripItineraryDay,
  createTripItineraryTimeSlot,
  createTripMedicalItem,
  createTripRescueContact,
  createTripRouteSegment,
  createTripSafetyRisk,
  createTripSharedGearDemand,
  deleteTripBudgetItem,
  deleteTripFoodItem,
  deleteTripFoodSupply,
  deleteTripGoalItem,
  deleteTripPersonalGearItem,
  deleteTripItineraryDay,
  deleteTripMedicalItem,
  deleteTripRouteSegment,
  deleteTripRescueContact,
  deleteTripSafetyRisk,
  deleteTripSharedGearDemand,
  createTripInvitation,
  deleteTrip,
  getErrorMessage,
  getTrip,
  hasAccessToken,
  importTripPackingList,
  isLoginRequiredError,
  removeTripMember,
  updateTripFoodMeal,
  updateTripFoodItem,
  updateTripFoodSupply,
  updateTripItineraryDay,
  updateTripMedicalItem,
  updateTripSections,
  updateTripRouteSegment,
  updateTrip,
  updateTripSharedGearDemand,
  updateTripMember,
} from "../../../utils/api-trips";
import { listGearPackingLists, listGears } from "../../../utils/api-gears";
import {
  getOutdoorProfile,
  type OutdoorProfile,
} from "../../../utils/api-profile";
import {
  formatGearQuantity,
  formatGearWeight,
  formatPackingMeta,
  formatPackingProgress,
  GEAR_CATEGORY_OPTIONS,
  GEAR_STATUS_FILTER_OPTIONS,
  getGearCategoryLabel,
  getGearStatusLabel,
  type GearCategory,
  type GearPackingListSummary,
  type GearSummary,
} from "../../../utils/gear-utils";
import {
  OPTIONAL_TRIP_SECTIONS,
  buildTripInvitationShareTitle,
  buildTripInvitationText,
  buildTripJoinPath,
  formatTripWeight,
  sectionLabel,
  type FieldVersions,
  type TripFoodItem,
  type TripFoodMeal,
  type TripFoodSupply,
  type TripItineraryDay,
  type TripGoalItem,
  type TripMemberGearView,
  type TripMedicalItem,
  type TripPersonalGearItem,
  type TripSectionKey,
  type TripRecordPatchRequest,
  type TripBudgetItem,
  type TripRescueContact,
  type TripRouteSegment,
  type TripSafetyRisk,
  type TripSharedGearDemand,
  type SharedGearDemandTemplate,
  type TripMember,
  type TripDetail,
  type TripType,
} from "../../../utils/trip-utils";
import {
  getDefaultLoginPrompt,
  hideLoginPrompt,
  openLoginPageFromPrompt,
  showLoginPrompt,
} from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { calculateAge } from "../../../utils/date-utils";

interface SectionTab {
  key: TripSectionKey;
  label: string;
  id: string;
}

interface OptionalSectionView {
  key: TripSectionKey;
  label: string;
  enabled: boolean;
  actionText: string;
  canMoveUp: boolean;
  canMoveDown: boolean;
}

interface MemberView extends TripMember {
  displayName: string;
  roleText: string;
  contactText: string;
  profileSummaryText: string;
  healthText: string;
  canEdit: boolean;
  canDelete: boolean;
  isMine: boolean;
  personalWeightText: string;
  sharedWeightText: string;
  showFoodWeight: boolean;
  foodWeightText: string;
  totalWeightText: string;
  totalWeightG: number;
  hasCarrySummary: boolean;
}

interface SharedGearView {
  id: string;
  recordId: string;
  name: string;
  slotKey: string;
  slotName: string;
  slotGroup: string;
  concreteName: string;
  sourceGearId: string;
  category: GearCategory;
  categoryLabel: string;
  plannedQuantity: number;
  plannedQuantityText: string;
  responsibleText: string;
  responsibleMemberId: string;
  responsibleIndex: number;
  sourceText: string;
  createdByText: string;
  weightText: string;
  hasConcreteGear: boolean;
  actionText: string;
  isDefaultSlot: boolean;
  isPlaceholder: boolean;
  fieldVersions: FieldVersions;
}

type SharedGearCategoryFilterValue = "all" | GearCategory;

interface SharedGearCategoryOptionView {
  value: SharedGearCategoryFilterValue;
  label: string;
  count: number;
  selected: boolean;
}

interface RouteSegmentView {
  id: string;
  dayId: string;
  slotId: string;
  name: string;
  metricChips: string[];
  detailText: string;
  estimateText: string;
  altitudeText: string;
  safetyText: string;
}

interface ItineraryDayView {
  id: string;
  dayIndex: number;
  title: string;
  dateText: string;
  estimateText: string;
  routeSegments: RouteSegmentView[];
}

interface FoodItemView extends TripFoodItem {
  weightText: string;
  costText: string;
  responsibleText: string;
  notesText: string;
}

interface FoodMealView extends Omit<
  TripFoodMeal,
  "items" | "responsible_member_id"
> {
  label: string;
  summaryText: string;
  dishText: string;
  responsibleMemberId: string;
  responsibleIndex: number;
  responsibleText: string;
  itemCountText: string;
  weightText: string;
  notesText: string;
  items: FoodItemView[];
}

interface FoodDayView {
  id: string;
  dayIndex: number;
  title: string;
  dateText: string;
  summaryText: string;
  meals: FoodMealView[];
}

interface FoodSupplyView extends TripFoodSupply {
  typeText: string;
  weightText: string;
  costText: string;
  responsibleMemberId: string;
  responsibleIndex: number;
  responsibleText: string;
  notesText: string;
}

interface FoodBudgetSummaryView {
  visible: boolean;
  totalCents: number;
  totalText: string;
  itemCountText: string;
  pendingText: string;
}

interface MedicalItemView extends TripMedicalItem {
  typeText: string;
  scopeText: string;
  scopeClass: string;
  quantityText: string;
  statusText: string;
  statusClass: string;
  suggestedText: string;
}

interface MedicalSummaryView {
  hasItems: boolean;
  itemCountText: string;
  requiredText: string;
  packedText: string;
  shortageText: string;
}

interface GoalItemView extends TripGoalItem {
  titleText: string;
}

interface MedicalFormData {
  name: string;
  itemType: string;
  scope: string;
  suggestedQuantity: string;
  requiredQuantity: string;
  packedQuantity: string;
}

interface PackingListChoice extends GearPackingListSummary {
  metaText: string;
  progressText: string;
  weightText: string;
}

interface MyGearImportCategoryOption {
  id: "all" | GearCategory;
  label: string;
}

interface MyGearImportChoice extends GearSummary {
  categoryText: string;
  statusText: string;
  weightText: string;
  quantityText: string;
  brandModelText: string;
}

interface PlanGearFormData {
  name: string;
  category: GearCategory;
  brand: string;
  model: string;
  plannedQuantity: string;
  unitWeightG: string;
  notes: string;
}

type GearFormTarget = "personal" | "shared";

interface SharedSlotFormData {
  name: string;
  category: GearCategory;
  plannedQuantity: string;
}

interface MemberEditorFormData {
  displayName: string;
  outdoorId: string;
  realName: string;
  gender: string;
  age: string;
  heightCm: string;
  phone: string;
  emergencyContact: string;
  emergencyContactRelationship: string;
  emergencyPhone: string;
  bloodType: string;
  medicalHistory: string;
  allergyHistory: string;
  medicalResponseNote: string;
  dietPreference: string;
  insurancePolicyNo: string;
  insuranceCompanyPhone: string;
  experienceNote: string;
  roleLabel: string;
}

interface DayInfoFormData {
  weather: string;
  highTemperatureC: string;
  lowTemperatureC: string;
  weatherSummary: string;
  weatherNotes: string;
  campName: string;
  campAltitudeM: string;
  campTerrain: string;
  campSlope: string;
  campArea: string;
  campWaterSource: string;
  campNotes: string;
}

interface FoodMealFormData {
  mealType: string;
  dishName: string;
  responsibleIndex: number;
  notes: string;
}

interface FoodItemFormData {
  name: string;
  amountG: string;
  costYuan: string;
  responsibleIndex: number;
  notes: string;
}

interface FoodSupplyFormData {
  name: string;
  supplyType: string;
  amountG: string;
  costYuan: string;
  responsibleIndex: number;
  notes: string;
}

interface MemberRoleOptionView {
  label: string;
  selected: boolean;
}

type RouteEstimateRuleKey = "naismith" | "slope" | "altitude";

interface RouteEstimateRuleView {
  key: RouteEstimateRuleKey;
  title: string;
  desc: string;
  enabled: boolean;
  locked: boolean;
  actionText: string;
}

const DEFAULT_PLAN_GEAR_CATEGORY = "other_gear" as GearCategory;
const MEMBER_GENDER_OPTIONS = ["未填写", "男", "女", "其他"];
const MEMBER_BLOOD_TYPE_OPTIONS = ["未填写", "A", "B", "AB", "O", "其他"];
const MEMBER_ROLE_OPTIONS = [
  "领队",
  "副领队",
  "收队",
  "协作队员",
  "导航",
  "摄影",
  "医疗",
  "财务",
  "装备管理",
  "记录",
  "后勤",
];
const DEFAULT_PLAN_GEAR_CATEGORY_INDEX = Math.max(
  0,
  GEAR_CATEGORY_OPTIONS.findIndex(
    (item) => item.value === DEFAULT_PLAN_GEAR_CATEGORY,
  ),
);
const MY_GEAR_IMPORT_CATEGORY_OPTIONS: MyGearImportCategoryOption[] = [
  { id: "all", label: "全部分类" },
  ...GEAR_CATEGORY_OPTIONS.map((item) => ({
    id: item.value,
    label: item.label,
  })),
];
const MY_GEAR_IMPORT_DEFAULT_STATUS_INDEX = Math.max(
  0,
  GEAR_STATUS_FILTER_OPTIONS.findIndex((item) => item.value === "available"),
);
const TEAM_DEFAULT_SECTIONS: TripSectionKey[] = ["members", "personal_gear"];
const SOLO_DEFAULT_SECTIONS: TripSectionKey[] = ["personal_gear"];
const TEAM_OPTIONAL_SECTIONS: TripSectionKey[] = [...OPTIONAL_TRIP_SECTIONS];
const SOLO_OPTIONAL_SECTIONS: TripSectionKey[] = OPTIONAL_TRIP_SECTIONS.filter(
  (section) => section !== "shared_gear",
);
const TRIP_HOME_SHOULD_REFRESH_KEY = "stellartrail_home_trip_refresh";
const SECTION_NAV_ORDER: TripSectionKey[] = [
  "members",
  "personal_gear",
  ...OPTIONAL_TRIP_SECTIONS,
];
const ROUTE_ESTIMATE_RULES: Array<
  Pick<RouteEstimateRuleView, "key" | "title" | "desc" | "locked">
> = [
  {
    key: "naismith",
    title: "基础 Naismith",
    desc: "默认启用：距离 5 km/h，爬升 600 m/h。",
    locked: true,
  },
  {
    key: "slope",
    title: "坡度修正",
    desc: "按坡度分档，陡上陡下会额外耗时。",
    locked: false,
  },
  {
    key: "altitude",
    title: "高海拔修正",
    desc: "按计划起点海拔和路段累计海拔叠加耗时。",
    locked: false,
  },
];
const ROUTE_ESTIMATE_RULE_DETAIL: Record<RouteEstimateRuleKey, string> = {
  naismith:
    "基础 Naismith 始终启用：平路距离按 5 km/h 估算，爬升按 600 m/h 估算，下降默认不额外增加耗时。",
  slope:
    "坡度修正开启后，系统按平均坡度分档估算：普通坡度接近基础 Naismith；急上坡会降低爬升速度；急下坡会按下降强度额外计时。8%-15% 按中等坡度处理，15%-25% 按陡坡处理，25% 以上按急坡处理。",
  altitude:
    "高海拔修正开启后，需要填写计划起点海拔。系统按行程顺序累计每段爬升和下降，估算每段最高海拔：2500m 以上 +10%，3500m 以上 +20%，4500m 以上 +35%。",
};
const FOOD_MEAL_LABELS: Record<string, string> = {
  breakfast: "早餐",
  lunch: "午餐",
  dinner: "晚餐",
};
const FOOD_MEAL_ORDER: Record<string, number> = {
  breakfast: 0,
  lunch: 1,
  dinner: 2,
};
const MEDICAL_SCOPE_OPTIONS = [
  {
    value: "public_first_aid",
    label: "公共急救用品",
    shortLabel: "公共急救",
  },
  {
    value: "personal_reminder",
    label: "个人药品提醒",
    shortLabel: "个人提醒",
  },
] as const;

Page({
  data: {
    id: "",
    detail: null as TripDetail | null,
    planMetaText: "",
    canDeletePlan: false,
    selectedSection: "members" as TripSectionKey,
    activeSectionTabId: "",
    sectionTabs: [] as SectionTab[],
    optionalSections: [] as OptionalSectionView[],
    optionalSectionOrder: [] as TripSectionKey[],
    sectionSheetVisible: false,
    sectionActionLoadingKey: "" as TripSectionKey | "",
    inviteVisible: false,
    inviteToken: "",
    inviteText: "",
    invitePath: "",
    inviteShareTitle: "",
    personalGearActionSheetVisible: false,
    members: [] as MemberView[],
    myMember: null as MemberView | null,
    myGearView: null as TripMemberGearView | null,
    sharedGearEnabled: false,
    sharedGearRows: [] as SharedGearView[],
    sharedGearCategoryFilter: "all" as SharedGearCategoryFilterValue,
    sharedGearCategoryOptions: [] as SharedGearCategoryOptionView[],
    routeSegments: [] as RouteSegmentView[],
    orphanRouteSegments: [] as RouteSegmentView[],
    itineraryDays: [] as ItineraryDayView[],
    activeItineraryDayId: "",
    activeItineraryDayIndex: 0,
    activeItineraryDay: null as ItineraryDayView | null,
    itineraryDayCount: 0,
    itineraryDayPagerText: "",
    canSwitchPrevDay: false,
    canSwitchNextDay: false,
    routeEstimateSummaryText: "",
    routeEstimateDetailText: "",
    routeEstimateSheetVisible: false,
    routeEstimateRules: [] as RouteEstimateRuleView[],
    routeUseSlopeAdjustmentDraft: false,
    routeUseHighAltitudeAdjustmentDraft: false,
    routeStartAltitudeDraft: "",
    foodMeals: [] as TripFoodMeal[],
    foodSupplies: [] as TripFoodSupply[],
    foodDays: [] as FoodDayView[],
    activeFoodDayId: "",
    activeFoodDayIndex: 0,
    activeFoodDay: null as FoodDayView | null,
    foodDayPagerText: "",
    canSwitchPrevFoodDay: false,
    canSwitchNextFoodDay: false,
    foodSupplyRows: [] as FoodSupplyView[],
    medicalItems: [] as MedicalItemView[],
    medicalSummary: defaultMedicalSummary(),
    safetyRisks: [] as TripSafetyRisk[],
    rescueContacts: [] as TripRescueContact[],
    budgetItems: [] as TripBudgetItem[],
    foodBudgetSummary: defaultFoodBudgetSummary(),
    goals: [] as GoalItemView[],
    budgetTotalText: "¥0",
    budgetPerPersonText: "",
    budgetSharedGearLabels: ["不关联公共装备"] as string[],
    memberEditorVisible: false,
    memberEditorMember: null as MemberView | null,
    memberEditorForm: defaultMemberEditorForm(),
    memberGenderOptions: MEMBER_GENDER_OPTIONS,
    memberGenderIndex: 0,
    memberBloodTypeOptions: MEMBER_BLOOD_TYPE_OPTIONS,
    memberBloodTypeIndex: 0,
    memberRoleOptions: buildMemberRoleOptionViews(""),
    gearCategoryOptions: GEAR_CATEGORY_OPTIONS,
    gearCategoryLabels: GEAR_CATEGORY_OPTIONS.map((item) => item.label),
    importSheetVisible: false,
    importLists: [] as PackingListChoice[],
    importNextCursor: null as string | null,
    importLoading: false,
    importLoadingMore: false,
    importError: "",
    myGearImportSheetVisible: false,
    myGearImportItems: [] as MyGearImportChoice[],
    myGearImportNextCursor: null as string | null,
    myGearImportLoading: false,
    myGearImportLoadingMore: false,
    myGearImportError: "",
    myGearImportTarget: "personal" as GearFormTarget,
    myGearImportQ: "",
    myGearImportCategoryOptions: MY_GEAR_IMPORT_CATEGORY_OPTIONS,
    myGearImportCategoryLabels: MY_GEAR_IMPORT_CATEGORY_OPTIONS.map(
      (item) => item.label,
    ),
    myGearImportCategoryIndex: 0,
    myGearImportStatusOptions: GEAR_STATUS_FILTER_OPTIONS,
    myGearImportStatusLabels: GEAR_STATUS_FILTER_OPTIONS.map(
      (item) => item.label,
    ),
    myGearImportStatusIndex: MY_GEAR_IMPORT_DEFAULT_STATUS_INDEX,
    planGearFormVisible: false,
    planGearFormTarget: "personal" as GearFormTarget,
    planGearForm: defaultPlanGearForm(),
    planGearCategoryIndex: DEFAULT_PLAN_GEAR_CATEGORY_INDEX,
    sharedSlotSheetVisible: false,
    selectedSharedGearSlot: null as SharedGearView | null,
    sharedSlotPlannedQuantity: "1",
    sharedSlotResponsibleIndex: 0,
    sharedSlotFormVisible: false,
    sharedSlotForm: defaultSharedSlotForm(),
    sharedSlotCategoryIndex: DEFAULT_PLAN_GEAR_CATEGORY_INDEX,
    myRoleLabel: "",
    sharedGearName: "",
    sharedResponsibleIndex: 0,
    routeName: "",
    routeCheckpoint: "",
    routeDistanceKm: "5",
    routeAscentM: "300",
    routeDescentM: "0",
    routeTrailCondition: "",
    routeEditorVisible: false,
    routeEditorSegmentId: "",
    dayInfoEditorVisible: false,
    dayInfoEditorDay: null as ItineraryDayView | null,
    dayInfoForm: defaultDayInfoForm(),
    foodMealEditorVisible: false,
    foodMealEditorMeal: null as FoodMealView | null,
    foodMealForm: defaultFoodMealForm(),
    foodItemEditorVisible: false,
    foodItemEditorMeal: null as FoodMealView | null,
    foodItemEditorItem: null as FoodItemView | null,
    foodItemForm: defaultFoodItemForm(),
    foodSupplyEditorVisible: false,
    foodSupplyEditorItem: null as FoodSupplyView | null,
    foodSupplyForm: defaultFoodSupplyForm(),
    medicalEditorVisible: false,
    medicalEditorItem: null as MedicalItemView | null,
    medicalForm: defaultMedicalForm(),
    medicalScopeLabels: MEDICAL_SCOPE_OPTIONS.map((item) => item.label),
    medicalScopeIndex: 0,
    safetyRiskType: "",
    safetyPrevention: "",
    safetyResponse: "",
    safetyResponsibleIndex: 0,
    rescueOrganization: "",
    rescueAddress: "",
    rescuePhone: "",
    budgetEditorVisible: false,
    budgetCategory: "",
    budgetName: "",
    budgetQuantity: "1",
    budgetUnitPriceYuan: "",
    budgetSplitMemberCount: "",
    budgetLinkedSharedGearIndex: 0,
    goalEditorVisible: false,
    goalContent: "",
    foodBlocked: false,
    loading: false,
    mutating: false,
    error: "",
    offlineNotice: "",
    loginPrompt: getDefaultLoginPrompt(),
    ...getThemeViewData(),
  },

  itinerarySwipeStartPoint: null as { x: number; y: number } | null,

  onLoad(options: Record<string, string | undefined>) {
    const id = options.id || "";
    if (!id) {
      this.setData({ error: "没有找到这份组队计划书，请返回后重试" });
      return;
    }
    const selectedSection = normalizeInitialSection(options.section);
    this.setData({
      id,
      ...(selectedSection ? { selectedSection } : {}),
    });
    this.loadDetail();
  },

  onShow() {
    syncPageTheme(this);
    const shouldRefresh = wx.getStorageSync("stellartrail_trips_refresh");
    if (shouldRefresh && this.data.id) {
      wx.removeStorageSync("stellartrail_trips_refresh");
      this.loadDetail();
      return;
    }
    if (!hasAccessToken()) {
      this.setData({ detail: null });
    }
  },

  onPullDownRefresh() {
    this.loadDetail().finally(() => wx.stopPullDownRefresh());
  },

  async loadDetail() {
    if (!this.data.id) {
      return;
    }
    if (!hasAccessToken()) {
      showLoginPrompt(this, {
        message: "登录后可以查看组队计划书。",
        redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}`,
      });
      return;
    }
    this.setData({ loading: true, error: "" });
    try {
      const detail = await getTrip(this.data.id);
      const offlineNotice = consumeOfflineCacheNotice();
      this.applyDetail(detail, offlineNotice);
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后查看组队计划。",
          redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  applyDetail(detail: TripDetail, offlineNotice = "") {
    const view = buildDetailView(
      detail,
      this.data.selectedSection,
      this.data.optionalSectionOrder,
      this.data.activeItineraryDayId,
      this.data.activeFoodDayId,
      this.data.sharedGearCategoryFilter,
    );
    this.setData({
      ...view,
      ...(offlineNotice ? { offlineNotice } : {}),
    });
    wx.setNavigationBarTitle({ title: detail.plan.name });
  },

  switchSection(event: WechatMiniprogram.BaseEvent) {
    const selectedSection = event.currentTarget.dataset.key as TripSectionKey;
    if (!selectedSection || selectedSection === this.data.selectedSection) {
      return;
    }
    this.setData({ selectedSection });
    if (this.data.detail) {
      this.applyDetail(this.data.detail);
    }
  },

  switchSharedGearCategory(event: WechatMiniprogram.BaseEvent) {
    const value = normalizeSharedGearCategoryFilter(
      event.currentTarget.dataset.value,
    );
    if (value === this.data.sharedGearCategoryFilter) {
      return;
    }
    this.setData({ sharedGearCategoryFilter: value });
    if (this.data.detail) {
      this.applyDetail(this.data.detail);
    }
  },

  openPersonalGearActionSheet() {
    this.setData({ personalGearActionSheetVisible: true });
  },

  closePersonalGearActionSheet() {
    this.setData({ personalGearActionSheetVisible: false });
  },

  goEditPlan() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    wx.navigateTo({
      url: `/pages/trips/form/index?id=${encodeURIComponent(this.data.id)}`,
    });
  },

  confirmDeletePlan() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (!this.data.detail || !this.data.canDeletePlan || this.data.mutating) {
      return;
    }
    wx.showModal({
      title: "删除组队计划书？",
      content: "删除后队员将无法继续查看这份计划。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteCurrentPlan();
        }
      },
    });
  },

  async deleteCurrentPlan() {
    const detail = this.data.detail;
    if (!detail || !this.data.canDeletePlan) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({ mutating: true });
    try {
      await deleteTrip(detail.plan.id);
      wx.setStorageSync("stellartrail_trips_refresh", true);
      wx.setStorageSync(TRIP_HOME_SHOULD_REFRESH_KEY, true);
      wx.showToast({ title: "已删除", icon: "success" });
      wx.switchTab({ url: "/pages/trips/index" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后删除计划。",
          redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      wx.showToast({ title: getErrorMessage(error), icon: "none" });
    } finally {
      this.setData({ mutating: false });
    }
  },

  openSectionSheet() {
    this.setData({ sectionSheetVisible: true });
  },

  closeSectionSheet() {
    this.setData({ sectionSheetVisible: false });
  },

  openRouteEstimateSheet() {
    const plan = this.data.detail?.plan;
    if (!plan || this.data.mutating) {
      return;
    }
    this.setData({
      routeEstimateSheetVisible: true,
      routeUseSlopeAdjustmentDraft: plan.route_use_slope_adjustment,
      routeUseHighAltitudeAdjustmentDraft:
        plan.route_use_high_altitude_adjustment,
      routeStartAltitudeDraft:
        plan.route_start_altitude_m == null
          ? ""
          : String(plan.route_start_altitude_m),
      routeEstimateRules: buildRouteEstimateRuleViews(
        plan.route_use_slope_adjustment,
        plan.route_use_high_altitude_adjustment,
      ),
    });
  },

  closeRouteEstimateSheet() {
    if (this.data.mutating) {
      return;
    }
    this.setData({ routeEstimateSheetVisible: false });
  },

  toggleRouteEstimateRule(event: WechatMiniprogram.BaseEvent) {
    const key = event.currentTarget.dataset.key as
      | RouteEstimateRuleKey
      | undefined;
    if (!key || key === "naismith" || this.data.mutating) {
      return;
    }
    const nextSlope =
      key === "slope"
        ? !this.data.routeUseSlopeAdjustmentDraft
        : this.data.routeUseSlopeAdjustmentDraft;
    const nextAltitude =
      key === "altitude"
        ? !this.data.routeUseHighAltitudeAdjustmentDraft
        : this.data.routeUseHighAltitudeAdjustmentDraft;
    this.setData({
      routeUseSlopeAdjustmentDraft: nextSlope,
      routeUseHighAltitudeAdjustmentDraft: nextAltitude,
      routeEstimateRules: buildRouteEstimateRuleViews(nextSlope, nextAltitude),
    });
  },

  showRouteEstimateRuleInfo(event: WechatMiniprogram.BaseEvent) {
    const key = event.currentTarget.dataset.key as
      | RouteEstimateRuleKey
      | undefined;
    const rule = ROUTE_ESTIMATE_RULES.find((item) => item.key === key);
    if (!rule || !key) {
      return;
    }
    wx.showModal({
      title: rule.title,
      content: ROUTE_ESTIMATE_RULE_DETAIL[key],
      showCancel: false,
      confirmText: "知道了",
    });
  },

  onRouteStartAltitudeInput(event: WechatMiniprogram.Input) {
    this.setData({ routeStartAltitudeDraft: event.detail.value });
  },

  async saveRouteEstimateSettings() {
    const detail = this.data.detail;
    if (!detail || this.data.mutating) {
      return;
    }
    if (this.data.routeUseHighAltitudeAdjustmentDraft) {
      const altitudeText = this.data.routeStartAltitudeDraft.trim();
      const altitude = Number(altitudeText);
      if (
        !altitudeText ||
        !Number.isFinite(altitude) ||
        altitude < -500 ||
        altitude > 9000
      ) {
        wx.showToast({
          title: "请填写 -500 到 9000 米之间的起点海拔",
          icon: "none",
        });
        return;
      }
    }
    const routeStartAltitudeM = this.data.routeUseHighAltitudeAdjustmentDraft
      ? Math.round(Number(this.data.routeStartAltitudeDraft.trim()))
      : null;
    await this.runMutation(async () => {
      const next = await updateTrip(this.data.id, {
        route_use_slope_adjustment: this.data.routeUseSlopeAdjustmentDraft,
        route_use_high_altitude_adjustment:
          this.data.routeUseHighAltitudeAdjustmentDraft,
        route_start_altitude_m: routeStartAltitudeM,
        base_field_versions: {
          route_use_slope_adjustment:
            detail.plan.field_versions.route_use_slope_adjustment ?? 0,
          route_use_high_altitude_adjustment:
            detail.plan.field_versions.route_use_high_altitude_adjustment ?? 0,
          route_start_altitude_m:
            detail.plan.field_versions.route_start_altitude_m ?? 0,
        },
      });
      this.setData({ routeEstimateSheetVisible: false });
      this.applyDetail(next);
    });
  },

  stopTap() {
    // Used by bottom sheets to keep inner taps from closing the sheet.
  },

  selectOptionalSection(event: WechatMiniprogram.BaseEvent) {
    if (this.data.mutating) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const section = event.currentTarget.dataset.key as TripSectionKey;
    const option = this.data.optionalSections.find(
      (item) => item.key === section,
    );
    if (!option) {
      return;
    }
    if (!option.enabled) {
      void this.updateOptionalSection(section, true, false);
      return;
    }
    wx.showModal({
      title: `隐藏${option.label}`,
      content: "隐藏后已填写内容会保留，需要时可以重新添加回来。",
      confirmText: "隐藏",
      confirmColor: "#0f766e",
      success: (result) => {
        if (result.confirm) {
          void this.updateOptionalSection(section, false, false);
        }
      },
    });
  },

  async moveOptionalSection(event: WechatMiniprogram.BaseEvent) {
    if (this.data.mutating) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const detail = this.data.detail;
    const section = event.currentTarget.dataset.key as TripSectionKey;
    const direction = Number(event.currentTarget.dataset.direction || 0);
    if (!detail || !section || !direction) {
      return;
    }
    const tripType = detail.plan.trip_type;
    const currentOrder = normalizeOptionalSectionOrder(
      detail.plan.enabled_sections,
      this.data.optionalSectionOrder,
      tripType,
    );
    const currentIndex = currentOrder.indexOf(section);
    const nextIndex = currentIndex + direction;
    if (currentIndex < 0 || nextIndex < 0 || nextIndex >= currentOrder.length) {
      return;
    }
    const nextOrder = [...currentOrder];
    [nextOrder[currentIndex], nextOrder[nextIndex]] = [
      nextOrder[nextIndex],
      nextOrder[currentIndex],
    ];
    const enabled = new Set(
      sanitizeEnabledSectionsForTripType(
        detail.plan.enabled_sections,
        tripType,
      ),
    );
    defaultSectionsForTripType(tripType).forEach((key) => enabled.add(key));
    const currentPayload = buildEnabledSectionsPayload(
      enabled,
      currentOrder,
      tripType,
    );
    const nextPayload = buildEnabledSectionsPayload(
      enabled,
      nextOrder,
      tripType,
    );
    if (sameSectionOrder(currentPayload, nextPayload)) {
      this.setData({ optionalSectionOrder: nextOrder });
      if (this.data.detail) {
        this.applyDetail(this.data.detail);
      }
      return;
    }
    this.setData({ sectionActionLoadingKey: section });
    try {
      await this.runMutation(async () => {
        const next = await updateTripSections(this.data.id, {
          enabled_sections: nextPayload,
          base_field_versions: {
            enabled_sections: detail.plan.field_versions.enabled_sections ?? 0,
          },
        });
        this.setData({
          optionalSectionOrder: nextOrder,
          sectionSheetVisible: true,
        });
        this.applyDetail(next);
      });
    } finally {
      this.setData({ sectionActionLoadingKey: "" });
    }
  },

  async updateOptionalSection(
    section: TripSectionKey,
    shouldEnable: boolean,
    shouldCloseSheet: boolean,
  ) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const detail = this.data.detail;
    if (!detail || !section) {
      return;
    }
    const tripType = detail.plan.trip_type;
    if (!isSectionAllowedForTripType(section, tripType)) {
      return;
    }
    const enabled = new Set(
      sanitizeEnabledSectionsForTripType(
        detail.plan.enabled_sections,
        tripType,
      ),
    );
    if (shouldEnable) {
      enabled.add(section);
      if (section === "food_plan") {
        enabled.add("itinerary");
      }
    } else {
      enabled.delete(section);
      if (section === "itinerary") {
        enabled.delete("food_plan");
      }
    }
    defaultSectionsForTripType(tripType).forEach((key) => enabled.add(key));
    const optionalOrder = normalizeOptionalSectionOrder(
      detail.plan.enabled_sections,
      this.data.optionalSectionOrder,
      tripType,
    );
    this.setData({ sectionActionLoadingKey: section });
    try {
      await this.runMutation(async () => {
        const next = await updateTripSections(this.data.id, {
          enabled_sections: buildEnabledSectionsPayload(
            enabled,
            optionalOrder,
            tripType,
          ),
          base_field_versions: {
            enabled_sections: detail.plan.field_versions.enabled_sections ?? 0,
          },
        });
        this.setData({
          selectedSection: pickNextSelectedSection(
            this.data.selectedSection,
            section,
            shouldEnable,
            enabled,
            tripType,
          ),
          sectionSheetVisible: !shouldCloseSheet,
          optionalSectionOrder: optionalOrder,
        });
        this.applyDetail(next);
      });
    } finally {
      this.setData({ sectionActionLoadingKey: "" });
    }
  },

  async createInvite() {
    await this.runMutation(async () => {
      const planName = this.data.detail?.plan.name || "组队计划书";
      const response = await createTripInvitation(this.data.id);
      const inviteToken = response.invitation.token;
      const inviteText = buildTripInvitationText(planName, inviteToken);
      this.setData({
        inviteVisible: true,
        inviteToken,
        inviteText,
        invitePath: buildTripJoinPath(inviteToken),
        inviteShareTitle: buildTripInvitationShareTitle(planName),
      });
      wx.setClipboardData({ data: inviteText });
    });
  },

  copyInviteToken() {
    if (!this.data.inviteToken) {
      return;
    }
    wx.setClipboardData({
      data: this.data.inviteToken,
      success: () => wx.showToast({ title: "邀请口令已复制", icon: "success" }),
    });
  },

  closeInvitePanel() {
    this.setData({ inviteVisible: false });
  },

  onShareAppMessage() {
    return {
      title: this.data.inviteShareTitle || "邀请你加入组队计划书",
      path: this.data.invitePath || "/pages/trips/index",
    };
  },

  onInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as string;
    this.setData({ [field]: event.detail.value });
  },

  onResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({ sharedResponsibleIndex: Number(event.detail.value) || 0 });
  },

  openMemberEditor(event: WechatMiniprogram.BaseEvent) {
    const memberId = event.currentTarget.dataset.id as string | undefined;
    const member = this.data.members.find((item) => item.id === memberId);
    if (!member || !member.canEdit || this.data.mutating) {
      return;
    }
    const form = buildMemberEditorForm(member);
    this.setData({
      memberEditorVisible: true,
      memberEditorMember: member,
      memberEditorForm: form,
      memberGenderIndex: optionIndex(MEMBER_GENDER_OPTIONS, form.gender),
      memberBloodTypeIndex: optionIndex(
        MEMBER_BLOOD_TYPE_OPTIONS,
        form.bloodType,
      ),
      memberRoleOptions: buildMemberRoleOptionViews(form.roleLabel),
    });
  },

  confirmRemoveMember(event: WechatMiniprogram.BaseEvent) {
    const memberId = event.currentTarget.dataset.id as string | undefined;
    const member = this.data.members.find((item) => item.id === memberId);
    if (!member || !member.canDelete || this.data.mutating) {
      return;
    }
    wx.showModal({
      title: "删除成员",
      content: `确定删除「${member.displayName}」吗？会从本计划移除该成员，已填写的计划内容不会自动清空。`,
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.removeMember(member.id);
        }
      },
    });
  },

  async removeMember(memberId: string) {
    await this.runMutation(async () => {
      const next = await removeTripMember(this.data.id, memberId);
      this.applyDetail(next);
      wx.showToast({ title: "已移除成员", icon: "success" });
    });
  },

  closeMemberEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      memberEditorVisible: false,
      memberEditorMember: null,
      memberEditorForm: defaultMemberEditorForm(),
      memberGenderIndex: 0,
      memberBloodTypeIndex: 0,
      memberRoleOptions: buildMemberRoleOptionViews(""),
    });
  },

  onMemberEditorInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | keyof MemberEditorFormData
      | undefined;
    if (!field) {
      return;
    }
    if (field === "roleLabel") {
      const roleLabel = event.detail.value;
      this.setData({
        "memberEditorForm.roleLabel": roleLabel,
        memberRoleOptions: buildMemberRoleOptionViews(roleLabel),
      });
      return;
    }
    this.setData({ [`memberEditorForm.${field}`]: event.detail.value });
  },

  toggleMemberRoleOption(event: WechatMiniprogram.BaseEvent) {
    const label = event.currentTarget.dataset.label as string | undefined;
    if (!label) {
      return;
    }
    const roleLabel = toggleRoleLabel(
      this.data.memberEditorForm.roleLabel,
      label,
    );
    this.setData({
      "memberEditorForm.roleLabel": roleLabel,
      memberRoleOptions: buildMemberRoleOptionViews(roleLabel),
    });
  },

  onMemberGenderChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value) || 0;
    this.setData({
      memberGenderIndex: index,
      "memberEditorForm.gender":
        index === 0 ? "" : MEMBER_GENDER_OPTIONS[index],
    });
  },

  onMemberBloodTypeChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value) || 0;
    this.setData({
      memberBloodTypeIndex: index,
      "memberEditorForm.bloodType":
        index === 0 ? "" : MEMBER_BLOOD_TYPE_OPTIONS[index],
    });
  },

  async saveMemberProfile() {
    const member = this.data.memberEditorMember;
    if (!member) {
      return;
    }
    const request = buildMemberUpdateRequest(
      this.data.memberEditorForm,
      member,
    );
    if (!request) {
      return;
    }
    await this.runMutation(async () => {
      const next = await updateTripMember(this.data.id, member.id, request);
      this.applyDetail(next);
      this.setData({
        memberEditorVisible: false,
        memberEditorMember: null,
        memberEditorForm: defaultMemberEditorForm(),
        memberRoleOptions: buildMemberRoleOptionViews(""),
      });
    });
  },

  async importMyOutdoorProfile() {
    const member = this.data.memberEditorMember;
    if (!member || !member.isMine) {
      return;
    }
    await this.runMutation(async () => {
      const outdoor = await getOutdoorProfile();
      const fields = outdoorProfileToMemberFields(outdoor.profile);
      if (!Object.keys(fields).length) {
        wx.showToast({ title: "我的资料还没有可导入内容", icon: "none" });
        return;
      }
      const next = await updateTripMember(this.data.id, member.id, {
        ...fields,
        base_field_versions: baseVersionsForFields(fields, member),
      });
      this.applyDetail(next);
      this.setData({
        memberEditorVisible: false,
        memberEditorMember: null,
        memberEditorForm: defaultMemberEditorForm(),
        memberRoleOptions: buildMemberRoleOptionViews(""),
      });
      wx.showToast({ title: "已导入我的资料", icon: "success" });
    });
  },

  async saveMyRole() {
    const myMember = this.data.myMember;
    if (!myMember) {
      return;
    }
    await this.runMutation(async () => {
      const next = await updateTripMember(this.data.id, myMember.id, {
        role_label: this.data.myRoleLabel.trim() || null,
        base_field_versions: {
          role_label: myMember.field_versions.role_label ?? 0,
        },
      });
      this.applyDetail(next);
    });
  },

  openImportSheet() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      personalGearActionSheetVisible: false,
      importSheetVisible: true,
    });
    this.loadImportLists(true);
  },

  closeImportSheet() {
    this.setData({ importSheetVisible: false });
  },

  loadMoreImportLists() {
    this.loadImportLists(false);
  },

  async loadImportLists(reset: boolean) {
    if (this.data.importLoading || this.data.importLoadingMore) {
      return;
    }
    const cursor = reset ? undefined : this.data.importNextCursor || undefined;
    if (!reset && !cursor) {
      return;
    }
    this.setData({
      importError: "",
      ...(reset ? { importLoading: true } : { importLoadingMore: true }),
    });
    try {
      const response = await listGearPackingLists({ limit: 20, cursor });
      const offlineNotice = consumeOfflineCacheNotice();
      const nextLists = response.items.map(mapPackingListChoice);
      this.setData({
        importLists: reset
          ? nextLists
          : this.data.importLists.concat(nextLists),
        importNextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后导入打包清单。",
          redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}&section=personal_gear`,
        });
        return;
      }
      this.setData({ importError: getErrorMessage(error) });
    } finally {
      this.setData({
        ...(reset ? { importLoading: false } : { importLoadingMore: false }),
      });
    }
  },

  async selectImportPackingList(event: WechatMiniprogram.BaseEvent) {
    const packingListId = event.currentTarget.dataset.id as string;
    if (!packingListId) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      const next = await importTripPackingList(this.data.id, {
        packing_list_id: packingListId,
      });
      this.setData({ importSheetVisible: false });
      this.applyDetail(next);
    });
  },

  openMyGearImportSheet() {
    this.openGearImportSheet("personal");
  },

  openSharedGearImportSheet() {
    if (!this.data.selectedSharedGearSlot) {
      wx.showToast({ title: "请先选择公共装备需求", icon: "none" });
      return;
    }
    this.openGearImportSheet("shared");
  },

  openGearImportSheet(target: GearFormTarget) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      personalGearActionSheetVisible: false,
      myGearImportSheetVisible: true,
      myGearImportTarget: target,
    });
    this.loadMyGearImportItems(true);
  },

  closeMyGearImportSheet() {
    this.setData({ myGearImportSheetVisible: false });
  },

  onMyGearImportSearchInput(event: WechatMiniprogram.Input) {
    this.setData({ myGearImportQ: event.detail.value });
  },

  submitMyGearImportSearch() {
    this.loadMyGearImportItems(true);
  },

  clearMyGearImportSearch() {
    this.setData({ myGearImportQ: "" });
    this.loadMyGearImportItems(true);
  },

  onMyGearImportCategoryChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      myGearImportCategoryIndex: Number(event.detail.value) || 0,
    });
    this.loadMyGearImportItems(true);
  },

  onMyGearImportStatusChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      myGearImportStatusIndex:
        Number(event.detail.value) || MY_GEAR_IMPORT_DEFAULT_STATUS_INDEX,
    });
    this.loadMyGearImportItems(true);
  },

  loadMoreMyGearImportItems() {
    this.loadMyGearImportItems(false);
  },

  async loadMyGearImportItems(reset: boolean) {
    if (this.data.myGearImportLoading || this.data.myGearImportLoadingMore) {
      return;
    }
    const cursor = reset
      ? undefined
      : this.data.myGearImportNextCursor || undefined;
    if (!reset && !cursor) {
      return;
    }
    this.setData({
      myGearImportError: "",
      ...(reset
        ? { myGearImportLoading: true }
        : { myGearImportLoadingMore: true }),
    });
    try {
      const response = await listGears(this.buildMyGearImportRequest(cursor));
      const offlineNotice = consumeOfflineCacheNotice();
      const items = response.items.map(mapMyGearImportChoice);
      this.setData({
        myGearImportItems: reset
          ? items
          : this.data.myGearImportItems.concat(items),
        myGearImportNextCursor: response.next_cursor ?? null,
        ...(offlineNotice ? { offlineNotice } : {}),
      });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后导入我的装备。",
          redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}&section=personal_gear`,
        });
        return;
      }
      this.setData({ myGearImportError: getErrorMessage(error) });
    } finally {
      this.setData({
        ...(reset
          ? { myGearImportLoading: false }
          : { myGearImportLoadingMore: false }),
      });
    }
  },

  buildMyGearImportRequest(cursor?: string) {
    const category =
      this.data.myGearImportCategoryOptions[this.data.myGearImportCategoryIndex]
        ?.id ?? "all";
    const status =
      this.data.myGearImportStatusOptions[this.data.myGearImportStatusIndex]
        ?.value ?? "available";
    return {
      category: category === "all" ? undefined : category,
      status: status || undefined,
      q: this.data.myGearImportQ.trim() || undefined,
      sort: "created_at_desc" as const,
      limit: 20,
      cursor,
    };
  },

  async importMyGearItem(event: WechatMiniprogram.BaseEvent) {
    const gearId = event.currentTarget.dataset.id as string | undefined;
    const gear = this.data.myGearImportItems.find((item) => item.id === gearId);
    if (!gear) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    if (this.data.myGearImportTarget === "shared" && !this.data.myMember) {
      wx.showToast({ title: "请先加入计划", icon: "none" });
      return;
    }
    const selectedSlot = this.data.selectedSharedGearSlot;
    if (this.data.myGearImportTarget === "shared" && !selectedSlot) {
      wx.showToast({ title: "请先选择公共装备需求", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const target = this.data.myGearImportTarget;
      const next =
        target === "shared" && selectedSlot
          ? await this.upsertSharedGearSlot(selectedSlot, {
              ...sharedGearSlotBasePayload(
                selectedSlot,
                this.data.sharedSlotPlannedQuantity,
                this.data.members[this.data.sharedSlotResponsibleIndex]?.id ||
                  selectedSlot.responsibleMemberId ||
                  this.data.myMember?.id ||
                  "",
              ),
              concrete_name: gear.name,
              source_gear_id: gear.id,
              source_member_id: this.data.myMember?.id ?? null,
              brand: gear.brand ?? null,
              model: gear.model ?? null,
              unit_weight_g: gear.weight_g ?? null,
              notes: null,
            })
          : await createTripPersonalGearItem(this.data.id, {
              source_gear_id: gear.id,
              ...gearSnapshotPayloadFromGear(gear),
            });
      this.applyDetail(next);
      this.setData({ myGearImportSheetVisible: false });
      wx.showToast({
        title: target === "shared" ? "已导入公共装备" : "已导入我的装备",
        icon: "success",
      });
    });
  },

  goCreateReusablePackingList() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      personalGearActionSheetVisible: false,
      importSheetVisible: false,
      myGearImportSheetVisible: false,
    });
    wx.navigateTo({
      url: `/pages/packing-lists/form/index?returnTripId=${encodeURIComponent(this.data.id)}`,
    });
  },

  openPlanGearForm() {
    if (
      this.data.myGearImportTarget === "shared" &&
      !this.data.selectedSharedGearSlot
    ) {
      wx.showToast({ title: "请先选择公共装备需求", icon: "none" });
      return;
    }
    const selectedSlot = this.data.selectedSharedGearSlot;
    const target = this.data.myGearImportTarget;
    const category = selectedSlot?.category ?? DEFAULT_PLAN_GEAR_CATEGORY;
    const categoryIndex = Math.max(
      0,
      GEAR_CATEGORY_OPTIONS.findIndex((item) => item.value === category),
    );
    this.setData({
      myGearImportSheetVisible: false,
      planGearFormVisible: true,
      planGearFormTarget: target,
      planGearForm: {
        ...defaultPlanGearForm(),
        category,
        plannedQuantity:
          target === "shared"
            ? this.data.sharedSlotPlannedQuantity ||
              String(selectedSlot?.plannedQuantity || 1)
            : "1",
      },
      planGearCategoryIndex: categoryIndex,
      sharedResponsibleIndex: defaultMemberIndex(
        this.data.members,
        target === "shared"
          ? this.data.members[this.data.sharedSlotResponsibleIndex]?.id ||
              selectedSlot?.responsibleMemberId ||
              this.data.myMember?.id
          : this.data.myMember?.id,
      ),
    });
  },

  openSharedGearForm() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({
      sharedSlotFormVisible: true,
      sharedSlotForm: defaultSharedSlotForm(),
      sharedSlotCategoryIndex: DEFAULT_PLAN_GEAR_CATEGORY_INDEX,
      sharedSlotResponsibleIndex: defaultMemberIndex(
        this.data.members,
        this.data.myMember?.id,
      ),
    });
  },

  closeSharedSlotForm() {
    this.setData({ sharedSlotFormVisible: false });
  },

  onSharedSlotFormInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof SharedSlotFormData;
    this.setData({ [`sharedSlotForm.${field}`]: event.detail.value });
  },

  onSharedSlotCategoryChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value || 0);
    const option =
      this.data.gearCategoryOptions[index] ||
      this.data.gearCategoryOptions[DEFAULT_PLAN_GEAR_CATEGORY_INDEX];
    this.setData({
      sharedSlotCategoryIndex: index,
      "sharedSlotForm.category": option.value,
    });
  },

  onSharedSlotResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      sharedSlotResponsibleIndex: Number(event.detail.value) || 0,
    });
  },

  onSharedSlotQuantityInput(event: WechatMiniprogram.Input) {
    this.setData({ sharedSlotPlannedQuantity: event.detail.value });
  },

  openSharedSlotSheet(event: WechatMiniprogram.BaseEvent) {
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.sharedGearRows.find((gear) => gear.id === itemId);
    if (!item) {
      return;
    }
    this.setData({
      selectedSharedGearSlot: item,
      sharedSlotSheetVisible: true,
      sharedSlotPlannedQuantity: String(item.plannedQuantity || 1),
      sharedSlotResponsibleIndex: defaultMemberIndex(
        this.data.members,
        item.responsibleMemberId || this.data.myMember?.id,
      ),
    });
  },

  closeSharedSlotSheet() {
    this.setData({ sharedSlotSheetVisible: false });
  },

  async saveSharedSlotDemand() {
    const slot = this.data.selectedSharedGearSlot;
    if (!slot) {
      return;
    }
    const plannedQuantity = parsePositiveInteger(
      this.data.sharedSlotPlannedQuantity,
      slot.plannedQuantity || 1,
    );
    if (!plannedQuantity) {
      wx.showToast({ title: "请填写有效总需求量", icon: "none" });
      return;
    }
    const responsibleMember =
      this.data.members[this.data.sharedSlotResponsibleIndex];
    if (!responsibleMember) {
      wx.showToast({ title: "请选择负责人", icon: "none" });
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      const next = await this.upsertSharedGearSlot(slot, {
        ...sharedGearSlotBasePayload(
          slot,
          String(plannedQuantity),
          responsibleMember.id,
        ),
        concrete_name: slot.hasConcreteGear ? slot.concreteName : null,
        source_gear_id: slot.sourceGearId || null,
      });
      this.setData({ sharedSlotSheetVisible: false });
      this.applyDetail(next);
      wx.showToast({ title: "已保存需求", icon: "success" });
    });
  },

  openSharedSlotImportSheet() {
    if (!this.data.selectedSharedGearSlot) {
      wx.showToast({ title: "请先选择公共装备需求", icon: "none" });
      return;
    }
    this.setData({
      sharedSlotSheetVisible: false,
      myGearImportTarget: "shared",
      myGearImportSheetVisible: true,
    });
    this.loadMyGearImportItems(true);
  },

  openSharedSlotConcreteForm() {
    const slot = this.data.selectedSharedGearSlot;
    if (!slot) {
      wx.showToast({ title: "请先选择公共装备需求", icon: "none" });
      return;
    }
    const categoryIndex = Math.max(
      0,
      GEAR_CATEGORY_OPTIONS.findIndex((item) => item.value === slot.category),
    );
    this.setData({
      sharedSlotSheetVisible: false,
      planGearFormVisible: true,
      planGearFormTarget: "shared",
      planGearForm: {
        ...defaultPlanGearForm(),
        name: slot.concreteName || "",
        category: slot.category,
        brand: "",
        model: "",
        plannedQuantity: this.data.sharedSlotPlannedQuantity,
      },
      planGearCategoryIndex: categoryIndex,
      sharedResponsibleIndex: defaultMemberIndex(
        this.data.members,
        this.data.members[this.data.sharedSlotResponsibleIndex]?.id ||
          slot.responsibleMemberId ||
          this.data.myMember?.id,
      ),
    });
  },

  closePlanGearForm() {
    this.setData({ planGearFormVisible: false });
  },

  onPlanGearInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof PlanGearFormData;
    this.setData({ [`planGearForm.${field}`]: event.detail.value });
  },

  onPlanGearCategoryChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value || 0);
    const option =
      this.data.gearCategoryOptions[index] ||
      this.data.gearCategoryOptions[DEFAULT_PLAN_GEAR_CATEGORY_INDEX];
    this.setData({
      planGearCategoryIndex: index,
      "planGearForm.category": option.value,
    });
  },

  async createPlanGearOnly() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const form = this.data.planGearForm;
    const name = form.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写装备名称", icon: "none" });
      return;
    }
    const plannedQuantity = parsePositiveInteger(form.plannedQuantity, 1);
    if (!plannedQuantity) {
      wx.showToast({ title: "请填写有效数量", icon: "none" });
      return;
    }
    const unitWeightG = parseOptionalNonNegativeInteger(form.unitWeightG);
    if (unitWeightG === undefined) {
      wx.showToast({ title: "请填写有效重量", icon: "none" });
      return;
    }
    if (
      this.data.planGearFormTarget === "shared" &&
      (!this.data.members[this.data.sharedResponsibleIndex] ||
        !this.data.selectedSharedGearSlot)
    ) {
      wx.showToast({ title: "请选择需求项和负责人", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const target = this.data.planGearFormTarget;
      const payload = {
        name,
        category: form.category,
        brand: nullableText(form.brand),
        model: nullableText(form.model),
        planned_quantity: plannedQuantity,
        packed_quantity: 0,
        unit_weight_g: unitWeightG,
        notes: nullableText(form.notes),
      };
      const next =
        target === "shared" && this.data.selectedSharedGearSlot
          ? await this.upsertSharedGearSlot(this.data.selectedSharedGearSlot, {
              ...sharedGearSlotBasePayload(
                this.data.selectedSharedGearSlot,
                String(plannedQuantity),
                this.data.members[this.data.sharedResponsibleIndex]?.id ?? "",
              ),
              concrete_name: name,
              source_gear_id: null,
              source_member_id: null,
              brand: payload.brand,
              model: payload.model,
              unit_weight_g: payload.unit_weight_g,
              notes: payload.notes,
            })
          : await createTripPersonalGearItem(this.data.id, payload);
      this.setData({
        planGearFormVisible: false,
        planGearFormTarget: "personal",
        planGearForm: defaultPlanGearForm(),
        planGearCategoryIndex: DEFAULT_PLAN_GEAR_CATEGORY_INDEX,
      });
      this.applyDetail(next);
    });
  },

  async createSharedSlotOnly() {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const form = this.data.sharedSlotForm;
    const name = form.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写需求名称", icon: "none" });
      return;
    }
    const plannedQuantity = parsePositiveInteger(form.plannedQuantity, 1);
    if (!plannedQuantity) {
      wx.showToast({ title: "请填写有效总需求量", icon: "none" });
      return;
    }
    const responsibleMember =
      this.data.members[this.data.sharedSlotResponsibleIndex];
    if (!responsibleMember) {
      wx.showToast({ title: "请选择负责人", icon: "none" });
      return;
    }
    const slotKey = customSharedGearSlotKey(name);
    const slot: SharedGearView = {
      id: slotKey,
      recordId: "",
      name,
      slotKey,
      slotName: name,
      slotGroup: "自定义",
      concreteName: "",
      sourceGearId: "",
      category: form.category,
      categoryLabel: getGearCategoryLabel(form.category),
      plannedQuantity,
      plannedQuantityText: `${plannedQuantity} 件`,
      responsibleText: responsibleMember.displayName,
      responsibleMemberId: responsibleMember.id,
      responsibleIndex: this.data.sharedSlotResponsibleIndex,
      sourceText: "待填写",
      createdByText: "自定义",
      weightText: formatTripWeight(0),
      hasConcreteGear: false,
      actionText: "填写",
      isDefaultSlot: false,
      isPlaceholder: true,
      fieldVersions: {},
    };
    await this.runMutation(async () => {
      const next = await createTripSharedGearDemand(this.data.id, {
        ...sharedGearSlotBasePayload(
          slot,
          String(plannedQuantity),
          responsibleMember.id,
        ),
        concrete_name: null,
        source_gear_id: null,
        source_member_id: null,
        brand: null,
        model: null,
        unit_weight_g: null,
        notes: null,
      });
      this.setData({ sharedSlotFormVisible: false });
      this.applyDetail(next);
      wx.showToast({ title: "已新增需求项", icon: "success" });
    });
  },

  async upsertSharedGearSlot(
    slot: SharedGearView,
    payload: TripRecordPatchRequest,
  ): Promise<TripDetail> {
    if (!payload.responsible_member_id) {
      throw new Error("shared gear requires a responsible member");
    }
    if (slot.recordId) {
      return updateTripSharedGearDemand(
        this.data.id,
        slot.recordId,
        withSharedGearBaseVersions(slot, payload),
      );
    }
    return createTripSharedGearDemand(this.data.id, payload);
  },

  confirmMovePersonalGearToShared(event: WechatMiniprogram.BaseEvent) {
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.detail?.personal_gear_items.find(
      (gear) => gear.id === itemId,
    );
    if (!item) {
      return;
    }
    wx.showModal({
      title: "移到公共装备",
      content: `会把「${item.name}」转为公共装备，并从个人装备中移除。`,
      confirmText: "移动",
      success: (result) => {
        if (result.confirm) {
          this.movePersonalGearToShared(item);
        }
      },
    });
  },

  async movePersonalGearToShared(item: TripPersonalGearItem) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      await createTripSharedGearDemand(this.data.id, {
        name: item.name,
        category: item.category,
        category_label: item.category_label,
        brand: item.brand ?? null,
        model: item.model ?? null,
        planned_quantity: item.planned_quantity,
        packed_quantity: item.packed_quantity,
        unit_weight_g: item.unit_weight_g ?? null,
        notes: item.notes ?? null,
        source_member_id: item.member_id,
        responsible_member_id: item.member_id,
      });
      const next = await deleteTripPersonalGearItem(this.data.id, item.id);
      this.applyDetail(next);
      wx.showToast({ title: "已移到公共装备", icon: "success" });
    });
  },

  async changeSharedGearResponsible(event: WechatMiniprogram.PickerChange) {
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.sharedGearRows.find((gear) => gear.id === itemId);
    const nextMember = this.data.members[Number(event.detail.value) || 0];
    if (!item || !nextMember || item.responsibleMemberId === nextMember.id) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      const next = await this.upsertSharedGearSlot(
        item,
        item.recordId
          ? { responsible_member_id: nextMember.id }
          : sharedGearSlotBasePayload(
              item,
              String(item.plannedQuantity || 1),
              nextMember.id,
            ),
      );
      this.applyDetail(next);
      wx.showToast({ title: "已更新负责人", icon: "success" });
    });
  },

  confirmDeleteSharedGear(event: WechatMiniprogram.BaseEvent) {
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.sharedGearRows.find((gear) => gear.id === itemId);
    if (!item) {
      return;
    }
    if (!item.recordId) {
      wx.showToast({ title: "这个需求还没有保存内容", icon: "none" });
      return;
    }
    if (!item.isDefaultSlot && item.hasConcreteGear) {
      wx.showActionSheet({
        itemList: ["清空具体装备", "删除整个需求项"],
        success: (result) => {
          if (result.tapIndex === 0) {
            this.clearSharedGearConcrete(item);
          } else if (result.tapIndex === 1) {
            this.deleteSharedGear(item.recordId, "已删除需求项");
          }
        },
      });
      return;
    }
    wx.showModal({
      title: "删除公共装备需求",
      content: `确定删除「${item.slotName}」这项公共装备需求吗？`,
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          this.deleteSharedGear(item.recordId, "已删除需求项");
        }
      },
    });
  },

  async clearSharedGearConcrete(item: SharedGearView) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      const next = await this.upsertSharedGearSlot(item, {
        name: item.slotName,
        template_key: item.slotKey,
        demand_name: item.slotName,
        concrete_name: null,
        source_gear_id: null,
        source_member_id: null,
        brand: null,
        model: null,
        packed_quantity: 0,
        unit_weight_g: null,
        notes: null,
        planned_quantity: item.plannedQuantity,
        responsible_member_id: item.responsibleMemberId,
      });
      this.applyDetail(next);
      wx.showToast({ title: "已清空具体装备", icon: "success" });
    });
  },

  async deleteSharedGear(itemId: string, toastTitle = "已删除公共装备") {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    await this.runMutation(async () => {
      const next = await deleteTripSharedGearDemand(this.data.id, itemId);
      this.applyDetail(next);
      wx.showToast({ title: toastTitle, icon: "success" });
    });
  },

  async addItineraryDay() {
    const detail = this.data.detail;
    if (!detail) {
      return;
    }
    await this.runMutation(async () => {
      const next = await createTripItineraryDay(this.data.id, {
        day_index: detail.itinerary_days.length + 1,
        title: `第 ${detail.itinerary_days.length + 1} 天`,
      });
      const createdDay = newestItineraryDay(
        detail.itinerary_days,
        next.itinerary_days,
      );
      if (createdDay) {
        this.setData({
          activeItineraryDayId: createdDay.id,
          activeFoodDayId: createdDay.id,
          routeEditorVisible: false,
        });
      }
      this.applyDetail(next);
    });
  },

  onItineraryTouchStart(event: WechatMiniprogram.TouchEvent) {
    const touch = event.touches[0];
    if (!touch) {
      return;
    }
    this.itinerarySwipeStartPoint = { x: touch.clientX, y: touch.clientY };
  },

  onItineraryTouchEnd(event: WechatMiniprogram.TouchEvent) {
    const touch = event.changedTouches[0];
    const startPoint = this.itinerarySwipeStartPoint;
    this.itinerarySwipeStartPoint = null;
    if (!touch || !startPoint || this.data.itineraryDays.length < 2) {
      return;
    }
    const deltaX = touch.clientX - startPoint.x;
    const deltaY = touch.clientY - startPoint.y;
    if (Math.abs(deltaX) < 36 || Math.abs(deltaX) < Math.abs(deltaY) * 1.1) {
      return;
    }
    this.switchActiveItineraryDay(deltaX < 0 ? 1 : -1);
  },

  switchPrevItineraryDay() {
    this.switchActiveItineraryDay(-1);
  },

  switchNextItineraryDay() {
    this.switchActiveItineraryDay(1);
  },

  openRouteEditor() {
    this.setData({
      routeName: "",
      routeCheckpoint: "",
      routeDistanceKm: "5",
      routeAscentM: "300",
      routeDescentM: "0",
      routeTrailCondition: "",
      routeEditorVisible: true,
      routeEditorSegmentId: "",
    });
  },

  openEditRouteEditor(event: WechatMiniprogram.BaseEvent) {
    const segmentId = event.currentTarget.dataset.id as string | undefined;
    const segment = findRouteSegment(this.data.detail, segmentId);
    if (!segment || this.data.mutating) {
      return;
    }
    this.setData({
      routeName: segment.name,
      routeCheckpoint: segment.checkpoint || "",
      routeDistanceKm: String(segment.distance_km),
      routeAscentM: String(segment.ascent_m),
      routeDescentM: String(segment.descent_m),
      routeTrailCondition: segment.trail_condition || "",
      routeEditorVisible: true,
      routeEditorSegmentId: segment.id,
    });
  },

  closeRouteEditor() {
    this.setData({
      routeEditorVisible: false,
      routeEditorSegmentId: "",
    });
  },

  switchActiveItineraryDay(offset: number) {
    const days = this.data.itineraryDays;
    if (!days.length) {
      return;
    }
    const currentIndex = Math.max(0, this.data.activeItineraryDayIndex);
    const nextIndex = Math.min(
      days.length - 1,
      Math.max(0, currentIndex + offset),
    );
    const nextDay = days[nextIndex];
    if (!nextDay || nextDay.id === this.data.activeItineraryDayId) {
      return;
    }
    this.setData({
      activeItineraryDayId: nextDay.id,
      routeEditorVisible: false,
    });
    if (this.data.detail) {
      this.applyDetail(this.data.detail);
    }
  },

  switchPrevFoodDay() {
    this.switchActiveFoodDay(-1);
  },

  switchNextFoodDay() {
    this.switchActiveFoodDay(1);
  },

  switchActiveFoodDay(offset: number) {
    const days = this.data.foodDays;
    if (!days.length) {
      return;
    }
    const currentIndex = Math.max(0, this.data.activeFoodDayIndex);
    const nextIndex = Math.min(
      days.length - 1,
      Math.max(0, currentIndex + offset),
    );
    const nextDay = days[nextIndex];
    if (!nextDay || nextDay.id === this.data.activeFoodDayId) {
      return;
    }
    this.setData({ activeFoodDayId: nextDay.id });
    if (this.data.detail) {
      this.applyDetail(this.data.detail);
    }
  },

  confirmDeleteItineraryDay(event: WechatMiniprogram.BaseEvent) {
    if (this.data.mutating) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const dayId = event.currentTarget.dataset.id as string | undefined;
    const day = this.data.itineraryDays.find((item) => item.id === dayId);
    if (!day) {
      return;
    }
    wx.showModal({
      title: `删除${day.title}？`,
      content: "删除后这一天会从行程里隐藏，路段可继续单独删除。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteItineraryDay(day.id);
        }
      },
    });
  },

  async deleteItineraryDay(dayId: string) {
    await this.runMutation(async () => {
      const currentIndex = this.data.itineraryDays.findIndex(
        (day) => day.id === dayId,
      );
      const next = await deleteTripItineraryDay(this.data.id, dayId);
      if (this.data.activeItineraryDayId === dayId) {
        const fallbackIndex =
          currentIndex >= 0
            ? Math.min(currentIndex, next.itinerary_days.length - 1)
            : 0;
        const nextDay =
          fallbackIndex >= 0
            ? (next.itinerary_days[fallbackIndex] ?? null)
            : null;
        this.setData({
          activeItineraryDayId: nextDay?.id ?? "",
          routeEditorVisible: false,
        });
      }
      if (this.data.activeFoodDayId === dayId) {
        const fallbackIndex =
          currentIndex >= 0
            ? Math.min(currentIndex, next.itinerary_days.length - 1)
            : 0;
        const nextDay =
          fallbackIndex >= 0
            ? (next.itinerary_days[fallbackIndex] ?? null)
            : null;
        this.setData({ activeFoodDayId: nextDay?.id ?? "" });
      }
      this.applyDetail(next);
    });
  },

  openDayInfoEditor(event: WechatMiniprogram.BaseEvent) {
    const dayId = event.currentTarget.dataset.id as string | undefined;
    const rawDay = this.data.detail?.itinerary_days.find(
      (item) => item.id === dayId,
    );
    const day = this.data.itineraryDays.find((item) => item.id === dayId);
    if (!rawDay || !day) {
      return;
    }
    this.setData({
      dayInfoEditorVisible: true,
      dayInfoEditorDay: day,
      dayInfoForm: {
        weather: rawDay.weather || "",
        highTemperatureC:
          typeof rawDay.high_temperature_c === "number"
            ? String(rawDay.high_temperature_c)
            : "",
        lowTemperatureC:
          typeof rawDay.low_temperature_c === "number"
            ? String(rawDay.low_temperature_c)
            : "",
        weatherSummary: rawDay.weather_summary || "",
        weatherNotes: rawDay.weather_notes || "",
        campName: rawDay.camp_name || "",
        campAltitudeM:
          typeof rawDay.camp_altitude_m === "number"
            ? String(rawDay.camp_altitude_m)
            : "",
        campTerrain: rawDay.camp_terrain || "",
        campSlope: rawDay.camp_slope || "",
        campArea: rawDay.camp_area || "",
        campWaterSource: rawDay.camp_water_source || "",
        campNotes: rawDay.camp_notes || "",
      },
    });
  },

  closeDayInfoEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      dayInfoEditorVisible: false,
      dayInfoEditorDay: null,
      dayInfoForm: defaultDayInfoForm(),
    });
  },

  onDayInfoInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as
      | keyof DayInfoFormData
      | undefined;
    if (!field) {
      return;
    }
    this.setData({ [`dayInfoForm.${field}`]: event.detail.value });
  },

  async saveDayInfo() {
    const day = this.data.detail?.itinerary_days.find(
      (item) => item.id === this.data.dayInfoEditorDay?.id,
    );
    if (!day) {
      return;
    }
    const highTemperature = parseNullableInteger(
      this.data.dayInfoForm.highTemperatureC,
    );
    const lowTemperature = parseNullableInteger(
      this.data.dayInfoForm.lowTemperatureC,
    );
    const campAltitude = parseNullableInteger(
      this.data.dayInfoForm.campAltitudeM,
    );
    if (
      highTemperature === undefined ||
      lowTemperature === undefined ||
      campAltitude === undefined
    ) {
      wx.showToast({ title: "温度和海拔需要填写整数", icon: "none" });
      return;
    }
    const fields: Record<string, string | number | null> = {
      weather: nullableText(this.data.dayInfoForm.weather),
      high_temperature_c: highTemperature,
      low_temperature_c: lowTemperature,
      weather_summary: nullableText(this.data.dayInfoForm.weatherSummary),
      weather_notes: nullableText(this.data.dayInfoForm.weatherNotes),
      camp_name: nullableText(this.data.dayInfoForm.campName),
      camp_altitude_m: campAltitude,
      camp_terrain: nullableText(this.data.dayInfoForm.campTerrain),
      camp_slope: nullableText(this.data.dayInfoForm.campSlope),
      camp_area: nullableText(this.data.dayInfoForm.campArea),
      camp_water_source: nullableText(this.data.dayInfoForm.campWaterSource),
      camp_notes: nullableText(this.data.dayInfoForm.campNotes),
    };
    await this.runMutation(async () => {
      const next = await updateTripItineraryDay(this.data.id, day.id, {
        ...fields,
        base_field_versions: baseVersionsForRecordFields(fields, day),
      });
      this.setData({
        dayInfoEditorVisible: false,
        dayInfoEditorDay: null,
        dayInfoForm: defaultDayInfoForm(),
      });
      this.applyDetail(next);
    });
  },

  async saveRouteSegment(event: WechatMiniprogram.BaseEvent) {
    if (this.data.routeEditorSegmentId) {
      await this.updateRouteSegment(this.data.routeEditorSegmentId);
      return;
    }
    await this.addRouteSegment(event);
  },

  async addRouteSegment(event: WechatMiniprogram.BaseEvent) {
    const dayId = event.currentTarget.dataset.dayId as string | undefined;
    if (!dayId) {
      wx.showToast({ title: "请先展开某一天", icon: "none" });
      return;
    }
    const name = this.data.routeName.trim();
    if (!name) {
      wx.showToast({ title: "请填写路段名称", icon: "none" });
      return;
    }
    const detail = this.data.detail;
    const beforeSegments = new Set(
      detail?.route_segments.map((segment) => segment.id) ?? [],
    );
    await this.runMutation(async () => {
      let next = await createTripRouteSegment(this.data.id, {
        name,
        checkpoint: nullableText(this.data.routeCheckpoint),
        leader_member_id: null,
        bailout_route: null,
        trail_condition: nullableText(this.data.routeTrailCondition),
        distance_km: Number(this.data.routeDistanceKm) || 0,
        ascent_m: Number(this.data.routeAscentM) || 0,
        descent_m: Number(this.data.routeDescentM) || 0,
        descent_profile: "none",
      });
      const createdSegment = newestRouteSegment(
        beforeSegments,
        next.route_segments,
      );
      if (createdSegment) {
        this.applyDetail(next);
        next = await createTripItineraryTimeSlot(this.data.id, dayId, {
          slot_key: "route",
          route_segment_id: createdSegment.id,
        });
      }
      this.setData({
        routeName: "",
        routeCheckpoint: "",
        routeDistanceKm: "5",
        routeAscentM: "300",
        routeDescentM: "0",
        routeTrailCondition: "",
        routeEditorVisible: false,
        routeEditorSegmentId: "",
      });
      this.applyDetail(next);
    });
  },

  async updateRouteSegment(segmentId: string) {
    const segment = findRouteSegment(this.data.detail, segmentId);
    if (!segment) {
      wx.showToast({ title: "路段不存在", icon: "none" });
      return;
    }
    const name = this.data.routeName.trim();
    if (!name) {
      wx.showToast({ title: "请填写路段名称", icon: "none" });
      return;
    }
    const fields: Record<string, string | number | null> = {
      name,
      checkpoint: nullableText(this.data.routeCheckpoint),
      trail_condition: nullableText(this.data.routeTrailCondition),
      distance_km: Number(this.data.routeDistanceKm) || 0,
      ascent_m: Number(this.data.routeAscentM) || 0,
      descent_m: Number(this.data.routeDescentM) || 0,
      descent_profile: segment.descent_profile || "none",
    };
    await this.runMutation(async () => {
      const next = await updateTripRouteSegment(this.data.id, segment.id, {
        ...fields,
        base_field_versions: baseVersionsForRecordFields(fields, segment),
      });
      this.setData({
        routeName: "",
        routeCheckpoint: "",
        routeDistanceKm: "5",
        routeAscentM: "300",
        routeDescentM: "0",
        routeTrailCondition: "",
        routeEditorVisible: false,
        routeEditorSegmentId: "",
      });
      this.applyDetail(next);
    });
  },

  confirmDeleteRouteSegment(event: WechatMiniprogram.BaseEvent) {
    if (this.data.mutating) {
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const segmentId = event.currentTarget.dataset.id as string | undefined;
    const segment = findRouteSegmentView(this.data, segmentId);
    if (!segment) {
      return;
    }
    wx.showModal({
      title: `删除${segment.name}？`,
      content: "删除后这条路段估算会从行程里移除。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteRouteSegment(segment.id);
        }
      },
    });
  },

  async deleteRouteSegment(segmentId: string) {
    await this.runMutation(async () => {
      const next = await deleteTripRouteSegment(this.data.id, segmentId);
      this.applyDetail(next);
    });
  },

  openMedicalEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      medicalEditorVisible: true,
      medicalEditorItem: null,
      medicalForm: defaultMedicalForm(),
      medicalScopeIndex: defaultMedicalScopeIndex("public_first_aid"),
    });
  },

  openEditMedicalEditor(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.medicalItems.find((entry) => entry.id === id);
    if (!item || this.data.mutating) {
      return;
    }
    const form = medicalFormFromItem(item);
    this.setData({
      medicalEditorVisible: true,
      medicalEditorItem: item,
      medicalForm: form,
      medicalScopeIndex: defaultMedicalScopeIndex(form.scope),
    });
  },

  closeMedicalEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      medicalEditorVisible: false,
      medicalEditorItem: null,
      medicalForm: defaultMedicalForm(),
      medicalScopeIndex: defaultMedicalScopeIndex("public_first_aid"),
    });
  },

  onMedicalFormInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof MedicalFormData;
    this.setData({ [`medicalForm.${field}`]: event.detail.value });
  },

  onMedicalScopeChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value || 0);
    const option = MEDICAL_SCOPE_OPTIONS[index] ?? MEDICAL_SCOPE_OPTIONS[0];
    this.setData({
      medicalScopeIndex: index,
      "medicalForm.scope": option.value,
    });
  },

  async saveMedicalItem() {
    const form = this.data.medicalForm;
    const name = form.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写医药包条目", icon: "none" });
      return;
    }
    const suggestedQuantity = parseNullableInteger(form.suggestedQuantity);
    const parsedRequiredQuantity = parseNullableInteger(form.requiredQuantity);
    const parsedPackedQuantity = parseNullableInteger(form.packedQuantity);
    if (
      suggestedQuantity === undefined ||
      parsedRequiredQuantity === undefined ||
      parsedPackedQuantity === undefined
    ) {
      wx.showToast({ title: "请填写有效数量", icon: "none" });
      return;
    }
    const requiredQuantity = parsedRequiredQuantity ?? 1;
    const packedQuantity = parsedPackedQuantity ?? 0;
    const fields = {
      name,
      item_type: nullableText(form.itemType),
      scope: nullableText(form.scope),
      suggested_quantity: suggestedQuantity,
      required_quantity: Math.max(0, requiredQuantity),
      packed_quantity: Math.max(0, packedQuantity),
    };
    const editingItem = this.data.medicalEditorItem;
    await this.runMutation(async () => {
      const next = editingItem
        ? await updateTripMedicalItem(this.data.id, editingItem.id, {
            ...fields,
            base_field_versions: baseVersionsForRecordFields(
              fields,
              editingItem,
            ),
          })
        : await createTripMedicalItem(this.data.id, fields);
      this.setData({
        medicalEditorVisible: false,
        medicalEditorItem: null,
        medicalForm: defaultMedicalForm(),
        medicalScopeIndex: defaultMedicalScopeIndex("public_first_aid"),
      });
      this.applyDetail(next);
    });
  },

  confirmDeleteMedicalItem(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.medicalItems.find((entry) => entry.id === id);
    if (!item || this.data.mutating) {
      return;
    }
    wx.showModal({
      title: `删除${item.name}？`,
      content: "删除后会从当前计划的医药包中移除。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteMedicalItem(item.id);
        }
      },
    });
  },

  async deleteMedicalItem(itemId: string) {
    await this.runMutation(async () => {
      const next = await deleteTripMedicalItem(this.data.id, itemId);
      this.applyDetail(next);
    });
  },

  openFoodMealEditor(event: WechatMiniprogram.BaseEvent) {
    const mealId = event.currentTarget.dataset.id as string | undefined;
    const meal = findFoodMealView(this.data.foodDays, mealId);
    if (!meal || this.data.mutating) {
      return;
    }
    this.setData({
      foodMealEditorVisible: true,
      foodMealEditorMeal: meal,
      foodMealForm: {
        mealType: meal.meal_type || "",
        dishName: meal.dish_name || "",
        responsibleIndex: defaultMemberIndex(
          this.data.members,
          meal.responsibleMemberId || this.data.myMember?.id,
        ),
        notes: meal.notes || "",
      },
    });
  },

  closeFoodMealEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      foodMealEditorVisible: false,
      foodMealEditorMeal: null,
      foodMealForm: defaultFoodMealForm(),
    });
  },

  onFoodMealInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof FoodMealFormData;
    this.setData({ [`foodMealForm.${field}`]: event.detail.value });
  },

  onFoodMealResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      "foodMealForm.responsibleIndex": Number(event.detail.value) || 0,
    });
  },

  async saveFoodMeal() {
    const meal = this.data.foodMealEditorMeal;
    if (!meal) {
      return;
    }
    const responsible =
      this.data.members[this.data.foodMealForm.responsibleIndex];
    const fields = {
      meal_type: nullableText(this.data.foodMealForm.mealType),
      dish_name: nullableText(this.data.foodMealForm.dishName),
      responsible_member_id: responsible?.id || null,
      notes: nullableText(this.data.foodMealForm.notes),
      skipped: false,
    };
    await this.runMutation(async () => {
      const next = await updateTripFoodMeal(this.data.id, meal.id, {
        ...fields,
        base_field_versions: baseVersionsForRecordFields(fields, meal),
      });
      this.setData({
        foodMealEditorVisible: false,
        foodMealEditorMeal: null,
        foodMealForm: defaultFoodMealForm(),
      });
      this.applyDetail(next);
    });
  },

  async toggleMealSkip(event: WechatMiniprogram.BaseEvent) {
    const mealId = event.currentTarget.dataset.id as string;
    const meal = findFoodMealView(this.data.foodDays, mealId);
    if (!meal) {
      return;
    }
    const skipped = !meal.skipped;
    await this.runMutation(async () => {
      const next = await updateTripFoodMeal(this.data.id, meal.id, {
        skipped,
        base_field_versions: {
          skipped: meal.field_versions.skipped ?? 0,
        },
      });
      this.applyDetail(next);
      wx.showToast({
        title: skipped ? "已跳过餐次" : "已恢复餐次",
        icon: "success",
      });
    });
  },

  openFoodItemEditor(event: WechatMiniprogram.BaseEvent) {
    const mealId = event.currentTarget.dataset.mealId as string | undefined;
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const meal = findFoodMealView(this.data.foodDays, mealId);
    if (!meal || this.data.mutating) {
      return;
    }
    const item = meal.items.find((entry) => entry.id === itemId) || null;
    this.setData({
      foodItemEditorVisible: true,
      foodItemEditorMeal: meal,
      foodItemEditorItem: item,
      foodItemForm: item
        ? {
            name: item.name,
            amountG: item.amount_g == null ? "" : String(item.amount_g),
            costYuan:
              item.total_price_cents == null
                ? ""
                : formatMoneyValueInput(item.total_price_cents),
            responsibleIndex: defaultMemberIndex(
              this.data.members,
              item.responsible_member_id ||
                meal.responsibleMemberId ||
                this.data.myMember?.id,
            ),
            notes: item.notes || "",
          }
        : {
            ...defaultFoodItemForm(),
            responsibleIndex: defaultMemberIndex(
              this.data.members,
              meal.responsibleMemberId || this.data.myMember?.id,
            ),
          },
    });
  },

  closeFoodItemEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      foodItemEditorVisible: false,
      foodItemEditorMeal: null,
      foodItemEditorItem: null,
      foodItemForm: defaultFoodItemForm(),
    });
  },

  onFoodItemInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof FoodItemFormData;
    this.setData({ [`foodItemForm.${field}`]: event.detail.value });
  },

  onFoodItemResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      "foodItemForm.responsibleIndex": Number(event.detail.value) || 0,
    });
  },

  async saveFoodItem() {
    const meal = this.data.foodItemEditorMeal;
    if (!meal) {
      return;
    }
    const form = this.data.foodItemForm;
    const name = form.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写食材名称", icon: "none" });
      return;
    }
    const amountG = parseOptionalNonNegativeInteger(form.amountG);
    if (amountG === undefined) {
      wx.showToast({ title: "请填写有效重量", icon: "none" });
      return;
    }
    const totalPriceCents = parseNullableMoneyCents(form.costYuan);
    if (totalPriceCents === undefined) {
      wx.showToast({ title: "请填写有效费用", icon: "none" });
      return;
    }
    const responsible = this.data.members[form.responsibleIndex];
    const fields = {
      name,
      amount_g: amountG,
      per_person_amount_g: null,
      total_price_cents: totalPriceCents,
      responsible_member_id: responsible?.id || null,
      notes: nullableText(form.notes),
    };
    const editingItem = this.data.foodItemEditorItem;
    await this.runMutation(async () => {
      const next = editingItem
        ? await updateTripFoodItem(this.data.id, meal.id, editingItem.id, {
            ...fields,
            base_field_versions: baseVersionsForRecordFields(
              fields,
              editingItem,
            ),
          })
        : await createTripFoodItem(this.data.id, meal.id, fields);
      this.setData({
        foodItemEditorVisible: false,
        foodItemEditorMeal: null,
        foodItemEditorItem: null,
        foodItemForm: defaultFoodItemForm(),
      });
      this.applyDetail(next);
    });
  },

  confirmDeleteFoodItem(event: WechatMiniprogram.BaseEvent) {
    const mealId = event.currentTarget.dataset.mealId as string | undefined;
    const itemId = event.currentTarget.dataset.id as string | undefined;
    const meal = findFoodMealView(this.data.foodDays, mealId);
    const item = meal?.items.find((entry) => entry.id === itemId);
    if (!meal || !item || this.data.mutating) {
      return;
    }
    wx.showModal({
      title: `删除${item.name}？`,
      content: "删除后会从食品计划中移除，并同步更新食材背负重量。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteFoodItem(meal.id, item.id);
        }
      },
    });
  },

  async deleteFoodItem(mealId: string, itemId: string) {
    await this.runMutation(async () => {
      const next = await deleteTripFoodItem(this.data.id, mealId, itemId);
      this.applyDetail(next);
    });
  },

  openFoodSupplyEditor(event?: WechatMiniprogram.BaseEvent) {
    const itemId = event?.currentTarget.dataset.id as string | undefined;
    const item =
      this.data.foodSupplyRows.find((entry) => entry.id === itemId) || null;
    this.setData({
      foodSupplyEditorVisible: true,
      foodSupplyEditorItem: item,
      foodSupplyForm: item
        ? {
            name: item.name,
            supplyType: item.supply_type || "",
            amountG: item.amount_g == null ? "" : String(item.amount_g),
            costYuan:
              item.total_price_cents == null
                ? ""
                : formatMoneyValueInput(item.total_price_cents),
            responsibleIndex: defaultMemberIndex(
              this.data.members,
              item.responsible_member_id || this.data.myMember?.id,
            ),
            notes: item.notes || "",
          }
        : {
            ...defaultFoodSupplyForm(),
            responsibleIndex: defaultMemberIndex(
              this.data.members,
              this.data.myMember?.id,
            ),
          },
    });
  },

  closeFoodSupplyEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      foodSupplyEditorVisible: false,
      foodSupplyEditorItem: null,
      foodSupplyForm: defaultFoodSupplyForm(),
    });
  },

  onFoodSupplyInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as keyof FoodSupplyFormData;
    this.setData({ [`foodSupplyForm.${field}`]: event.detail.value });
  },

  onFoodSupplyResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({
      "foodSupplyForm.responsibleIndex": Number(event.detail.value) || 0,
    });
  },

  async saveFoodSupply() {
    const form = this.data.foodSupplyForm;
    const name = form.name.trim();
    if (!name) {
      wx.showToast({ title: "请填写公共食材名称", icon: "none" });
      return;
    }
    const amountG = parseOptionalNonNegativeInteger(form.amountG);
    if (amountG === undefined) {
      wx.showToast({ title: "请填写有效重量", icon: "none" });
      return;
    }
    const totalPriceCents = parseNullableMoneyCents(form.costYuan);
    if (totalPriceCents === undefined) {
      wx.showToast({ title: "请填写有效费用", icon: "none" });
      return;
    }
    const responsible = this.data.members[form.responsibleIndex];
    const fields = {
      name,
      supply_type: nullableText(form.supplyType),
      amount_g: amountG,
      per_person_amount_g: null,
      total_price_cents: totalPriceCents,
      responsible_member_id: responsible?.id || null,
      notes: nullableText(form.notes),
    };
    const editingItem = this.data.foodSupplyEditorItem;
    await this.runMutation(async () => {
      const next = editingItem
        ? await updateTripFoodSupply(this.data.id, editingItem.id, {
            ...fields,
            base_field_versions: baseVersionsForRecordFields(
              fields,
              editingItem,
            ),
          })
        : await createTripFoodSupply(this.data.id, fields);
      this.setData({
        foodSupplyEditorVisible: false,
        foodSupplyEditorItem: null,
        foodSupplyForm: defaultFoodSupplyForm(),
      });
      this.applyDetail(next);
    });
  },

  confirmDeleteFoodSupply(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    const item = this.data.foodSupplyRows.find((entry) => entry.id === id);
    if (!item || this.data.mutating) {
      return;
    }
    wx.showModal({
      title: `删除${item.name}？`,
      content: "删除后会从食品计划中移除，并同步更新食材背负重量。",
      confirmText: "删除",
      confirmColor: "#dc2626",
      success: (result) => {
        if (result.confirm) {
          void this.deleteFoodSupply(item.id);
        }
      },
    });
  },

  async deleteFoodSupply(id: string) {
    await this.runMutation(async () => {
      const next = await deleteTripFoodSupply(this.data.id, id);
      this.applyDetail(next);
    });
  },

  async addSafetyRisk() {
    const riskType = this.data.safetyRiskType.trim();
    if (!riskType) {
      wx.showToast({ title: "请填写风险类型", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const next = await createTripSafetyRisk(this.data.id, {
        risk_type: riskType,
        prevention: nullableText(this.data.safetyPrevention),
        response: nullableText(this.data.safetyResponse),
        responsible_member_id:
          this.data.members[this.data.safetyResponsibleIndex]?.id || null,
      });
      this.setData({
        safetyRiskType: "",
        safetyPrevention: "",
        safetyResponse: "",
      });
      this.applyDetail(next);
    });
  },

  onSafetyResponsibleChange(event: WechatMiniprogram.PickerChange) {
    this.setData({ safetyResponsibleIndex: Number(event.detail.value) || 0 });
  },

  async deleteSafetyRisk(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    await this.runMutation(async () => {
      const next = await deleteTripSafetyRisk(this.data.id, id);
      this.applyDetail(next);
    });
  },

  async addRescueContact() {
    const organization = this.data.rescueOrganization.trim();
    if (!organization) {
      wx.showToast({ title: "请填写救援单位名称", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const next = await createTripRescueContact(this.data.id, {
        organization,
        address: nullableText(this.data.rescueAddress),
        phone: nullableText(this.data.rescuePhone),
      });
      this.setData({
        rescueOrganization: "",
        rescueAddress: "",
        rescuePhone: "",
      });
      this.applyDetail(next);
    });
  },

  async deleteRescueContact(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    await this.runMutation(async () => {
      const next = await deleteTripRescueContact(this.data.id, id);
      this.applyDetail(next);
    });
  },

  onBudgetSharedGearChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value) || 0;
    const sharedGear = index > 0 ? this.data.sharedGearRows[index - 1] : null;
    this.setData({
      budgetLinkedSharedGearIndex: index,
      ...(sharedGear && !this.data.budgetName.trim()
        ? { budgetName: sharedGear.concreteName || sharedGear.slotName }
        : {}),
      ...(sharedGear && this.data.budgetQuantity === "1"
        ? { budgetQuantity: String(sharedGear.plannedQuantity || 1) }
        : {}),
    });
  },

  openBudgetEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      budgetEditorVisible: true,
      budgetCategory: "",
      budgetName: "",
      budgetQuantity: "1",
      budgetUnitPriceYuan: "",
      budgetSplitMemberCount: "",
      budgetLinkedSharedGearIndex: 0,
    });
  },

  closeBudgetEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      budgetEditorVisible: false,
      budgetCategory: "",
      budgetName: "",
      budgetQuantity: "1",
      budgetUnitPriceYuan: "",
      budgetSplitMemberCount: "",
      budgetLinkedSharedGearIndex: 0,
    });
  },

  async addBudgetItem() {
    const linked = this.data.budgetLinkedSharedGearIndex
      ? this.data.sharedGearRows[this.data.budgetLinkedSharedGearIndex - 1]
      : null;
    const name = this.data.budgetName.trim() || linked?.slotName || "";
    if (!name) {
      wx.showToast({ title: "请填写预算名称", icon: "none" });
      return;
    }
    const quantity = parseNullableInteger(this.data.budgetQuantity) ?? 1;
    const unitPriceCents = parseNullableMoneyCents(
      this.data.budgetUnitPriceYuan,
    );
    const splitCount = parseNullableInteger(this.data.budgetSplitMemberCount);
    if (unitPriceCents === undefined || splitCount === undefined) {
      wx.showToast({ title: "金额和分摊人数格式不正确", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const next = await createTripBudgetItem(this.data.id, {
        category: nullableText(this.data.budgetCategory),
        name,
        quantity,
        unit_price_cents: unitPriceCents,
        total_price_cents:
          unitPriceCents == null
            ? null
            : unitPriceCents * Math.max(1, quantity),
        split_member_count: splitCount,
        linked_shared_gear_id: linked?.recordId || null,
      });
      this.setData({
        budgetCategory: "",
        budgetName: "",
        budgetQuantity: "1",
        budgetUnitPriceYuan: "",
        budgetSplitMemberCount: "",
        budgetLinkedSharedGearIndex: 0,
        budgetEditorVisible: false,
      });
      this.applyDetail(next);
    });
  },

  async deleteBudgetItem(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    await this.runMutation(async () => {
      const next = await deleteTripBudgetItem(this.data.id, id);
      this.applyDetail(next);
    });
  },

  openGoalEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      goalEditorVisible: true,
      goalContent: "",
    });
  },

  closeGoalEditor() {
    if (this.data.mutating) {
      return;
    }
    this.setData({
      goalEditorVisible: false,
      goalContent: "",
    });
  },

  async addGoalItem() {
    const content = this.data.goalContent.trim();
    if (!content) {
      wx.showToast({ title: "请填写目标内容", icon: "none" });
      return;
    }
    await this.runMutation(async () => {
      const next = await createTripGoalItem(this.data.id, {
        scope: "team",
        member_id: null,
        content,
      });
      this.setData({
        goalEditorVisible: false,
        goalContent: "",
      });
      this.applyDetail(next);
    });
  },

  async deleteGoalItem(event: WechatMiniprogram.BaseEvent) {
    const id = event.currentTarget.dataset.id as string | undefined;
    if (!id) {
      return;
    }
    await this.runMutation(async () => {
      const next = await deleteTripGoalItem(this.data.id, id);
      this.applyDetail(next);
    });
  },

  async runMutation(action: () => Promise<void>) {
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    this.setData({ mutating: true });
    try {
      await action();
      wx.setStorageSync("stellartrail_trips_refresh", true);
    } catch (error) {
      if (isLoginRequiredError(error)) {
        showLoginPrompt(this, {
          message: "登录状态已过期，请重新登录后继续编辑计划。",
          redirectUrl: `/pages/trips/detail/index?id=${encodeURIComponent(this.data.id)}`,
        });
        return;
      }
      showMutationError(error);
    } finally {
      this.setData({ mutating: false });
    }
  },

  loginPromptClose() {
    hideLoginPrompt(this);
  },

  loginPromptGoLogin() {
    openLoginPageFromPrompt(this);
  },
});

function buildDetailView(
  detail: TripDetail,
  selectedSection: TripSectionKey,
  preferredOptionalOrder: TripSectionKey[],
  activeItineraryDayId: string,
  activeFoodDayId: string,
  sharedGearCategoryFilter: SharedGearCategoryFilterValue,
) {
  const tripType = detail.plan.trip_type;
  const isTeamTrip = tripType === "team";
  const defaultSections = defaultSectionsForTripType(tripType);
  const enabled = new Set(
    sanitizeEnabledSectionsForTripType(detail.plan.enabled_sections, tripType),
  );
  defaultSections.forEach((key) => enabled.add(key));
  const dayIds = new Set(detail.itinerary_days.map((day) => day.id));
  const normalizedActiveItineraryDayId = dayIds.has(activeItineraryDayId)
    ? activeItineraryDayId
    : (detail.itinerary_days[0]?.id ?? "");
  const itineraryView = buildItineraryView(detail);
  const activeItineraryDayIndex = Math.max(
    0,
    itineraryView.days.findIndex(
      (day) => day.id === normalizedActiveItineraryDayId,
    ),
  );
  const activeItineraryDay =
    itineraryView.days.find(
      (day) => day.id === normalizedActiveItineraryDayId,
    ) ?? null;
  const rawMyMember =
    detail.members.find((member) => member.id === detail.my_member_id) ?? null;
  const canEditAllMembers = rawMyMember?.is_owner === true;
  const members = detail.members.map((member) =>
    mapMember(
      member,
      member.id === detail.my_member_id || canEditAllMembers,
      member.id === detail.my_member_id,
      canEditAllMembers && !member.is_owner,
      buildMemberCarrySummary(detail, member.id, enabled.has("food_plan")),
    ),
  );
  const myMember = members.find((member) => member.isMine) ?? null;
  const foodView = buildFoodView(detail, members, activeFoodDayId);
  const foodBudgetSummary = buildFoodBudgetSummary(
    detail,
    enabled.has("food_plan"),
  );
  const optionalOrder = normalizeOptionalSectionOrder(
    detail.plan.enabled_sections,
    preferredOptionalOrder,
    tripType,
  );
  const normalizedSelectedSection =
    isSectionAllowedForTripType(selectedSection, tripType) &&
    enabled.has(selectedSection)
      ? selectedSection
      : fallbackSectionForTripType(tripType);
  const tabs = [
    ...defaultSections,
    ...optionalOrder.filter((key) => enabled.has(key)),
  ].map((key) => ({
    key,
    label: sectionLabel(key),
    id: sectionTabDomId(key),
  }));
  const myGearView = buildMyGearViewForTripType(detail);
  const allSharedGearRows = buildSharedGearRows(detail, members);
  const activeSharedGearCategoryFilter =
    normalizeSharedGearCategoryFilterForRows(
      sharedGearCategoryFilter,
      allSharedGearRows,
    );
  return {
    detail,
    planMetaText: formatPlanMeta(detail),
    canDeletePlan: canEditAllMembers,
    selectedSection: normalizedSelectedSection,
    activeSectionTabId: sectionTabDomId(normalizedSelectedSection),
    sectionTabs: tabs,
    optionalSections: optionalOrder.map((key, index) => ({
      key,
      label: sectionLabel(key),
      enabled: enabled.has(key),
      actionText: enabled.has(key) ? "已添加" : "添加",
      canMoveUp: index > 0,
      canMoveDown: index < optionalOrder.length - 1,
    })),
    optionalSectionOrder: optionalOrder,
    activeItineraryDayId: normalizedActiveItineraryDayId,
    activeItineraryDayIndex,
    activeItineraryDay,
    itineraryDayCount: itineraryView.days.length,
    itineraryDayPagerText: activeItineraryDay
      ? `${activeItineraryDayIndex + 1} / ${itineraryView.days.length}`
      : "",
    canSwitchPrevDay: activeItineraryDayIndex > 0,
    canSwitchNextDay:
      activeItineraryDayIndex >= 0 &&
      activeItineraryDayIndex < itineraryView.days.length - 1,
    members,
    myMember,
    myRoleLabel: myMember?.profile.role_label || "",
    myGearView,
    sharedGearEnabled: isTeamTrip && enabled.has("shared_gear"),
    sharedGearRows: filterSharedGearRows(
      allSharedGearRows,
      activeSharedGearCategoryFilter,
    ),
    sharedGearCategoryFilter: activeSharedGearCategoryFilter,
    sharedGearCategoryOptions: buildSharedGearCategoryOptions(
      allSharedGearRows,
      activeSharedGearCategoryFilter,
    ),
    itineraryDays: itineraryView.days,
    orphanRouteSegments: itineraryView.orphanRouteSegments,
    routeSegments: itineraryView.orphanRouteSegments,
    routeEstimateSummaryText: formatRouteEstimateSummary(detail.plan),
    routeEstimateDetailText: formatRouteEstimateDetail(detail.plan),
    routeEstimateRules: buildRouteEstimateRuleViews(
      detail.plan.route_use_slope_adjustment,
      detail.plan.route_use_high_altitude_adjustment,
    ),
    foodMeals: detail.food_meals,
    foodSupplies: detail.food_supplies || [],
    foodDays: foodView.days,
    activeFoodDayId: foodView.activeDay?.id || "",
    activeFoodDayIndex: foodView.activeDayIndex,
    activeFoodDay: foodView.activeDay,
    foodDayPagerText: foodView.activeDay
      ? `${foodView.activeDayIndex + 1} / ${foodView.days.length}`
      : "",
    canSwitchPrevFoodDay: foodView.activeDayIndex > 0,
    canSwitchNextFoodDay:
      foodView.activeDayIndex >= 0 &&
      foodView.activeDayIndex < foodView.days.length - 1,
    foodSupplyRows: buildFoodSupplyViews(detail.food_supplies || [], members),
    medicalItems: buildMedicalItemViews(detail.medical_items),
    medicalSummary: buildMedicalSummary(detail.medical_items),
    safetyRisks: detail.safety_risks || [],
    rescueContacts: detail.rescue_contacts || [],
    budgetItems: detail.budget_items || [],
    foodBudgetSummary,
    goals: buildGoalViews(detail.goals || []),
    budgetTotalText: formatBudgetTotal(
      detail.budget_items || [],
      foodBudgetSummary.totalCents,
    ),
    budgetPerPersonText: formatBudgetPerPerson(
      detail.budget_items || [],
      detail.members.length,
      foodBudgetSummary.totalCents,
    ),
    budgetSharedGearLabels: [
      "不关联公共装备",
      ...allSharedGearRows.map((item) => item.slotName),
    ],
    foodBlocked: enabled.has("food_plan") && detail.itinerary_days.length === 0,
    sharedResponsibleIndex: defaultMemberIndex(members, myMember?.id),
    safetyResponsibleIndex: defaultMemberIndex(members, myMember?.id),
  };
}

function buildSharedGearRows(
  detail: TripDetail,
  members: MemberView[],
): SharedGearView[] {
  const rows: SharedGearView[] = [];
  const usedRecordIds = new Set<string>();
  const templates = detail.shared_gear_demand_templates || [];
  templates.forEach((template) => {
    const record = detail.shared_gear_items.find((item) => {
      if (usedRecordIds.has(item.id)) {
        return false;
      }
      return sharedGearMatchesTemplate(item, template);
    });
    if (record) {
      usedRecordIds.add(record.id);
    }
    rows.push(mapSharedGearTemplateRow(template, record, members));
  });
  detail.shared_gear_items.forEach((item) => {
    if (!usedRecordIds.has(item.id)) {
      rows.push(mapSharedGearRecordRow(item, members));
    }
  });
  return rows;
}

function mapSharedGearTemplateRow(
  template: SharedGearDemandTemplate,
  item: TripSharedGearDemand | undefined,
  members: MemberView[],
): SharedGearView {
  const slotName = item?.slot_name || template.slot_name;
  return mapSharedGearRow(
    {
      id: item?.id || `template:${template.slot_key}`,
      recordId: item?.id || "",
      slotKey: template.slot_key,
      slotName,
      slotGroup: template.group_label,
      name: slotName,
      category: template.category,
      categoryLabel:
        template.category_label || getGearCategoryLabel(template.category),
      plannedQuantity: item?.planned_quantity ?? template.planned_quantity,
      responsibleMemberId: item?.responsible_member_id || "",
      sourceMemberId: item?.source_member_id || null,
      createdByUserId: item?.created_by_user_id || null,
      sourceGearId: item?.source_gear_id || "",
      concreteName: item ? concreteNameForSharedGear(item) : "",
      hasConcreteGear: item ? isBoundSharedGear(item) : false,
      unitWeightG: item?.unit_weight_g ?? null,
      fieldVersions: item?.field_versions || {},
      isPlaceholder: !item,
      isDefaultSlot: true,
    },
    members,
  );
}

function mapSharedGearRecordRow(
  item: TripSharedGearDemand,
  members: MemberView[],
): SharedGearView {
  const slotName = item.slot_name || item.name;
  return mapSharedGearRow(
    {
      id: item.id,
      recordId: item.id,
      slotKey: item.slot_key || customSharedGearSlotKey(slotName, item.id),
      slotName,
      slotGroup: item.slot_key ? "自定义" : "旧公共装备",
      name: slotName,
      category: item.category,
      categoryLabel: item.category_label || getGearCategoryLabel(item.category),
      plannedQuantity: item.planned_quantity,
      responsibleMemberId: item.responsible_member_id,
      sourceMemberId: item.source_member_id || null,
      createdByUserId: item.created_by_user_id || null,
      sourceGearId: item.source_gear_id || "",
      concreteName: concreteNameForSharedGear(item),
      hasConcreteGear: isBoundSharedGear(item),
      unitWeightG: item.unit_weight_g ?? null,
      fieldVersions: item.field_versions,
      isPlaceholder: false,
      isDefaultSlot: false,
    },
    members,
  );
}

function mapSharedGearRow(
  source: {
    id: string;
    recordId: string;
    slotKey: string;
    slotName: string;
    slotGroup: string;
    name: string;
    category: GearCategory;
    categoryLabel: string;
    plannedQuantity: number;
    responsibleMemberId: string;
    sourceMemberId: string | null;
    createdByUserId: string | null;
    sourceGearId: string;
    concreteName: string;
    hasConcreteGear: boolean;
    unitWeightG: number | null;
    fieldVersions: FieldVersions;
    isPlaceholder: boolean;
    isDefaultSlot: boolean;
  },
  members: MemberView[],
): SharedGearView {
  const plannedQuantity = Math.max(0, source.plannedQuantity || 0);
  const responsible = members.find(
    (member) => member.id === source.responsibleMemberId,
  );
  const sourceMember = members.find(
    (member) => member.id === source.sourceMemberId,
  );
  const creator = members.find(
    (member) => member.user_id === source.createdByUserId,
  );
  return {
    id: source.id,
    recordId: source.recordId,
    name: source.name,
    slotKey: source.slotKey,
    slotName: source.slotName,
    slotGroup: source.slotGroup,
    concreteName: source.concreteName,
    sourceGearId: source.sourceGearId,
    category: source.category,
    categoryLabel: source.categoryLabel,
    plannedQuantity,
    plannedQuantityText: `${plannedQuantity} 件`,
    responsibleMemberId: source.responsibleMemberId,
    responsibleIndex: defaultMemberIndex(members, source.responsibleMemberId),
    responsibleText: responsible?.displayName || "未指定",
    sourceText: source.hasConcreteGear
      ? sourceMember?.displayName || "队伍公共"
      : "待填写",
    createdByText: source.recordId
      ? creator?.displayName || "未知"
      : source.isDefaultSlot
        ? "后端模板"
        : "自定义",
    weightText: source.hasConcreteGear
      ? formatTripWeight((source.unitWeightG || 0) * plannedQuantity)
      : formatTripWeight(0),
    hasConcreteGear: source.hasConcreteGear,
    actionText: source.hasConcreteGear ? "管理" : "填写",
    isDefaultSlot: source.isDefaultSlot,
    isPlaceholder: source.isPlaceholder,
    fieldVersions: source.fieldVersions,
  };
}

function sharedGearMatchesTemplate(
  item: TripSharedGearDemand,
  template: SharedGearDemandTemplate,
): boolean {
  if (item.slot_key) {
    return item.slot_key === template.slot_key;
  }
  return (
    normalizeGearSlotText(item.slot_name || item.name) ===
      normalizeGearSlotText(template.slot_name) &&
    item.category === template.category
  );
}

function isBoundSharedGear(item: TripSharedGearDemand): boolean {
  return Boolean(
    item.concrete_name ||
    item.source_gear_id ||
    (!item.slot_key && !item.slot_name),
  );
}

function concreteNameForSharedGear(item: TripSharedGearDemand): string {
  if (item.concrete_name) {
    return item.concrete_name;
  }
  return isBoundSharedGear(item) ? item.name : "";
}

function findFoodMealView(
  days: FoodDayView[],
  mealId?: string,
): FoodMealView | null {
  if (!mealId) {
    return null;
  }
  for (const day of days) {
    const meal = day.meals.find((item) => item.id === mealId);
    if (meal) {
      return meal;
    }
  }
  return null;
}

function buildSharedGearCategoryOptions(
  rows: SharedGearView[],
  selected: SharedGearCategoryFilterValue,
): SharedGearCategoryOptionView[] {
  const counts = rows.reduce((acc, row) => {
    acc.set(row.category, (acc.get(row.category) || 0) + 1);
    return acc;
  }, new Map<GearCategory, number>());
  const options: SharedGearCategoryOptionView[] = [
    {
      value: "all",
      label: "全部",
      count: rows.length,
      selected: selected === "all",
    },
  ];
  GEAR_CATEGORY_OPTIONS.forEach((item) => {
    const count = counts.get(item.value) || 0;
    if (!count) {
      return;
    }
    options.push({
      value: item.value,
      label: item.label,
      count,
      selected: selected === item.value,
    });
  });
  return options;
}

function filterSharedGearRows(
  rows: SharedGearView[],
  category: SharedGearCategoryFilterValue,
): SharedGearView[] {
  if (category === "all") {
    return rows;
  }
  return rows.filter((row) => row.category === category);
}

function normalizeSharedGearCategoryFilter(
  value: unknown,
): SharedGearCategoryFilterValue {
  if (value === "all") {
    return "all";
  }
  return GEAR_CATEGORY_OPTIONS.some((item) => item.value === value)
    ? (value as GearCategory)
    : "all";
}

function normalizeSharedGearCategoryFilterForRows(
  value: SharedGearCategoryFilterValue,
  rows: SharedGearView[],
): SharedGearCategoryFilterValue {
  if (value === "all") {
    return "all";
  }
  return rows.some((row) => row.category === value) ? value : "all";
}

function buildItineraryView(detail: TripDetail): {
  days: ItineraryDayView[];
  orphanRouteSegments: RouteSegmentView[];
} {
  const routeById = new Map(
    detail.route_segments.map((segment) => [segment.id, segment] as const),
  );
  const linkedRouteIds = new Set<string>();
  const days = detail.itinerary_days.map((day) => {
    const routeSegments = day.time_slots
      .map((slot) => {
        const segmentId = slot.route_segment_id;
        if (!segmentId) {
          return null;
        }
        const segment = routeById.get(segmentId);
        if (!segment) {
          return null;
        }
        linkedRouteIds.add(segmentId);
        return mapRouteSegment(segment, day.id, slot.id);
      })
      .filter((segment): segment is RouteSegmentView => Boolean(segment));
    return mapItineraryDay(day, routeSegments, detail.plan.start_date);
  });
  const orphanRouteSegments = detail.route_segments
    .filter((segment) => !linkedRouteIds.has(segment.id))
    .map((segment) => mapRouteSegment(segment));
  return { days, orphanRouteSegments };
}

function mapItineraryDay(
  day: TripItineraryDay,
  routeSegments: RouteSegmentView[],
  planStartDate?: string | null,
): ItineraryDayView {
  return {
    id: day.id,
    dayIndex: day.day_index,
    title: formatItineraryDayTitle(day),
    dateText: formatItineraryDayDateText(day, planStartDate),
    estimateText: `预计 ${day.estimate_minutes} 分钟`,
    routeSegments,
  };
}

function buildFoodView(
  detail: TripDetail,
  members: MemberView[],
  activeFoodDayId: string,
): {
  days: FoodDayView[];
  activeDayIndex: number;
  activeDay: FoodDayView | null;
} {
  const mealsByDay = new Map<string, TripFoodMeal[]>();
  detail.food_meals.forEach((meal) => {
    const list = mealsByDay.get(meal.itinerary_day_id) || [];
    list.push(meal);
    mealsByDay.set(meal.itinerary_day_id, list);
  });
  const days = detail.itinerary_days.map((day) => {
    const meals = (mealsByDay.get(day.id) || [])
      .slice()
      .sort(
        (left, right) =>
          (FOOD_MEAL_ORDER[left.meal_key] ?? 99) -
            (FOOD_MEAL_ORDER[right.meal_key] ?? 99) ||
          left.created_at.localeCompare(right.created_at),
      )
      .map((meal) => mapFoodMeal(meal, members));
    const activeMeals = meals.filter((meal) => !meal.skipped);
    const filledMeals = activeMeals.filter(
      (meal) => meal.dish_name || meal.items.length,
    ).length;
    const totalWeightG = activeMeals.reduce(
      (total, meal) => total + foodItemsWeightG(meal.items),
      0,
    );
    return {
      id: day.id,
      dayIndex: day.day_index,
      title: formatItineraryDayTitle(day),
      dateText: formatItineraryDayDateText(day, detail.plan.start_date),
      summaryText: `${filledMeals}/${meals.length || 3} 餐已填写 · ${formatTripWeight(totalWeightG)}`,
      meals,
    };
  });
  const activeDayId = days.some((day) => day.id === activeFoodDayId)
    ? activeFoodDayId
    : (days[0]?.id ?? "");
  const activeDayIndex = Math.max(
    0,
    days.findIndex((day) => day.id === activeDayId),
  );
  return {
    days,
    activeDayIndex,
    activeDay: days[activeDayIndex] ?? null,
  };
}

function mapFoodMeal(meal: TripFoodMeal, members: MemberView[]): FoodMealView {
  const items = meal.items.map((item) => mapFoodItem(item, members));
  const responsibleMemberId = meal.responsible_member_id || "";
  const responsible = members.find(
    (member) => member.id === responsibleMemberId,
  );
  const weightG = foodItemsWeightG(items);
  const label = FOOD_MEAL_LABELS[meal.meal_key] || meal.meal_key;
  return {
    ...meal,
    label,
    summaryText: meal.skipped
      ? "已跳过，可随时恢复"
      : meal.dish_name
        ? meal.dish_name
        : "待填写餐食",
    dishText: meal.dish_name || "未填写菜品",
    responsibleMemberId,
    responsibleIndex: defaultMemberIndex(members, responsibleMemberId),
    responsibleText: responsible?.displayName || "未指定",
    itemCountText: `${items.length} 个食材`,
    weightText: formatTripWeight(weightG),
    notesText: meal.notes || "",
    items,
  };
}

function mapFoodItem(item: TripFoodItem, members: MemberView[]): FoodItemView {
  const responsible = members.find(
    (member) => member.id === item.responsible_member_id,
  );
  return {
    ...item,
    weightText:
      item.amount_g == null
        ? "0 g · 待补重量"
        : formatTripWeight(foodRecordWeightG(item)),
    costText: formatFoodCostText(item),
    responsibleText: responsible?.displayName || "未指定",
    notesText: item.notes || "",
  };
}

function buildFoodSupplyViews(
  supplies: TripFoodSupply[],
  members: MemberView[],
): FoodSupplyView[] {
  return supplies.map((item) => {
    const responsibleMemberId = item.responsible_member_id || "";
    const responsible = members.find(
      (member) => member.id === responsibleMemberId,
    );
    return {
      ...item,
      typeText: item.supply_type || "跨餐食材",
      weightText:
        item.amount_g == null
          ? "0 g · 待补重量"
          : formatTripWeight(foodRecordWeightG(item)),
      costText: formatFoodCostText(item),
      responsibleMemberId,
      responsibleIndex: defaultMemberIndex(members, responsibleMemberId),
      responsibleText: responsible?.displayName || "未指定",
      notesText: item.notes || "",
    };
  });
}

function buildMedicalItemViews(items: TripMedicalItem[]): MedicalItemView[] {
  return items.map((item) => {
    const requiredQuantity = normalizeMedicalQuantity(item.required_quantity);
    const packedQuantity = normalizeMedicalQuantity(item.packed_quantity);
    const shortage = Math.max(0, requiredQuantity - packedQuantity);
    const scopeOption = medicalScopeOption(item.scope);
    return {
      ...item,
      typeText: item.item_type || "未分类",
      scopeText: scopeOption.shortLabel,
      scopeClass:
        scopeOption.value === "personal_reminder"
          ? "medical-scope-personal"
          : "medical-scope-public",
      quantityText: `已带 ${packedQuantity} / 计划 ${requiredQuantity}`,
      statusText: shortage > 0 ? `缺 ${shortage}` : "已备齐",
      statusClass: shortage > 0 ? "missing" : "ready",
      suggestedText:
        item.suggested_quantity == null || item.suggested_quantity <= 0
          ? ""
          : `建议 ${item.suggested_quantity}`,
    };
  });
}

function buildMedicalSummary(items: TripMedicalItem[]): MedicalSummaryView {
  const requiredQuantity = items.reduce(
    (total, item) => total + normalizeMedicalQuantity(item.required_quantity),
    0,
  );
  const packedQuantity = items.reduce(
    (total, item) => total + normalizeMedicalQuantity(item.packed_quantity),
    0,
  );
  const shortage = items.reduce(
    (total, item) =>
      total +
      Math.max(
        0,
        normalizeMedicalQuantity(item.required_quantity) -
          normalizeMedicalQuantity(item.packed_quantity),
      ),
    0,
  );
  return {
    hasItems: items.length > 0,
    itemCountText: `${items.length} 项`,
    requiredText: `计划 ${requiredQuantity}`,
    packedText: `已带 ${packedQuantity}`,
    shortageText: `缺 ${shortage}`,
  };
}

function medicalFormFromItem(item: TripMedicalItem): MedicalFormData {
  return {
    name: item.name,
    itemType: item.item_type || "",
    scope: medicalScopeOption(item.scope).value,
    suggestedQuantity:
      item.suggested_quantity == null ? "" : String(item.suggested_quantity),
    requiredQuantity: String(normalizeMedicalQuantity(item.required_quantity)),
    packedQuantity: String(normalizeMedicalQuantity(item.packed_quantity)),
  };
}

function defaultMedicalScopeIndex(scope?: string | null): number {
  const normalized = medicalScopeOption(scope).value;
  const index = MEDICAL_SCOPE_OPTIONS.findIndex(
    (item) => item.value === normalized,
  );
  return index >= 0 ? index : 0;
}

function medicalScopeOption(scope?: string | null) {
  return (
    MEDICAL_SCOPE_OPTIONS.find((item) => item.value === scope) ??
    MEDICAL_SCOPE_OPTIONS[0]
  );
}

function normalizeMedicalQuantity(value?: number | null): number {
  return Math.max(0, Math.round(value || 0));
}

function foodItemsWeightG(
  items: Array<Pick<TripFoodItem, "amount_g">>,
): number {
  return items.reduce((total, item) => total + foodRecordWeightG(item), 0);
}

function foodRecordWeightG(
  item: Pick<TripFoodItem | TripFoodSupply, "amount_g">,
): number {
  return Math.max(0, item.amount_g || 0);
}

function foodRecordCostCents(
  item: Pick<TripFoodItem | TripFoodSupply, "total_price_cents">,
): number {
  return Math.max(0, Number(item.total_price_cents || 0));
}

function formatFoodCostText(
  item: Pick<TripFoodItem | TripFoodSupply, "total_price_cents">,
): string {
  return item.total_price_cents == null
    ? "待补费用"
    : `费用 ${formatMoneyCents(foodRecordCostCents(item))}`;
}

function buildFoodBudgetSummary(
  detail: TripDetail,
  foodPlanEnabled: boolean,
): FoodBudgetSummaryView {
  if (!foodPlanEnabled) {
    return defaultFoodBudgetSummary();
  }
  const activeMealItems = (detail.food_meals || [])
    .filter((meal) => !meal.skipped)
    .flatMap((meal) => meal.items || []);
  const supplies = detail.food_supplies || [];
  const records: Array<
    Pick<TripFoodItem | TripFoodSupply, "total_price_cents">
  > = [...activeMealItems, ...supplies];
  if (!records.length) {
    return defaultFoodBudgetSummary();
  }
  const totalCents = records.reduce(
    (sum, item) => sum + foodRecordCostCents(item),
    0,
  );
  const pendingCount = records.filter(
    (item) => item.total_price_cents == null,
  ).length;
  return {
    visible: true,
    totalCents,
    totalText: formatMoneyCents(totalCents),
    itemCountText: `${records.length} 个食材`,
    pendingText: pendingCount > 0 ? `${pendingCount} 个待补费用` : "费用已纳入",
  };
}

function formatItineraryDayTitle(day: TripItineraryDay): string {
  const title = (day.title || "").trim();
  const dayTitleMatch = title.match(/^第\s*(\d+)\s*天$/);
  if (dayTitleMatch) {
    return `第${dayTitleMatch[1]}天`;
  }
  return title || `第${day.day_index}天`;
}

function formatItineraryDayDateText(
  day: TripItineraryDay,
  planStartDate?: string | null,
): string {
  return (
    deriveItineraryDayDate(planStartDate, day.day_index) ||
    day.date_label ||
    "未设置日期"
  );
}

function deriveItineraryDayDate(
  planStartDate: string | null | undefined,
  dayIndex: number,
): string {
  const start = parseDateOnlyUtc(planStartDate);
  if (start === null || dayIndex < 1) {
    return "";
  }
  return formatDateOnlyUtc(start + (dayIndex - 1) * 86400000);
}

function mapRouteSegment(
  segment: TripRouteSegment,
  dayId = "",
  slotId = "",
): RouteSegmentView {
  return {
    id: segment.id,
    dayId,
    slotId,
    name: segment.name,
    metricChips: [
      `${segment.distance_km} km`,
      `爬升 ${segment.ascent_m} m`,
      `下降 ${segment.descent_m} m`,
    ],
    detailText: [
      segment.checkpoint ? `检查点：${segment.checkpoint}` : "",
      segment.trail_condition ? `路况：${segment.trail_condition}` : "",
    ]
      .filter(Boolean)
      .join(" · "),
    estimateText: `预估耗时 ${segment.final_estimate_minutes} 分钟`,
    altitudeText: formatRouteAltitudeText(segment),
    safetyText: "",
  };
}

function buildRouteEstimateRuleViews(
  useSlope: boolean,
  useHighAltitude: boolean,
): RouteEstimateRuleView[] {
  return ROUTE_ESTIMATE_RULES.map((rule) => {
    const enabled =
      rule.key === "naismith" ||
      (rule.key === "slope" && useSlope) ||
      (rule.key === "altitude" && useHighAltitude);
    return {
      ...rule,
      enabled,
      actionText: rule.locked ? "固定启用" : enabled ? "已启用" : "未启用",
    };
  });
}

function formatRouteEstimateSummary(
  plan: Pick<
    TripDetail["plan"],
    "route_use_slope_adjustment" | "route_use_high_altitude_adjustment"
  >,
): string {
  const parts = ["Naismith"];
  if (plan.route_use_slope_adjustment) {
    parts.push("坡度");
  }
  if (plan.route_use_high_altitude_adjustment) {
    parts.push("高海拔");
  }
  return parts.join(" + ");
}

function formatRouteEstimateDetail(
  plan: Pick<
    TripDetail["plan"],
    | "route_use_slope_adjustment"
    | "route_use_high_altitude_adjustment"
    | "route_start_altitude_m"
  >,
): string {
  const details = ["距离 5 km/h", "爬升 600 m/h"];
  if (plan.route_use_slope_adjustment) {
    details[1] = "坡度分档";
    details.push("陡坡加时");
  } else {
    details.push("下降不额外计时");
  }
  if (plan.route_use_high_altitude_adjustment) {
    const altitude =
      plan.route_start_altitude_m == null
        ? "未设置"
        : `${plan.route_start_altitude_m} m`;
    details.push(`起点海拔 ${altitude}`);
  }
  return details.join(" · ");
}

function formatRouteAltitudeText(segment: TripRouteSegment): string {
  const start = segment.estimated_start_altitude_m;
  const end = segment.estimated_end_altitude_m;
  const highest = segment.estimated_highest_altitude_m;
  const factor = segment.high_altitude_factor;
  if (start == null || end == null || highest == null || factor == null) {
    return "";
  }
  return `海拔 ${start}-${end} m / 最高 ${highest} m / 加成 ×${factor.toFixed(2)}`;
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

function formatDateOnlyUtc(timestamp: number): string {
  const date = new Date(timestamp);
  const year = date.getUTCFullYear();
  const month = `${date.getUTCMonth() + 1}`.padStart(2, "0");
  const day = `${date.getUTCDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function newestItineraryDay(
  previousDays: TripItineraryDay[],
  nextDays: TripItineraryDay[],
): TripItineraryDay | null {
  const previousIds = new Set(previousDays.map((day) => day.id));
  return nextDays.find((day) => !previousIds.has(day.id)) ?? null;
}

function newestRouteSegment(
  previousSegmentIds: Set<string>,
  nextSegments: TripRouteSegment[],
): TripRouteSegment | null {
  return (
    nextSegments.find((segment) => !previousSegmentIds.has(segment.id)) ?? null
  );
}

function findRouteSegmentView(
  data: {
    itineraryDays: ItineraryDayView[];
    orphanRouteSegments: RouteSegmentView[];
  },
  segmentId?: string,
): RouteSegmentView | null {
  if (!segmentId) {
    return null;
  }
  for (const day of data.itineraryDays) {
    const segment = day.routeSegments.find((item) => item.id === segmentId);
    if (segment) {
      return segment;
    }
  }
  return (
    data.orphanRouteSegments.find((segment) => segment.id === segmentId) ?? null
  );
}

function findRouteSegment(
  detail: TripDetail | null,
  segmentId?: string,
): TripRouteSegment | null {
  if (!detail || !segmentId) {
    return null;
  }
  return (
    detail.route_segments.find((segment) => segment.id === segmentId) ?? null
  );
}

function buildMemberCarrySummary(
  detail: TripDetail,
  memberId: string,
  includeFoodWeight: boolean,
) {
  const personalWeightG = sumGearWeight(
    detail.personal_gear_items.filter((item) => item.member_id === memberId),
  );
  const sharedWeightG = sumGearWeight(
    detail.shared_gear_items.filter(
      (item) => item.responsible_member_id === memberId,
    ),
  );
  const foodWeightG = includeFoodWeight
    ? sumFoodWeightForMember(detail, memberId)
    : 0;
  const totalWeightG = personalWeightG + sharedWeightG + foodWeightG;
  return {
    personalWeightText: formatTripWeight(personalWeightG),
    sharedWeightText: formatTripWeight(sharedWeightG),
    showFoodWeight: includeFoodWeight,
    foodWeightText: formatTripWeight(foodWeightG),
    totalWeightText: formatTripWeight(totalWeightG),
    totalWeightG,
    hasCarrySummary: totalWeightG > 0,
  };
}

function sumGearWeight(
  items: Array<
    Pick<
      TripPersonalGearItem | TripSharedGearDemand,
      "planned_quantity" | "unit_weight_g"
    >
  >,
): number {
  return items.reduce(
    (total, item) =>
      total + Math.max(0, item.planned_quantity) * (item.unit_weight_g || 0),
    0,
  );
}

function sumFoodWeightForMember(detail: TripDetail, memberId: string): number {
  const mealWeightG = detail.food_meals.reduce((total, meal) => {
    if (meal.skipped) {
      return total;
    }
    return (
      total +
      meal.items.reduce((mealTotal, item) => {
        const responsibleMemberId =
          item.responsible_member_id || meal.responsible_member_id || "";
        if (responsibleMemberId !== memberId) {
          return mealTotal;
        }
        return mealTotal + foodRecordWeightG(item);
      }, 0)
    );
  }, 0);
  const supplyWeightG = (detail.food_supplies || []).reduce((total, item) => {
    if (item.responsible_member_id !== memberId) {
      return total;
    }
    return total + foodRecordWeightG(item);
  }, 0);
  return mealWeightG + supplyWeightG;
}

function defaultMemberIndex(members: MemberView[], memberId?: string): number {
  const index = members.findIndex((member) => member.id === memberId);
  return index >= 0 ? index : 0;
}

function gearSnapshotPayloadFromGear(gear: MyGearImportChoice) {
  return {
    name: gear.name,
    category: gear.category,
    category_label: gear.categoryText,
    brand: gear.brand ?? null,
    model: gear.model ?? null,
    planned_quantity: 1,
    packed_quantity: 0,
    unit_weight_g: gear.weight_g ?? null,
    notes: null,
  };
}

function sharedGearSlotBasePayload(
  slot: SharedGearView,
  plannedQuantityValue: string,
  responsibleMemberId: string,
): TripRecordPatchRequest {
  const plannedQuantity =
    parsePositiveInteger(plannedQuantityValue, slot.plannedQuantity || 1) || 1;
  return {
    name: slot.slotName,
    template_key: slot.slotKey,
    demand_name: slot.slotName,
    category: slot.category,
    category_label: slot.categoryLabel,
    planned_quantity: plannedQuantity,
    responsible_member_id: responsibleMemberId,
  };
}

function withSharedGearBaseVersions(
  slot: SharedGearView,
  payload: TripRecordPatchRequest,
): TripRecordPatchRequest {
  const baseFieldVersions = Object.keys(payload).reduce<FieldVersions>(
    (acc, field) => {
      if (field !== "base_field_versions" && field !== "force_fields") {
        acc[field] = slot.fieldVersions[field] ?? 0;
      }
      return acc;
    },
    {},
  );
  return {
    ...payload,
    base_field_versions: baseFieldVersions,
  };
}

function customSharedGearSlotKey(name: string, suffix = ""): string {
  const normalized = normalizeGearSlotText(name)
    .replace(/[^a-z0-9_\u4e00-\u9fff]+/gi, "_")
    .replace(/^_+|_+$/g, "");
  return `custom_${normalized || "slot"}_${suffix || Date.now()}`;
}

function normalizeGearSlotText(value: string): string {
  return value.trim().replace(/\s+/g, "").toLowerCase();
}

function defaultSectionsForTripType(tripType: TripType): TripSectionKey[] {
  return tripType === "solo" ? SOLO_DEFAULT_SECTIONS : TEAM_DEFAULT_SECTIONS;
}

function optionalSectionsForTripType(tripType: TripType): TripSectionKey[] {
  return tripType === "solo" ? SOLO_OPTIONAL_SECTIONS : TEAM_OPTIONAL_SECTIONS;
}

function isSectionAllowedForTripType(
  section: TripSectionKey,
  tripType: TripType,
): boolean {
  return [
    ...defaultSectionsForTripType(tripType),
    ...optionalSectionsForTripType(tripType),
  ].includes(section);
}

function fallbackSectionForTripType(tripType: TripType): TripSectionKey {
  return defaultSectionsForTripType(tripType)[0];
}

function sanitizeEnabledSectionsForTripType(
  sections: TripSectionKey[],
  tripType: TripType,
): TripSectionKey[] {
  return sections.filter((section) =>
    isSectionAllowedForTripType(section, tripType),
  );
}

function normalizeOptionalSectionOrder(
  enabledSections: TripSectionKey[],
  preferredOrder: TripSectionKey[] = [],
  tripType: TripType = "team",
): TripSectionKey[] {
  const seen = new Set<TripSectionKey>();
  const ordered: TripSectionKey[] = [];
  const allowedOptionalSections = optionalSectionsForTripType(tripType);
  const append = (section: TripSectionKey) => {
    if (allowedOptionalSections.includes(section) && !seen.has(section)) {
      seen.add(section);
      ordered.push(section);
    }
  };
  preferredOrder.forEach(append);
  enabledSections.forEach(append);
  allowedOptionalSections.forEach(append);
  return ordered;
}

function buildEnabledSectionsPayload(
  enabled: Set<TripSectionKey>,
  optionalOrder: TripSectionKey[],
  tripType: TripType = "team",
): TripSectionKey[] {
  const allowedOptionalSections = new Set(
    optionalSectionsForTripType(tripType),
  );
  return [
    ...defaultSectionsForTripType(tripType),
    ...optionalOrder.filter(
      (section) => allowedOptionalSections.has(section) && enabled.has(section),
    ),
  ];
}

function sameSectionOrder(
  left: TripSectionKey[],
  right: TripSectionKey[],
): boolean {
  return (
    left.length === right.length &&
    left.every((section, index) => section === right[index])
  );
}

function sectionTabDomId(section: TripSectionKey): string {
  return `section-tab-${section}`;
}

function defaultPlanGearForm(): PlanGearFormData {
  return {
    name: "",
    category: DEFAULT_PLAN_GEAR_CATEGORY,
    brand: "",
    model: "",
    plannedQuantity: "1",
    unitWeightG: "",
    notes: "",
  };
}

function defaultSharedSlotForm(): SharedSlotFormData {
  return {
    name: "",
    category: DEFAULT_PLAN_GEAR_CATEGORY,
    plannedQuantity: "1",
  };
}

function defaultFoodMealForm(): FoodMealFormData {
  return {
    mealType: "",
    dishName: "",
    responsibleIndex: 0,
    notes: "",
  };
}

function defaultFoodItemForm(): FoodItemFormData {
  return {
    name: "",
    amountG: "",
    costYuan: "",
    responsibleIndex: 0,
    notes: "",
  };
}

function defaultFoodSupplyForm(): FoodSupplyFormData {
  return {
    name: "",
    supplyType: "",
    amountG: "",
    costYuan: "",
    responsibleIndex: 0,
    notes: "",
  };
}

function defaultMedicalForm(): MedicalFormData {
  return {
    name: "",
    itemType: "",
    scope: "public_first_aid",
    suggestedQuantity: "",
    requiredQuantity: "1",
    packedQuantity: "0",
  };
}

function defaultMedicalSummary(): MedicalSummaryView {
  return {
    hasItems: false,
    itemCountText: "0 项",
    requiredText: "计划 0",
    packedText: "已带 0",
    shortageText: "缺 0",
  };
}

function buildMyGearViewForTripType(
  detail: TripDetail,
): TripMemberGearView | null {
  const sourceView =
    detail.member_gear_views.find(
      (view) => view.member_id === detail.my_member_id,
    ) ?? null;
  if (!sourceView || detail.plan.trip_type !== "solo") {
    return sourceView;
  }
  const items = sourceView.items.filter((item) => item.source === "personal");
  return {
    ...sourceView,
    all_weight_g: sumMemberGearViewWeight(items, "planned_quantity"),
    actual_weight_g: sumMemberGearViewWeight(items, "packed_quantity"),
    items,
  };
}

function sumMemberGearViewWeight(
  items: TripMemberGearView["items"],
  quantityField: "planned_quantity" | "packed_quantity",
): number {
  return items.reduce((total, item) => {
    if (!item.counts_weight) {
      return total;
    }
    return total + Math.max(0, item[quantityField]) * (item.unit_weight_g || 0);
  }, 0);
}

function buildGoalViews(items: TripGoalItem[]): GoalItemView[] {
  return items.map((item, index) => ({
    ...item,
    titleText: `目标${formatChineseOrdinal(index + 1)}`,
  }));
}

function formatChineseOrdinal(value: number): string {
  const digits = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
  const normalized = Math.max(1, Math.floor(value));
  if (normalized <= 10) {
    return normalized === 10 ? "十" : digits[normalized];
  }
  if (normalized < 20) {
    return `十${digits[normalized % 10]}`;
  }
  if (normalized < 100) {
    const tens = Math.floor(normalized / 10);
    const ones = normalized % 10;
    return `${digits[tens]}十${ones ? digits[ones] : ""}`;
  }
  return String(normalized);
}

function defaultFoodBudgetSummary(): FoodBudgetSummaryView {
  return {
    visible: false,
    totalCents: 0,
    totalText: "¥0.00",
    itemCountText: "0 个食材",
    pendingText: "",
  };
}

function normalizeInitialSection(
  value: string | undefined,
): TripSectionKey | null {
  if (!value) {
    return null;
  }
  return SECTION_NAV_ORDER.includes(value as TripSectionKey)
    ? (value as TripSectionKey)
    : null;
}

function mapPackingListChoice(item: GearPackingListSummary): PackingListChoice {
  return {
    ...item,
    metaText: formatPackingMeta(item.route_name, item.duration_label),
    progressText: formatPackingProgress(item),
    weightText: formatGearWeight(item.total_weight_g),
  };
}

function mapMyGearImportChoice(item: GearSummary): MyGearImportChoice {
  const brandModel = [item.brand, item.model].filter(Boolean).join(" · ");
  return {
    ...item,
    categoryText: item.category_label || getGearCategoryLabel(item.category),
    statusText: item.status_label || getGearStatusLabel(item.status),
    weightText: formatGearWeight(item.weight_g),
    quantityText: formatGearQuantity(item.quantity),
    brandModelText: brandModel || "未填写品牌型号",
  };
}

function parsePositiveInteger(value: string, fallback: number): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return fallback;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  const normalized = Math.floor(parsed);
  return normalized > 0 ? normalized : null;
}

function parseOptionalNonNegativeInteger(
  value: string,
): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return undefined;
  }
  return Math.round(parsed);
}

function parseNullableInteger(value: string): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return undefined;
  }
  return Number(trimmed);
}

function parseNullableMoneyCents(value: string): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return undefined;
  }
  return Math.round(parsed * 100);
}

function nullableText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function baseVersionsForRecordFields(
  fields: Record<string, unknown>,
  record: { field_versions: FieldVersions },
): Record<string, number> {
  return Object.keys(fields).reduce<Record<string, number>>(
    (versions, field) => {
      versions[field] = record.field_versions[field] ?? 0;
      return versions;
    },
    {},
  );
}

function pickNextSelectedSection(
  current: TripSectionKey,
  changed: TripSectionKey,
  shouldEnable: boolean,
  enabled: Set<TripSectionKey>,
  tripType: TripType,
): TripSectionKey {
  if (shouldEnable) {
    return changed;
  }
  if (isSectionAllowedForTripType(current, tripType) && enabled.has(current)) {
    return current;
  }
  return fallbackSectionForTripType(tripType);
}

function formatPlanMeta(detail: TripDetail): string {
  return (
    [detail.plan.start_date, detail.plan.end_date]
      .filter(Boolean)
      .join(" 至 ") || "未设置行程时间"
  );
}

function formatBudgetTotal(
  items: TripBudgetItem[],
  extraTotalCents = 0,
): string {
  const total = items.reduce(
    (sum, item) => sum + Number(item.total_price_cents ?? 0),
    extraTotalCents,
  );
  return formatMoneyCents(total);
}

function formatBudgetPerPerson(
  items: TripBudgetItem[],
  fallbackMemberCount: number,
  extraTotalCents = 0,
): string {
  const fallbackSplitCount = Math.max(0, fallbackMemberCount);
  const manualPerPerson = items.reduce((sum, item) => {
    const total = Number(item.total_price_cents ?? 0);
    const splitCount = Math.max(
      0,
      item.split_member_count || fallbackSplitCount,
    );
    return splitCount > 0 ? sum + total / splitCount : sum;
  }, 0);
  const extraPerPerson =
    fallbackSplitCount > 0 ? extraTotalCents / fallbackSplitCount : 0;
  const perPerson = manualPerPerson + extraPerPerson;
  return fallbackSplitCount > 0 ? `人均 ${formatMoneyCents(perPerson)}` : "";
}

function formatMoneyCents(value: number): string {
  return `¥${(value / 100).toFixed(2)}`;
}

function formatMoneyValueInput(cents: number): string {
  return (Math.max(0, cents) / 100).toFixed(2);
}

function mapMember(
  member: TripMember,
  canEdit: boolean,
  isMine: boolean,
  canDelete: boolean,
  carrySummary: ReturnType<typeof buildMemberCarrySummary>,
): MemberView {
  const profileSummary = [
    member.profile.real_name,
    member.profile.gender,
    typeof member.profile.age === "number" ? `${member.profile.age} 岁` : "",
    member.profile.height_cm ? `${member.profile.height_cm} cm` : "",
  ].filter(Boolean);
  const healthSummary = [
    member.profile.blood_type ? `血型 ${member.profile.blood_type}` : "",
    member.profile.medical_history
      ? `既往病：${member.profile.medical_history}`
      : "",
    member.profile.allergy_history
      ? `过敏：${member.profile.allergy_history}`
      : "",
    member.profile.medical_response_note
      ? `处置：${member.profile.medical_response_note}`
      : "",
    member.profile.diet_preference
      ? `饮食：${member.profile.diet_preference}`
      : "",
  ].filter(Boolean);
  return {
    ...member,
    displayName: member.profile.display_name || "队员",
    roleText: member.profile.role_label || "未填写分工",
    contactText:
      member.profile.phone || member.profile.emergency_phone || "未填写电话",
    profileSummaryText: profileSummary.length
      ? profileSummary.join(" · ")
      : "未填写姓名 / 性别 / 年龄 / 身高",
    healthText: healthSummary.length
      ? healthSummary.join(" · ")
      : "未填写血型和健康信息",
    canEdit,
    canDelete,
    isMine,
    ...carrySummary,
  };
}

function defaultMemberEditorForm(): MemberEditorFormData {
  return {
    displayName: "",
    outdoorId: "",
    realName: "",
    gender: "",
    age: "",
    heightCm: "",
    phone: "",
    emergencyContact: "",
    emergencyContactRelationship: "",
    emergencyPhone: "",
    bloodType: "",
    medicalHistory: "",
    allergyHistory: "",
    medicalResponseNote: "",
    dietPreference: "",
    insurancePolicyNo: "",
    insuranceCompanyPhone: "",
    experienceNote: "",
    roleLabel: "",
  };
}

function defaultDayInfoForm(): DayInfoFormData {
  return {
    weather: "",
    highTemperatureC: "",
    lowTemperatureC: "",
    weatherSummary: "",
    weatherNotes: "",
    campName: "",
    campAltitudeM: "",
    campTerrain: "",
    campSlope: "",
    campArea: "",
    campWaterSource: "",
    campNotes: "",
  };
}

function buildMemberEditorForm(member: TripMember): MemberEditorFormData {
  return {
    displayName: member.profile.display_name || "",
    outdoorId: member.profile.outdoor_id || "",
    realName: member.profile.real_name || "",
    gender: member.profile.gender || "",
    age:
      typeof member.profile.age === "number" ? String(member.profile.age) : "",
    heightCm: member.profile.height_cm ? String(member.profile.height_cm) : "",
    phone: member.profile.phone || "",
    emergencyContact: member.profile.emergency_contact || "",
    emergencyContactRelationship:
      member.profile.emergency_contact_relationship || "",
    emergencyPhone: member.profile.emergency_phone || "",
    bloodType: member.profile.blood_type || "",
    medicalHistory: member.profile.medical_history || "",
    allergyHistory: member.profile.allergy_history || "",
    medicalResponseNote: member.profile.medical_response_note || "",
    dietPreference: member.profile.diet_preference || "",
    insurancePolicyNo: member.profile.insurance_policy_no || "",
    insuranceCompanyPhone: member.profile.insurance_company_phone || "",
    experienceNote: member.profile.experience_note || "",
    roleLabel: member.profile.role_label || "",
  };
}

function buildMemberUpdateRequest(
  form: MemberEditorFormData,
  member: TripMember,
): TripRecordPatchRequest | null {
  const displayName = form.displayName.trim();
  if (!displayName) {
    wx.showToast({ title: "请填写显示名", icon: "none" });
    return null;
  }
  const heightCm = parseNullableHeight(form.heightCm);
  if (heightCm === undefined) {
    wx.showToast({ title: "身高需要填写 50-250 之间的整数", icon: "none" });
    return null;
  }
  const age = parseNullableAge(form.age);
  if (age === undefined) {
    wx.showToast({ title: "年龄需要填写 0-120 之间的整数", icon: "none" });
    return null;
  }
  const fields: Record<string, string | number | null> = {
    display_name: displayName,
    outdoor_id: nullableText(form.outdoorId),
    real_name: nullableText(form.realName),
    gender: nullableText(form.gender),
    age,
    height_cm: heightCm,
    phone: nullableText(form.phone),
    emergency_contact: nullableText(form.emergencyContact),
    emergency_contact_relationship: nullableText(
      form.emergencyContactRelationship,
    ),
    emergency_phone: nullableText(form.emergencyPhone),
    blood_type: nullableText(form.bloodType),
    medical_history: nullableText(form.medicalHistory),
    allergy_history: nullableText(form.allergyHistory),
    medical_response_note: nullableText(form.medicalResponseNote),
    diet_preference: nullableText(form.dietPreference),
    insurance_policy_no: nullableText(form.insurancePolicyNo),
    insurance_company_phone: nullableText(form.insuranceCompanyPhone),
    experience_note: nullableText(form.experienceNote),
    role_label: nullableText(form.roleLabel),
  };
  return {
    ...fields,
    base_field_versions: baseVersionsForFields(fields, member),
  };
}

function buildMemberRoleOptionViews(roleLabel: string): MemberRoleOptionView[] {
  const selectedRoles = new Set(splitRoleLabel(roleLabel));
  return MEMBER_ROLE_OPTIONS.map((label) => ({
    label,
    selected: selectedRoles.has(label),
  }));
}

function toggleRoleLabel(roleLabel: string, label: string): string {
  const parts = splitRoleLabel(roleLabel);
  const index = parts.indexOf(label);
  if (index >= 0) {
    parts.splice(index, 1);
  } else {
    parts.push(label);
  }
  return parts.join(" / ");
}

function splitRoleLabel(roleLabel: string): string[] {
  const seen = new Set<string>();
  return roleLabel
    .split(/[\/／、,，;；]+/)
    .map((part) => part.trim())
    .filter((part) => {
      if (!part || seen.has(part)) {
        return false;
      }
      seen.add(part);
      return true;
    });
}

function outdoorProfileToMemberFields(
  profile: OutdoorProfile,
): Record<string, string | number> {
  const fields: Record<string, string | number> = {};
  assignFilledString(fields, "outdoor_id", profile.outdoor_id);
  assignFilledString(fields, "real_name", profile.real_name);
  assignFilledString(fields, "gender", profile.gender);
  const age = calculateAge(profile.birth_date);
  if (age !== null) {
    fields.age = age;
  }
  if (typeof profile.height_cm === "number") {
    fields.height_cm = profile.height_cm;
  }
  assignFilledString(fields, "phone", profile.phone);
  assignFilledString(fields, "emergency_contact", profile.emergency_contact);
  assignFilledString(
    fields,
    "emergency_contact_relationship",
    profile.emergency_contact_relationship,
  );
  assignFilledString(fields, "emergency_phone", profile.emergency_phone);
  assignFilledString(fields, "blood_type", profile.blood_type);
  assignFilledString(fields, "medical_history", profile.medical_history);
  assignFilledString(fields, "allergy_history", profile.allergy_history);
  assignFilledString(
    fields,
    "medical_response_note",
    profile.medical_response_note,
  );
  assignFilledString(fields, "diet_preference", profile.diet_preference);
  assignFilledString(
    fields,
    "insurance_policy_no",
    profile.insurance_policy_no,
  );
  assignFilledString(
    fields,
    "insurance_company_phone",
    profile.insurance_company_phone,
  );
  assignFilledString(fields, "experience_note", profile.experience_note);
  return fields;
}

function baseVersionsForFields(
  fields: Record<string, unknown>,
  member: TripMember,
): Record<string, number> {
  return Object.keys(fields).reduce<Record<string, number>>(
    (versions, field) => {
      versions[field] = member.field_versions[field] ?? 0;
      return versions;
    },
    {},
  );
}

function assignFilledString(
  fields: Record<string, string | number>,
  field: string,
  value?: string | null,
) {
  const trimmed = value?.trim();
  if (trimmed) {
    fields[field] = trimmed;
  }
}

function parseNullableHeight(value: string): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  if (!/^\d+$/.test(trimmed)) {
    return undefined;
  }
  const height = Number(trimmed);
  return height >= 50 && height <= 250 ? height : undefined;
}

function parseNullableAge(value: string): number | null | undefined {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  if (!/^\d+$/.test(trimmed)) {
    return undefined;
  }
  const age = Number(trimmed);
  return age >= 0 && age <= 120 ? age : undefined;
}

function optionIndex(options: string[], value: string): number {
  const index = options.indexOf(value);
  return index >= 0 ? index : 0;
}

function showMutationError(error: unknown) {
  const code = (error as { code?: string }).code;
  if (code === "edit_conflict") {
    wx.showModal({
      title: "同一字段已被队友修改",
      content: "请先下拉刷新查看最新值，再决定是否覆盖。",
      showCancel: false,
    });
    return;
  }
  wx.showToast({ title: getErrorMessage(error), icon: "none" });
}
