import {
  getErrorMessage,
  getOutdoorProfile,
  hasAccessToken,
  isLoginRequiredError,
  updateOutdoorProfile,
  type OutdoorProfile,
  type UpdateOutdoorProfileRequest,
} from "../../../utils/api-profile";
import { loginPageUrl } from "../../../utils/auth-prompt";
import {
  isOffline,
  showOfflineWriteBlockedToast,
} from "../../../utils/network-state";
import { formatLocalDate } from "../../../utils/date-utils";
import { getThemeViewData, syncPageTheme } from "../../../utils/theme";

const GENDER_OPTIONS = ["未填写", "男", "女", "其他"];
const BLOOD_TYPE_OPTIONS = ["未填写", "A", "B", "AB", "O", "其他"];

Page({
  data: {
    loggedIn: hasAccessToken(),
    loading: false,
    saving: false,
    error: "",
    outdoorId: "",
    realName: "",
    gender: "",
    genderOptions: GENDER_OPTIONS,
    genderIndex: 0,
    birthDate: "",
    todayDate: formatLocalDate(),
    heightCm: "",
    phone: "",
    emergencyContact: "",
    emergencyContactRelationship: "",
    emergencyPhone: "",
    bloodType: "",
    bloodTypeOptions: BLOOD_TYPE_OPTIONS,
    bloodTypeIndex: 0,
    medicalHistory: "",
    allergyHistory: "",
    medicalResponseNote: "",
    dietPreference: "",
    insurancePolicyNo: "",
    insuranceCompanyPhone: "",
    ...getThemeViewData(),
  },

  onShow() {
    syncPageTheme(this);
    void this.loadProfile();
  },

  async loadProfile() {
    const loggedIn = hasAccessToken();
    this.setData({ loggedIn, error: "" });
    if (!loggedIn) {
      return;
    }
    this.setData({ loading: true });
    try {
      const response = await getOutdoorProfile();
      this.applyProfile(response.profile);
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ loggedIn: false });
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ loading: false });
    }
  },

  applyProfile(profile: OutdoorProfile) {
    const gender = profile.gender || "";
    const bloodType = profile.blood_type || "";
    this.setData({
      outdoorId: profile.outdoor_id || "",
      realName: profile.real_name || "",
      gender,
      genderIndex: optionIndex(GENDER_OPTIONS, gender),
      birthDate: profile.birth_date || "",
      todayDate: formatLocalDate(),
      heightCm: profile.height_cm ? String(profile.height_cm) : "",
      phone: profile.phone || "",
      emergencyContact: profile.emergency_contact || "",
      emergencyContactRelationship:
        profile.emergency_contact_relationship || "",
      emergencyPhone: profile.emergency_phone || "",
      bloodType,
      bloodTypeIndex: optionIndex(BLOOD_TYPE_OPTIONS, bloodType),
      medicalHistory: profile.medical_history || "",
      allergyHistory: profile.allergy_history || "",
      medicalResponseNote: profile.medical_response_note || "",
      dietPreference: profile.diet_preference || "",
      insurancePolicyNo: profile.insurance_policy_no || "",
      insuranceCompanyPhone: profile.insurance_company_phone || "",
    });
  },

  goLogin() {
    wx.navigateTo({ url: loginPageUrl("/pages/profile/outdoor/index") });
  },

  onInput(event: WechatMiniprogram.Input) {
    const field = event.currentTarget.dataset.field as string | undefined;
    if (!field) {
      return;
    }
    this.setData({ [field]: event.detail.value, error: "" });
  },

  onGenderChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value) || 0;
    this.setData({
      genderIndex: index,
      gender: index === 0 ? "" : GENDER_OPTIONS[index],
      error: "",
    });
  },

  onBirthDateChange(event: WechatMiniprogram.PickerChange) {
    const birthDate = String(event.detail.value || "");
    this.setData({
      birthDate,
      error: "",
    });
  },

  clearBirthDate() {
    this.setData({
      birthDate: "",
      error: "",
    });
  },

  onBloodTypeChange(event: WechatMiniprogram.PickerChange) {
    const index = Number(event.detail.value) || 0;
    this.setData({
      bloodTypeIndex: index,
      bloodType: index === 0 ? "" : BLOOD_TYPE_OPTIONS[index],
      error: "",
    });
  },

  async saveProfile() {
    if (this.data.saving) {
      return;
    }
    if (!hasAccessToken()) {
      this.goLogin();
      return;
    }
    if (isOffline()) {
      showOfflineWriteBlockedToast();
      return;
    }
    const heightCm = parseHeightCm(this.data.heightCm);
    if (heightCm === undefined) {
      this.setData({ error: "身高需要填写 50-250 之间的整数" });
      return;
    }
    const request: UpdateOutdoorProfileRequest = {
      outdoor_id: nullableText(this.data.outdoorId),
      real_name: nullableText(this.data.realName),
      gender: nullableText(this.data.gender),
      birth_date: nullableText(this.data.birthDate),
      height_cm: heightCm,
      phone: nullableText(this.data.phone),
      emergency_contact: nullableText(this.data.emergencyContact),
      emergency_contact_relationship: nullableText(
        this.data.emergencyContactRelationship,
      ),
      emergency_phone: nullableText(this.data.emergencyPhone),
      blood_type: nullableText(this.data.bloodType),
      medical_history: nullableText(this.data.medicalHistory),
      allergy_history: nullableText(this.data.allergyHistory),
      medical_response_note: nullableText(this.data.medicalResponseNote),
      diet_preference: nullableText(this.data.dietPreference),
      insurance_policy_no: nullableText(this.data.insurancePolicyNo),
      insurance_company_phone: nullableText(this.data.insuranceCompanyPhone),
    };
    this.setData({ saving: true, error: "" });
    try {
      const response = await updateOutdoorProfile(request);
      this.applyProfile(response.profile);
      wx.showToast({ title: "户外资料已保存", icon: "success" });
    } catch (error) {
      if (isLoginRequiredError(error)) {
        this.setData({ loggedIn: false });
        this.goLogin();
        return;
      }
      this.setData({ error: getErrorMessage(error) });
    } finally {
      this.setData({ saving: false });
    }
  },
});

function nullableText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function parseHeightCm(value: string): number | null | undefined {
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

function optionIndex(options: string[], value: string): number {
  const index = options.indexOf(value);
  return index >= 0 ? index : 0;
}
