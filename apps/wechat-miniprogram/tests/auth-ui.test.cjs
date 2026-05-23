const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const appRoot = path.resolve(__dirname, "..");
const miniRoot = path.join(appRoot, "miniprogram");

function read(rel) {
  return fs.readFileSync(path.join(miniRoot, rel), "utf8");
}

test("register page is available from the mini program page registry", () => {
  const config = JSON.parse(read("app.json"));
  assert.ok(config.pages.includes("pages/register/index"));
});

test("login page offers WeChat account email-code and password-reset entry points", () => {
  const wxml = read("pages/login/index.wxml");
  const ts = read("pages/login/index.ts");
  const pageSource = `${wxml}
${ts}`;
  assert.match(wxml, /微信登录/);
  assert.match(wxml, /账号登录/);
  assert.match(wxml, /邮箱登录/);
  assert.doesNotMatch(wxml, /使用微信身份快速进入/);
  assert.match(wxml, /找回密码/);
  assert.match(wxml, /重设密码并登录/);
  assert.match(wxml, /placeholder="账号或邮箱"/);
  assert.match(wxml, /placeholder="邮箱验证码"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(wxml, /注册账号/);
  assert.match(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
  assert.match(pageSource, /markWechatProfilePromptPending/);
  assert.match(pageSource, /afterLoginSuccess\("\/pages\/index\/index"\)/);
  assert.doesNotMatch(wxml, /open-type="chooseAvatar"/);
  assert.doesNotMatch(wxml, /type="nickname"/);
  assert.doesNotMatch(wxml, /导入并登录/);
  assert.match(pageSource, /sendLoginCode/);
  assert.match(pageSource, /sendResetCode/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
});

test("home page prompts to import WeChat profile after login", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /type="nickname"/);
  assert.match(wxml, /导入资料/);
  assert.match(wxml, /跳过/);
  assert.match(pageSource, /showWechatProfilePromptIfNeeded/);
  assert.match(pageSource, /importWechatProfile/);
  assert.match(pageSource, /skipWechatProfileImport/);
  assert.match(pageSource, /uploadWechatAvatar/);
  assert.match(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.match(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
});

test("profile page renders stored WeChat nickname and avatar", () => {
  const wxml = read("pages/profile/index.wxml");
  const wxss = read("pages/profile/index.wxss");
  const ts = read("pages/profile/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /accountProfile\.avatarUrl/);
  assert.match(wxml, /accountProfile\.displayName/);
  assert.match(wxml, /wx:if="{{loggedIn}}"/);
  assert.match(wxml, /account-avatar-image/);
  assert.match(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /bindchooseavatar="onChooseWechatAvatar"/);
  assert.doesNotMatch(wxml, /account-avatar-badge/);
  assert.match(wxml, /bindtap="openUserSettings"/);
  assert.match(wxml, /account-settings-entry/);
  assert.match(wxss, /\.account-main\s*\{[\s\S]*flex:\s*1;/);
  assert.match(wxss, /\.account-settings-entry\s*\{[\s\S]*flex:\s*1;/);
  assert.doesNotMatch(wxml, /type="nickname"/);
  assert.doesNotMatch(wxml, /bindtap="openNicknameModal"/);
  assert.doesNotMatch(wxml, /bindtap="openEmailBindingModal"/);
  assert.doesNotMatch(wxml, /账号状态/);
  assert.doesNotMatch(wxml, /可以管理自己的装备清单/);
  assert.doesNotMatch(wxml, /accountProfile\.emailText/);
  assert.doesNotMatch(wxml, /已绑定/);
  assert.match(wxml, /设置与帮助/);
  assert.match(wxml, /绳结离线缓存/);
  assert.match(wxml, /意见反馈/);
  assert.match(wxml, /版本信息/);
  assert.match(wxml, /versionInfoDesc/);
  assert.match(wxml, /bindtap="openVersionInfoModal"/);
  assert.match(wxml, /关于寻径星野/);
  assert.match(wxml, /已缓存绳结/);
  assert.match(wxml, /已缓存/);
  assert.match(wxml, /未缓存/);
  assert.match(wxml, /全部绳结/);
  assert.match(wxml, /删除全部/);
  assert.match(wxml, /cachedKnots\.length/);
  assert.match(wxml, /cachedKnotsInfo\.cachedCount/);
  assert.match(wxml, /cachedKnotsInfo\.uncachedCountText/);
  assert.match(wxml, /bindtap="openCachedKnotsModal"/);
  assert.match(wxml, /bindtap="goKnotOfflineCache"/);
  assert.match(wxml, /bindtap="openCachedKnotDetail"/);
  assert.match(wxml, /catchtap="removeCachedKnot"/);
  assert.match(wxml, /bindtap="clearCachedKnots"/);
  assert.match(wxml, /bindtap="closeCachedKnotsModal"/);
  assert.match(wxml, /bindtap="openFeedbackModal"/);
  assert.match(wxml, /versionModalVisible/);
  assert.match(wxml, /clientVersions/);
  assert.match(wxml, /最新/);
  assert.match(wxml, /bindtap="retryClientVersions"/);
  assert.match(wxml, /bindtap="closeVersionInfoModal"/);
  assert.match(wxml, /bindtap="openAboutModal"/);
  assert.match(wxml, /bindchange="onFeedbackCategoryChange"/);
  assert.match(wxml, /图片附件/);
  assert.match(wxml, /添加图片/);
  assert.match(wxml, /feedbackImages/);
  assert.match(wxml, /bindtap="chooseFeedbackImages"/);
  assert.match(wxml, /bindtap="previewFeedbackImage"/);
  assert.match(wxml, /catchtap="removeFeedbackImage"/);
  assert.match(wxml, /bindtap="submitFeedback"/);
  assert.match(wxml, /feedbackSuccessVisible/);
  assert.match(wxml, /feedback-success-card/);
  assert.match(wxml, /feedbackSuccessMessage/);
  assert.match(wxml, /feedbackSuccessSecondsRemaining/);
  assert.match(wxml, /秒后自动关闭/);
  assert.match(wxml, /bindtap="closeFeedbackSuccess"/);
  assert.match(wxml, /我知道了/);
  assert.doesNotMatch(
    wxml,
    /你的反馈会进入后台处理，我们会优先看问题和内容纠错。/,
  );
  assert.match(
    pageSource,
    /感谢你的反馈。反馈内容会进入后台处理，我们会及时查看并持续改进。你的建议会让寻径星野变得更好。/,
  );
  assert.match(pageSource, /FEEDBACK_SUCCESS_VISIBLE_MS = 10_000/);
  assert.match(pageSource, /FEEDBACK_SUCCESS_TICK_MS = 1_000/);
  assert.match(pageSource, /feedbackSuccessTimer/);
  assert.match(pageSource, /feedbackSuccessSecondsRemaining/);
  assert.match(pageSource, /showFeedbackSuccess/);
  assert.match(pageSource, /scheduleFeedbackSuccessCountdown/);
  assert.match(pageSource, /closeFeedbackSuccess/);
  assert.match(pageSource, /hideFeedbackSuccess/);
  assert.match(pageSource, /clearFeedbackSuccessTimer/);
  assert.match(pageSource, /setTimeout/);
  assert.match(pageSource, /onHide\(\)/);
  assert.match(pageSource, /onUnload\(\)/);
  assert.doesNotMatch(pageSource, /wx\.showToast\(\{ title: "反馈已提交"/);
  assert.match(wxml, /为户外爱好者准备的出行、装备与技能工具。/);
  assert.match(wxml, /about-modal-card/);
  assert.match(wxml, /about-hero/);
  assert.match(wxml, /🏕️ 寻径星野/);
  assert.match(wxml, /把每次出发前的准备，整理得更安心。/);
  assert.match(wxml, /🧭/);
  assert.match(wxml, /出发准备/);
  assert.match(
    wxml,
    /寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。/,
  );
  assert.match(wxml, /🎒/);
  assert.match(wxml, /山野陪伴/);
  assert.match(
    wxml,
    /它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。/,
  );
  assert.match(wxml, /✨/);
  assert.match(wxml, /作者的话/);
  assert.match(
    wxml,
    /这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。寻径星野会永久免费，无广告，不做打扰用户的商业化设计。/,
  );
  assert.doesNotMatch(wxml, /运行版本/);
  assert.doesNotMatch(wxml, /版本号/);
  assert.doesNotMatch(wxml, /aboutInfo\.envText/);
  assert.doesNotMatch(wxml, /aboutInfo\.versionText/);
  assert.doesNotMatch(wxml, /点这里后，在下方选择“用微信昵称”/);
  assert.doesNotMatch(wxml, /导入微信名称/);
  assert.doesNotMatch(wxml, /使用自定义名称/);
  assert.doesNotMatch(wxml, /导入昵称/);
  assert.doesNotMatch(wxml, /bindtap="saveNickname"/);
  assert.doesNotMatch(wxml, /bindtap="importWechatNickname"/);
  assert.doesNotMatch(wxml, /bindtap="useCustomNickname"/);
  assert.match(pageSource, /buildAccountProfile/);
  assert.match(pageSource, /getCurrentUser/);
  assert.doesNotMatch(pageSource, /sendBindEmailCode/);
  assert.doesNotMatch(pageSource, /bindEmailToCurrentAccount/);
  assert.doesNotMatch(pageSource, /getUserProfile/);
  assert.doesNotMatch(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.doesNotMatch(pageSource, /WechatNicknameSubmitEvent/);
  assert.doesNotMatch(pageSource, /nicknameEditMode/);
  assert.doesNotMatch(pageSource, /saveNicknameValue/);
  assert.match(pageSource, /uploadWechatAvatar/);
  assert.match(pageSource, /listClientVersions\("wechat_miniprogram"/);
  assert.match(pageSource, /refreshClientVersionSummary/);
  assert.match(pageSource, /openVersionInfoModal/);
  assert.match(pageSource, /loadClientVersions/);
  assert.match(pageSource, /uploadFeedbackImage/);
  assert.match(pageSource, /chooseMedia/);
  assert.match(pageSource, /wx\.chooseImage/);
  assert.match(pageSource, /createFeedback/);
  assert.match(pageSource, /getKnotOfflineCacheInventory/);
  assert.match(pageSource, /refreshKnotOfflineCacheInventory/);
  assert.match(pageSource, /deleteCachedKnot/);
  assert.match(pageSource, /clearKnotOfflineCache/);
  assert.match(
    pageSource,
    /wx\.switchTab\(\{ url: "\/pages\/skills\/index" \}\)/,
  );
  assert.match(pageSource, /wx\.getAccountInfoSync\(\)/);
  assert.match(pageSource, /wx\.getSystemInfoSync\(\)\.model/);
  assert.match(pageSource, /client_platform: "wechat_miniprogram"/);
  assert.match(pageSource, /image_ids: imageIds/);
  assert.match(pageSource, /isNotFoundApiError/);
  assert.doesNotMatch(pageSource, /normalizeProfileNickname\(user\.nickname\)/);
  assert.match(pageSource, /头像保存暂不可用/);
  assert.match(
    pageSource,
    /wx\.navigateTo\(\{ url: "\/pages\/profile\/settings\/index" \}\)/,
  );
});

test("profile settings page owns account identity and password actions", () => {
  const appConfig = JSON.parse(read("app.json"));
  const wxml = read("pages/profile/settings/index.wxml");
  const ts = read("pages/profile/settings/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.ok(appConfig.pages.includes("pages/profile/settings/index"));
  assert.match(wxml, /用户设置/);
  assert.match(wxml, /修改名称/);
  assert.match(wxml, /修改邮箱/);
  assert.match(wxml, /绑定邮箱/);
  assert.match(wxml, /修改密码/);
  assert.match(wxml, /头像可回到“我的”页点击头像修改/);
  assert.doesNotMatch(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /type="nickname"/);
  assert.match(wxml, /点这里后，在下方选择“用微信昵称”/);
  assert.match(wxml, /bindsubmit="saveWechatNickname"/);
  assert.match(wxml, /bindnicknamereview="onWechatNicknameReview"/);
  assert.match(wxml, /bindtap="openEmailBindingModal"/);
  assert.match(wxml, /bindtap="sendEmailBindingCode"/);
  assert.match(wxml, /bindtap="submitEmailBinding"/);
  assert.match(wxml, /bindtap="openPasswordModal"/);
  assert.match(wxml, /bindtap="sendPasswordCode"/);
  assert.match(wxml, /bindtap="submitPasswordChange"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.match(pageSource, /sendBindEmailCode/);
  assert.match(pageSource, /bindEmailToCurrentAccount/);
  assert.match(pageSource, /sendPasswordResetCode/);
  assert.match(pageSource, /resetPassword/);
  assert.match(pageSource, /openEmailBindingModal/);
  assert.match(pageSource, /先绑定邮箱/);
  assert.match(pageSource, /buildAccountProfile/);
  assert.match(pageSource, /WechatNicknameSubmitEvent/);
});

test("register page captures account email verification and password confirmation", () => {
  const wxml = read("pages/register/index.wxml");
  const ts = read("pages/register/index.ts");
  const pageSource = `${wxml}
${ts}`;
  assert.match(wxml, /创建账号/);
  assert.match(wxml, /placeholder="账号"/);
  assert.match(wxml, /placeholder="邮箱"/);
  assert.match(wxml, /placeholder="邮箱验证码"/);
  assert.match(wxml, /placeholder="密码"/);
  assert.match(wxml, /placeholder="确认密码"/);
  assert.match(pageSource, /获取验证码/);
  assert.match(wxml, /注册并登录/);
  assert.match(wxml, /返回登录/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
});

test("register page styles include dark theme surfaces", () => {
  const wxss = read("pages/register/index.wxss");
  assert.match(wxss, /theme-dark/);
  assert.match(wxss, /var\(--surface-color\)/);
  assert.match(wxss, /var\(--brand-color\)/);
});

test("home gear summary aligns logged-out and logged-in card surfaces", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const wxss = read("pages/index/index.wxss");
  const guestInlineBlock =
    wxss.match(/\.guest-inline \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(wxml, /showLoginForGearSummary/);
  assert.match(ts, /LOCKED_GEAR_STATS/);
  assert.match(ts, /登录后可见/);
  assert.match(ts, /buildHeroStatusText/);
  assert.doesNotMatch(ts, /登录后快速记录装备/);
  assert.doesNotMatch(ts, /我的装备已保存/);
  assert.match(ts, /value: "—"/);
  assert.match(
    guestInlineBlock,
    /border: 1rpx solid var\(--soft-border-color\)/,
  );
  assert.match(guestInlineBlock, /background: var\(--control-bg\)/);
  assert.doesNotMatch(guestInlineBlock, /var\(--notice-bg\)/);
});

test("gear page logged-out and logged-in cards share surface tokens", () => {
  const wxss = read("pages/gears/index.wxss");
  const cardSurfaceBlock =
    wxss.match(/\.tab-card,[\s\S]*?\.gear-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const metricBlock = wxss.match(/\.metric \{[\s\S]*?\n\}/)?.[0] ?? "";
  const metricValueBlock =
    wxss.match(/\.metric-value \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(cardSurfaceBlock, /border: 1rpx solid var\(--border-color\)/);
  assert.match(cardSurfaceBlock, /background: var\(--surface-color\)/);
  assert.match(cardSurfaceBlock, /box-shadow: var\(--shadow-soft\)/);
  assert.doesNotMatch(cardSurfaceBlock, /background: #ffffff/);
  assert.match(metricBlock, /background: var\(--control-bg\)/);
  assert.match(metricValueBlock, /color: var\(--text-color\)/);
});

test("gear form cancel falls back when there is no previous page", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");
  const formActionsBlock =
    wxss.match(/\.form-actions \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(wxml, /bindtap="cancel"/);
  assert.ok(
    wxml.lastIndexOf('class="form-actions"') >
      wxml.lastIndexOf('class="form-card"'),
  );
  assert.doesNotMatch(wxml, /footer-actions/);
  assert.match(wxss, /\.form-actions \{/);
  assert.match(wxss, /env\(safe-area-inset-bottom\)/);
  assert.doesNotMatch(formActionsBlock, /position: fixed/);
  assert.match(ts, /getCurrentPages\(\)\.length > 1/);
  assert.match(ts, /wx\.navigateBack\(\)/);
  assert.match(ts, /wx\.switchTab\(\{ url: "\/pages\/gears\/index" \}\)/);
  assert.match(ts, /wx\.redirectTo\(\{/);
  assert.match(ts, /\/pages\/gears\/detail\/index\?id=/);
});

test("gear form offers purchase location presets and custom input", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");

  assert.match(wxml, /选择或输入购买渠道/);
  assert.match(wxml, /purchaseLocationSheetVisible/);
  assert.match(wxml, /purchaseLocationOptions/);
  assert.match(wxml, /自定义输入/);
  assert.match(wxml, /自定义购买渠道/);
  assert.match(wxml, /bindtap="openPurchaseLocationSheet"/);
  assert.match(wxml, /bindtap="selectPurchaseLocation"/);
  assert.match(wxml, /bindtap="openCustomPurchaseLocation"/);
  assert.match(wxml, /bindtap="saveCustomPurchaseLocation"/);
  assert.doesNotMatch(wxml, /data-field="purchaseLocation"/);
  assert.match(ts, /PURCHASE_LOCATION_OPTIONS/);
  assert.match(ts, /selectPurchaseLocation/);
  assert.match(ts, /saveCustomPurchaseLocation/);
  assert.match(ts, /cancelCustomPurchaseLocation/);
  assert.match(ts, /customPurchaseLocationText\.trim\(\)/);
  assert.match(wxss, /\.purchase-location-sheet/);
  assert.match(wxss, /\.purchase-location-option\.selected/);
});

test("gear form offers colored tag chips and suggestions", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");
  const utils = read("utils/gear-utils.ts");

  assert.match(wxml, /添加标签/);
  assert.match(wxml, /tagSheetVisible/);
  assert.match(wxml, /tagSuggestions/);
  assert.match(wxml, /tagColorOptions/);
  assert.match(wxml, /tag-chip/);
  assert.match(wxml, /tag-color-option/);
  assert.match(wxml, /随机/);
  assert.doesNotMatch(wxml, /data-field="tagsText"/);
  assert.doesNotMatch(wxml, /用逗号分隔/);
  assert.match(ts, /getCachedGearTagSuggestions/);
  assert.match(ts, /selectTagSuggestion/);
  assert.match(ts, /saveCustomTag/);
  assert.match(ts, /removeTag/);
  assert.match(utils, /export type GearTagColor =/);
  assert.match(utils, /GEAR_TAG_COLOR_OPTIONS/);
  assert.match(wxss, /\.tag-color-teal/);
  assert.match(wxss, /\.tag-color-blue/);
});

test("gear atlas submission moved from gear form to detail and atlas pages", () => {
  const formWxml = read("pages/gears/form/index.wxml");
  const detailWxml = read("pages/gears/detail/index.wxml");
  const detailTs = read("pages/gears/detail/index.ts");
  const atlasListWxml = read("pages/gear-atlas/index.wxml");
  const atlasSubmitWxml = read("pages/gear-atlas/submit/index.wxml");
  const appJson = read("app.json");

  assert.doesNotMatch(formWxml, /共享给搭子参考/);
  assert.doesNotMatch(formWxml, /申请共享/);
  assert.doesNotMatch(formWxml, /confirmEnableShare/);
  assert.match(detailWxml, /装备图鉴投稿/);
  assert.match(detailWxml, /投稿到图鉴/);
  assert.match(detailTs, /submitGearToAtlas/);
  assert.match(
    detailTs,
    /只会复制分类、名称、品牌、型号、描述、重量、官方价和详细信息/,
  );
  assert.match(atlasListWxml, /装备图鉴/);
  assert.match(atlasSubmitWxml, /提交审核/);
  assert.match(appJson, /pages\/gear-atlas\/index/);
});

test("gear form exposes selectable weight and spec units", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const ts = read("pages/gears/form/index.ts");
  const utils = read("utils/gear-utils.ts");

  assert.match(wxml, /<view class="field-label">重量<\/view>/);
  assert.match(wxml, /weightUnitLabels/);
  assert.match(wxml, /bindchange="onWeightUnitChange"/);
  assert.doesNotMatch(wxml, /重量（kg）/);
  assert.match(ts, /GEAR_WEIGHT_UNIT_OPTIONS/);
  assert.match(ts, /onWeightUnitChange/);
  assert.match(
    utils,
    /export type GearWeightUnit = "kg" \| "g" \| "lb" \| "oz"/,
  );
  assert.match(utils, /const CAPACITY_UNITS = \["L", "ml", "fl oz"\]/);
  assert.match(utils, /const LOAD_UNITS = \["kg", "g", "lb"\]/);
  assert.match(utils, /const LENGTH_UNITS = \["cm", "m", "mm", "in"\]/);
  assert.match(utils, /const BACK_LENGTH_UNITS = \["cm", "in"\]/);
  assert.match(
    utils,
    /const BACKPACK_SIZE_UNITS = \["", "XS", "S", "M", "L", "XL", "XXL", "均码"\]/,
  );
  assert.match(wxml, /choice-value/);
  assert.match(wxml, /item\.choiceOnly/);
  assert.match(
    utils,
    /const SHOE_SIZE_OR_LENGTH_UNITS = \["cm", "EU", "US", "UK", "in"\]/,
  );
});
