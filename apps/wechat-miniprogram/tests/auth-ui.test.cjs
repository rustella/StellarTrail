const test = require("node:test");
const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const appRoot = path.resolve(__dirname, "..");
const miniRoot = path.join(appRoot, "miniprogram");

function read(rel) {
  return fs.readFileSync(path.join(miniRoot, rel), "utf8");
}

test("register page is available from the mini program page registry", () => {
  const config = JSON.parse(read("app.json"));
  assert.equal(config.lazyCodeLoading, "requiredComponents");
  assert.ok(config.pages.includes("pages/register/index"));
});

test("login page offers phone WeChat account email-code and password-reset entry points", () => {
  const wxml = read("pages/login/index.wxml");
  const ts = read("pages/login/index.ts");
  const pageSource = `${wxml}
${ts}`;
  assert.match(wxml, /手机登录/);
  assert.match(wxml, /验证码登录/);
  assert.match(wxml, /手机号密码登录/);
  assert.match(wxml, /微信登录/);
  assert.match(wxml, /账号登录/);
  assert.match(wxml, /邮箱登录/);
  assert.match(wxml, /placeholder="11 位大陆手机号"/);
  assert.match(wxml, /placeholder="短信验证码"/);
  assert.doesNotMatch(wxml, /使用微信身份快速进入/);
  assert.match(wxml, /找回密码/);
  assert.match(wxml, /重设密码并登录/);
  assert.match(wxml, /placeholder="账号、手机号或邮箱"/);
  assert.match(wxml, /placeholder="邮箱验证码"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(wxml, /注册账号/);
  assert.match(wxml, /登录后可以保存和同步个人装备/);
  assert.doesNotMatch(wxml, /不登录也可以先看装备图鉴/);
  assert.doesNotMatch(wxml, /绳结教学/);
  assert.match(pageSource, /this\.afterLoginSuccess\(\)/);
  assert.match(pageSource, /navigateToGuestFallback/);
  assert.match(ts, /loginMode: "phone"/);
  assert.match(ts, /sendSmsLoginCode/);
  assert.match(ts, /loginWithSmsCode/);
  assert.match(ts, /phoneLoginErrorMessage/);
  assert.doesNotMatch(
    pageSource,
    /afterLoginSuccess\("\/pages\/index\/index"\)/,
  );
  assert.doesNotMatch(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
  assert.doesNotMatch(pageSource, /markWechatProfilePromptPending/);
  assert.doesNotMatch(wxml, /open-type="chooseAvatar"/);
  assert.doesNotMatch(wxml, /type="nickname"/);
  assert.doesNotMatch(wxml, /导入并登录/);
  assert.match(pageSource, /sendLoginCode/);
  assert.match(pageSource, /sendResetCode/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
});

test("guest users can browse home and the gear atlas", () => {
  const navigationTs = read("utils/navigation.ts");
  const homeTs = read("pages/index/index.ts");
  const homeWxml = read("pages/index/index.wxml");
  const skillsTs = read("pages/skills/index.ts");
  const skillDetailTs = read("pages/skills/detail/index.ts");
  const profileTs = read("pages/profile/index.ts");
  const gearTs = read("pages/gears/index.ts");
  const gearWxml = read("pages/gears/index.wxml");
  const gearWxss = read("pages/gears/index.wxss");
  const tripsWxml = read("pages/trips/index.wxml");
  const tripsTs = read("pages/trips/index.ts");

  assert.match(
    navigationTs,
    /GUEST_FALLBACK_PAGE = "\/pages\/gear-atlas\/index"/,
  );
  assert.match(navigationTs, /"\/pages\/index\/index"/);
  assert.match(navigationTs, /"\/pages\/gears\/index"/);
  assert.match(navigationTs, /"\/pages\/trips\/index"/);
  assert.match(navigationTs, /"\/pages\/skills\/index"/);
  assert.match(navigationTs, /"\/pages\/profile\/index"/);
  assert.match(navigationTs, /"\/pages\/gear-atlas\/detail\/index"/);
  assert.match(navigationTs, /"\/pages\/login\/index"/);
  assert.match(navigationTs, /"\/pages\/register\/index"/);
  assert.match(navigationTs, /"\/pages\/profile\/roadmap\/index"/);
  assert.match(
    read("pages/login/index.ts"),
    /isGuestAccessiblePage\(this\.data\.redirect\)/,
  );
  assert.doesNotMatch(homeTs, /navigateToGuestFallback/);
  assert.match(homeTs, /GUEST_HERO_STATUS_TEXT = "装备状态待同步"/);
  assert.doesNotMatch(homeTs, /未登录也可先浏览/);
  assert.doesNotMatch(homeWxml, /可以先浏览装备图鉴/);
  assert.doesNotMatch(skillsTs, /navigateToGuestFallback/);
  assert.match(
    skillsTs,
    /message: "登录并同意绳结教程免责声明后，可以查看绳结列表。"/,
  );
  assert.match(skillDetailTs, /navigateToGuestFallback/);
  assert.doesNotMatch(profileTs, /navigateToGuestFallback/);
  assert.doesNotMatch(profileTs, /退出后仍可浏览装备图鉴和绳结教学/);
  assert.doesNotMatch(profileTs, /退出后仍可浏览装备图鉴/);
  assert.match(gearWxml, /装备图鉴/);
  assert.match(gearWxml, /goGearAtlas/);
  assert.match(gearWxml, /goPackingLists/);
  assert.match(gearTs, /emptyText: "登录后才能管理装备"/);
  assert.match(gearWxml, /\{\{emptySubtitle\}\}/);
  assert.match(gearWxml, /\{\{emptyActionText\}\}/);
  assert.doesNotMatch(gearWxml, /wx:if="\{\{!isLoggedIn\}\}"/);
  assert.doesNotMatch(gearWxml, /未登录也可先浏览/);
  assert.doesNotMatch(gearWxml, /可以先看装备图鉴/);
  assert.doesNotMatch(gearWxml, /绳结教学/);
  assert.doesNotMatch(gearWxss, /\.guest-quick-entry/);
  assert.match(tripsWxml, /管理单人行程与组队协作，出发前准备更清晰。/);
  assert.doesNotMatch(tripsWxml, /历史经历都从这里管理/);
  assert.doesNotMatch(tripsWxml, /查看装备图鉴/);
  assert.match(tripsTs, /登录后可以加入多人行程/);
  assert.doesNotMatch(tripsTs, /navigateToGuestFallback/);
});

test("home page does not automatically prompt to import WeChat profile", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.doesNotMatch(wxml, /导入微信资料/);
  assert.doesNotMatch(wxml, /导入资料/);
  assert.doesNotMatch(wxml, /可以导入微信头像和昵称/);
  assert.doesNotMatch(wxml, /type="nickname"/);
  assert.doesNotMatch(wxml, /bindchooseavatar="onChooseWechatAvatar"/);
  assert.doesNotMatch(pageSource, /showWechatProfilePromptIfNeeded/);
  assert.doesNotMatch(pageSource, /importWechatProfile/);
  assert.doesNotMatch(pageSource, /skipWechatProfileImport/);
  assert.doesNotMatch(pageSource, /uploadWechatAvatar/);
  assert.doesNotMatch(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.doesNotMatch(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
});

test("home page gear overview renders stats without concrete gear rows", () => {
  const wxml = read("pages/index/index.wxml");
  const wxss = read("pages/index/index.wxss");
  const ts = read("pages/index/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /gearStatCards/);
  assert.match(pageSource, /getGearStats\(\)/);
  assert.doesNotMatch(pageSource, /getGearOverview/);
  assert.doesNotMatch(pageSource, /recentGears/);
  assert.doesNotMatch(pageSource, /mapHomeGearCard/);
  assert.doesNotMatch(wxml, /recent-list/);
  assert.doesNotMatch(wxml, /recent-item/);
  assert.doesNotMatch(wxml, /brandModelText/);
  assert.doesNotMatch(wxml, /还没有装备记录，先添加第一件户外装备吧。/);
  assert.doesNotMatch(wxss, /\.recent-/);
});

test("home page shows a single trip reminder when available", () => {
  const wxml = read("pages/index/index.wxml");
  const wxss = read("pages/index/index.wxss");
  const ts = read("pages/index/index.ts");
  const pageSource = `${wxml}
${ts}`;
  const highlightWxmlBlock = wxml.slice(
    wxml.indexOf('wx:if="{{tripHighlight}}"'),
    wxml.indexOf('<view class="section-card">'),
  );
  const highlightCardBlock =
    wxss.match(/\.plan-highlight-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightStripBlock =
    wxss.match(/\.plan-highlight-card::before \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightCopyBlock =
    wxss.match(/\.plan-highlight-copy \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightEyebrowBlock =
    wxss.match(/\.plan-highlight-eyebrow \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightTitleBlock =
    wxss.match(/\.plan-highlight-title \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightMetaBlock =
    wxss.match(/\.plan-highlight-meta \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightTextCommonBlock =
    wxss.match(
      /\.plan-highlight-subtitle,[\s\S]*?\.plan-highlight-date \{[\s\S]*?\n\}/,
    )?.[0] ?? "";
  const highlightSubtitleBlock =
    wxss.match(/\.plan-highlight-subtitle \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightDateBlocks = [
    ...wxss.matchAll(/\.plan-highlight-date \{[\s\S]*?\n\}/g),
  ].map((match) => match[0]);
  const highlightDateBlock =
    highlightDateBlocks[highlightDateBlocks.length - 1] ?? "";
  const highlightActionBlock =
    wxss.match(/\.plan-highlight-action \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightChecklistBlock =
    wxss.match(/\.plan-highlight-checklist \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightCheckItemBlock =
    wxss.match(/\.plan-highlight-check-item \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightCheckIconBlock =
    wxss.match(/\.plan-highlight-check-icon \{[\s\S]*?\n\}/)?.[0] ?? "";
  const highlightDarkBlock =
    wxss.match(
      /\.home-page\.theme-dark \.plan-highlight-card \{[\s\S]*?\n\}/,
    )?.[0] ?? "";

  assert.match(wxml, /tripHighlight/);
  assert.match(wxml, /近期行程/);
  assert.match(wxml, /plan-highlight-title">\{\{tripHighlight\.statusText\}\}/);
  assert.match(wxml, /tripHighlight\.subtitle/);
  assert.match(wxml, /tripHighlight\.dateText/);
  assert.match(highlightWxmlBlock, /plan-highlight-checklist/);
  assert.match(highlightWxmlBlock, /出发前检查/);
  assert.match(highlightWxmlBlock, /wx:for="\{\{checklistItems\}\}"/);
  assert.match(highlightWxmlBlock, /plan-highlight-check-title/);
  assert.match(highlightWxmlBlock, /plan-highlight-check-desc/);
  assert.doesNotMatch(wxml, /checklist-card/);
  assert.doesNotMatch(wxml, /class="check-item"/);
  assert.match(wxml, /查看行程/);
  assert.match(wxml, /bindtap="goTripHighlight"/);
  assert.match(pageSource, /getTripHomeHighlight\(formatLocalDate\(\)\)/);
  assert.match(pageSource, /stellartrail_home_trip_refresh/);
  assert.match(pageSource, /"正在进行"/);
  assert.doesNotMatch(pageSource, /正在进行：/);
  assert.match(pageSource, /明天有行程计划/);
  assert.match(pageSource, /天后有行程计划/);
  assert.doesNotMatch(pageSource, /明天有计划/);
  assert.doesNotMatch(pageSource, /天后有计划/);
  assert.match(
    pageSource,
    /\/pages\/trips\/detail\/index\?id=\$\{encodeURIComponent\(id\)\}/,
  );
  assert.match(wxss, /\.plan-highlight-card/);
  assert.match(wxss, /\.plan-highlight-action/);
  assert.match(highlightCardBlock, /min-height:\s*156rpx/);
  assert.match(highlightCardBlock, /position:\s*relative/);
  assert.match(highlightCardBlock, /flex-direction:\s*column/);
  assert.match(highlightCardBlock, /overflow:\s*hidden/);
  assert.match(highlightCardBlock, /linear-gradient/);
  assert.match(highlightCardBlock, /rgba\(204,\s*251,\s*241,\s*0\.72\)/);
  assert.match(highlightStripBlock, /width:\s*8rpx/);
  assert.match(highlightStripBlock, /background:\s*var\(--brand-color\)/);
  assert.match(highlightEyebrowBlock, /display:\s*inline-flex/);
  assert.match(highlightEyebrowBlock, /border-radius:\s*999rpx/);
  assert.match(highlightEyebrowBlock, /background:\s*var\(--brand-soft-bg\)/);
  assert.match(highlightTitleBlock, /color:\s*var\(--brand-color\)/);
  assert.match(highlightCopyBlock, /flex-direction:\s*column/);
  assert.match(wxml, /plan-highlight-meta/);
  assert.match(highlightMetaBlock, /flex-direction:\s*column/);
  assert.match(
    pageSource,
    /stripInlineDateFromTripTitle\(trip\.name, dateText\)/,
  );
  assert.match(pageSource, /const dateText = formatTripDateRange/);
  assert.match(highlightTextCommonBlock, /display:\s*block/);
  assert.match(highlightTextCommonBlock, /width:\s*100%/);
  assert.match(highlightTextCommonBlock, /white-space:\s*nowrap/);
  assert.match(highlightSubtitleBlock, /color:\s*var\(--heading-muted-color\)/);
  assert.match(highlightSubtitleBlock, /font-size:\s*25rpx/);
  assert.match(highlightSubtitleBlock, /font-weight:\s*800/);
  assert.match(highlightDateBlock, /color:\s*var\(--muted-color\)/);
  assert.match(highlightActionBlock, /background:\s*var\(--brand-soft-bg\)/);
  assert.match(highlightActionBlock, /align-self:\s*flex-end/);
  assert.match(highlightActionBlock, /white-space:\s*nowrap/);
  assert.match(highlightChecklistBlock, /border-top:\s*1rpx solid/);
  assert.match(highlightCheckItemBlock, /display:\s*flex/);
  assert.match(
    highlightCheckItemBlock,
    /background:\s*rgba\(255,\s*255,\s*255,\s*0\.62\)/,
  );
  assert.match(highlightCheckIconBlock, /background:\s*var\(--brand-color\)/);
  assert.match(highlightDarkBlock, /linear-gradient/);
  assert.match(pageSource, /stripInlineDateFromTripTitle/);
  assert.doesNotMatch(highlightTextCommonBlock, /-webkit-line-clamp/);
});

test("trip invitation flow exposes manual join and share-card entry points", () => {
  const appConfig = JSON.parse(read("app.json"));
  const gearIndexWxml = read("pages/gears/index.wxml");
  const gearIndexTs = read("pages/gears/index.ts");
  const indexWxml = read("pages/trips/index.wxml");
  const indexTs = read("pages/trips/index.ts");
  const joinWxml = read("pages/trips/join/index.wxml");
  const joinTs = read("pages/trips/join/index.ts");
  const detailWxml = read("pages/trips/detail/index.wxml");
  const detailWxss = read("pages/trips/detail/index.wxss");
  const detailTs = read("pages/trips/detail/index.ts");
  const navigationTs = read("utils/navigation.ts");
  const personalGearActionCardBlock =
    detailWxss.match(/\.personal-gear-action-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const personalGearActionCopyBlock =
    detailWxss.match(/\.personal-gear-action-copy \{[\s\S]*?\n\}/)?.[0] ?? "";
  const personalGearActionsBlock =
    detailWxss.match(/\.personal-gear-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const personalGearActionButtonBlock =
    detailWxss.match(/\.personal-gear-actions button \{[\s\S]*?\n\}/)?.[0] ??
    "";
  const personalGearAddButtonBlock =
    detailWxss.match(
      /\.personal-gear-section-head \.section-add-button \{[\s\S]*?\n\}/,
    )?.[0] ?? "";
  const itineraryDayHeadBlock =
    detailWxss.match(/\.itinerary-day-head \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryHeadActionsBlock =
    detailWxss.match(/\.itinerary-head-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDaySwitcherBlock =
    detailWxss.match(/\.itinerary-day-switcher \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDaySwitchButtonBlock =
    detailWxss.match(/\.itinerary-day-switch-button \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDaySwitchDisabledBlock =
    detailWxss.match(
      /\.itinerary-day-switch-button\.disabled \{[\s\S]*?\n\}/,
    )?.[0] ?? "";
  const itineraryWorkbenchBlock =
    detailWxss.match(/\.itinerary-workbench \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDayTitleRowBlock =
    detailWxss.match(/\.itinerary-day-title-row \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDayPagerBlock =
    detailWxss.match(/\.itinerary-day-pager \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDayInfoActionBlock =
    detailWxss.match(/\.itinerary-day-info-action \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDayActionsBlock =
    detailWxss.match(/\.itinerary-day-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itineraryDayActionsWxmlBlock =
    detailWxml.match(
      /<view class="itinerary-day-actions">[\s\S]*?<\/view>/,
    )?.[0] ?? "";
  const itineraryDayBodyBlock =
    detailWxss.match(/\.itinerary-day-body \{[\s\S]*?\n\}/)?.[0] ?? "";
  const itinerarySwipeHintBlock =
    detailWxss.match(/\.itinerary-swipe-hint \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeSegmentRowBlock =
    detailWxss.match(/\.route-segment-row \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeSegmentMainBlock =
    detailWxss.match(/\.route-segment-main \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeSegmentActionsBlock =
    detailWxss.match(/\.route-segment-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeSegmentChipBlock =
    detailWxss.match(/\.route-segment-chip \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeSegmentEstimateBlock =
    detailWxss.match(/\.route-segment-estimate \{[\s\S]*?\n\}/)?.[0] ?? "";
  const deleteInlineButtonBlock =
    detailWxss.match(/^\.delete-inline-button \{[\s\S]*?\n\}/m)?.[0] ?? "";
  const editInlineButtonBlock =
    detailWxss.match(/^\.edit-inline-button \{[\s\S]*?\n\}/m)?.[0] ?? "";
  const memberCardActionsBlock =
    detailWxss.match(/\.member-card-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const memberCarrySummaryBlock =
    detailWxss.match(/\.member-carry-summary \{[\s\S]*?\n\}/)?.[0] ?? "";
  const memberCarryBreakdownBlock =
    detailWxss.match(/\.member-carry-breakdown \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEditorBlock =
    detailWxss.match(/\.route-editor-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEditorTriggerBlock =
    detailWxss.match(/\.route-editor-trigger \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEditorSectionBlock =
    detailWxss.match(/\.route-editor-section \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEditorSheetBlock =
    detailWxss.match(/\.route-editor-sheet \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEditorScrollBlock =
    detailWxss.match(/\.route-editor-scroll \{[\s\S]*?\n\}/)?.[0] ?? "";
  const orphanRouteBlock =
    detailWxss.match(/\.orphan-route-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEstimateCurrentBlock =
    detailWxss.match(/\.route-estimate-current \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEstimateEyebrowBlock =
    detailWxss.match(/\.route-estimate-eyebrow \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEstimateRuleBlock =
    detailWxss.match(/\.route-estimate-rule \{[\s\S]*?\n\}/)?.[0] ?? "";
  const routeEstimateRuleActiveBlock =
    detailWxss.match(/\.route-estimate-rule\.enabled \{[\s\S]*?\n\}/)?.[0] ??
    "";
  const routeRuleInfoBlock =
    detailWxss.match(/\.route-rule-info \{[\s\S]*?\n\}/)?.[0] ?? "";
  const sharedGearFilterChipBlock =
    detailWxss.match(/\.shared-gear-filter-chip \{[\s\S]*?\n\}/)?.[0] ?? "";
  const sharedGearFilterActiveBlock =
    detailWxss.match(/\.shared-gear-filter-chip\.active \{[\s\S]*?\n\}/)?.[0] ??
    "";
  const sharedGearRowBlock =
    detailWxss.match(/\.shared-gear-row \{[\s\S]*?\n\}/)?.[0] ?? "";
  const sharedGearActionsBlock =
    detailWxss.match(/\.shared-gear-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const medicalSummaryBlock =
    detailWxss.match(/\.medical-summary \{[\s\S]*?\n\}/)?.[0] ?? "";
  const medicalItemRowBlock =
    detailWxss.match(/\.medical-item-row \{[\s\S]*?\n\}/)?.[0] ?? "";
  const medicalItemActionsBlock =
    detailWxss.match(/\.medical-item-actions \{[\s\S]*?\n\}/)?.[0] ?? "";
  const medicalEditorSectionBlock =
    detailWxss.match(/\.medical-editor-section \{[\s\S]*?\n\}/)?.[0] ?? "";
  const medicalStatusMissingBlock =
    detailWxss.match(/\.medical-status-chip\.missing \{[\s\S]*?\n\}/)?.[0] ??
    "";
  const budgetAddButtonBlock =
    detailWxss.match(/\.budget-add-button \{[\s\S]*?\n\}/)?.[0] ?? "";
  const budgetEditorCardBlock =
    detailWxss.match(/\.budget-editor-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const goalAddButtonBlock =
    detailWxss.match(/\.goal-add-button \{[\s\S]*?\n\}/)?.[0] ?? "";
  const goalDeleteButtonBlock =
    detailWxss.match(/\.goal-delete-button \{[\s\S]*?\n\}/)?.[0] ?? "";
  const tripTab = appConfig.tabBar.list.find(
    (item) => item.pagePath === "pages/trips/index",
  );

  assert.equal(appConfig.tabBar.list.length, 5);
  assert.deepEqual(tripTab, {
    pagePath: "pages/trips/index",
    text: "行程",
    iconPath: "assets/tabbar/plan-normal.png",
    selectedIconPath: "assets/tabbar/plan-selected.png",
  });
  assert.ok(
    fs.existsSync(path.join(miniRoot, "assets/tabbar/plan-normal.png")),
  );
  assert.ok(
    fs.existsSync(path.join(miniRoot, "assets/tabbar/plan-selected.png")),
  );
  assert.match(navigationTs, /"\/pages\/trips\/index"/);
  assert.doesNotMatch(gearIndexWxml, /组队计划书|去组队|goTeamPlans/);
  assert.doesNotMatch(gearIndexTs, /goTeamPlans/);
  assert.doesNotMatch(
    gearIndexTs,
    /wx\.switchTab\(\{ url: "\/pages\/trips\/index" \}\)/,
  );
  assert.match(indexWxml, /bindtap="goJoin"/);
  assert.match(indexWxml, /加入多人行程|加入行程/);
  assert.match(indexWxml, /wx:if="{{trip\.canDelete}}"/);
  assert.match(indexTs, /\/pages\/trips\/join\/index/);
  assert.match(indexTs, /owner_user_id === currentUserId/);
  assert.match(joinWxml, /粘贴口令/);
  assert.match(joinWxml, /placeholder="邀请口令或邀请文案"/);
  assert.match(joinTs, /extractTripInvitationToken/);
  assert.match(joinTs, /buildTripJoinPath\(token\)/);
  assert.match(detailWxml, /open-type="share"/);
  assert.match(detailWxml, /分享微信群/);
  assert.match(detailWxml, /复制口令/);
  assert.match(detailWxml, /canDeletePlan/);
  assert.match(detailWxml, /bindtap="confirmDeletePlan"/);
  assert.match(detailTs, /copyInviteToken/);
  assert.match(detailTs, /deleteTrip/);
  assert.match(detailTs, /wx\.switchTab\(\{ url: "\/pages\/trips\/index" \}\)/);
  assert.match(detailTs, /canDeletePlan: canEditAllMembers/);
  assert.match(detailTs, /buildTripInvitationText/);
  assert.match(detailTs, /buildTripJoinPath/);
  assert.match(detailTs, /onShareAppMessage/);
  assert.match(detailTs, /已添加/);
  assert.doesNotMatch(detailTs, /已开启/);
  assert.match(detailWxml, /route-metric-label">距离/);
  assert.match(detailWxml, /route-metric-label">爬升/);
  assert.match(detailWxml, /route-metric-label">下降/);
  assert.match(detailWxml, /route-metric-unit">km/);
  assert.match(detailWxml, /route-metric-unit">m/);
  assert.match(detailWxml, /检查点（可选）/);
  assert.doesNotMatch(detailWxml, /下撤路线（可选）/);
  assert.match(detailWxml, /路况描述（可选）/);
  assert.match(detailWxml, /编辑日信息/);
  assert.doesNotMatch(detailWxml, /天气营地/);
  assert.doesNotMatch(detailWxml, /天气和营地/);
  assert.match(detailWxml, /记录当天的天气、温度和营地条件。/);
  assert.doesNotMatch(detailWxml, /行程日期/);
  assert.doesNotMatch(detailWxml, /dayInfoForm\.dateLabel/);
  assert.doesNotMatch(detailWxml, /dayInfoDateFallback/);
  assert.doesNotMatch(detailWxml, /onDayInfoDateChange/);
  assert.doesNotMatch(detailWxml, /clearDayInfoDate/);
  assert.doesNotMatch(detailWxml, /使用计划日期/);
  assert.match(
    detailWxml,
    /itinerary-day-meta[\s\S]*activeItineraryDay\.dateText[\s\S]*itinerary-day-info-action[\s\S]*catchtap="openDayInfoEditor"/,
  );
  assert.doesNotMatch(itineraryDayActionsWxmlBlock, /openDayInfoEditor/);
  assert.doesNotMatch(detailWxml, /day\.weatherText/);
  assert.doesNotMatch(detailWxml, /day\.campText/);
  assert.doesNotMatch(detailTs, /未填写天气/);
  assert.doesNotMatch(detailTs, /未填写营地/);
  assert.doesNotMatch(detailWxml, /分段分工|添加分工|检查点或路段/);
  assert.match(detailWxml, /安全预案/);
  assert.match(detailWxml, /救援信息/);
  assert.match(detailWxml, /财务预算/);
  assert.match(detailWxml, /目标/);
  assert.match(detailWxml, /scroll-into-view="\{\{activeSectionTabId\}\}"/);
  assert.match(detailWxml, /id="\{\{item\.id\}\}"/);
  assert.match(detailTs, /activeSectionTabId/);
  assert.match(detailTs, /sectionTabDomId/);
  assert.match(detailWxml, /medical-kit-head/);
  assert.match(detailWxml, /新增物品/);
  assert.match(detailWxml, /medicalSummary\.itemCountText/);
  assert.match(detailWxml, /medicalSummary\.requiredText/);
  assert.match(detailWxml, /medicalSummary\.packedText/);
  assert.match(detailWxml, /medicalSummary\.shortageText/);
  assert.match(detailWxml, /medical-item-row/);
  assert.match(detailWxml, /item\.quantityText/);
  assert.match(detailWxml, /item\.statusText/);
  assert.match(detailWxml, /item\.suggestedText/);
  assert.match(detailWxml, /catchtap="openEditMedicalEditor"/);
  assert.match(detailWxml, /catchtap="confirmDeleteMedicalItem"/);
  assert.match(detailWxml, /medicalEditorVisible/);
  assert.match(detailWxml, /medical-editor-sheet/);
  assert.match(detailWxml, /bindinput="onMedicalFormInput"/);
  assert.match(detailWxml, /bindchange="onMedicalScopeChange"/);
  assert.match(detailWxml, /bindtap="saveMedicalItem"/);
  assert.doesNotMatch(detailWxml, /value="\{\{medicalName\}\}"/);
  assert.doesNotMatch(detailWxml, /bindtap="addMedicalItem"/);
  assert.match(detailTs, /updateTripMedicalItem/);
  assert.match(detailTs, /deleteTripMedicalItem/);
  assert.match(detailTs, /buildMedicalItemViews/);
  assert.match(detailTs, /buildMedicalSummary/);
  assert.match(medicalSummaryBlock, /grid-template-columns:\s*repeat\(4/);
  assert.match(medicalItemRowBlock, /position:\s*relative/);
  assert.match(medicalItemRowBlock, /padding-right:\s*176rpx/);
  assert.match(medicalItemActionsBlock, /position:\s*absolute/);
  assert.match(medicalItemActionsBlock, /flex-direction:\s*column/);
  assert.match(medicalEditorSectionBlock, /background:\s*var\(--control-bg\)/);
  assert.match(
    medicalStatusMissingBlock,
    /background:\s*var\(--status-danger-bg\)/,
  );
  assert.match(detailWxml, /食材重量进入背负统计，费用进入财务预算/);
  assert.match(detailWxml, /food-day-switcher/);
  assert.match(detailWxml, /foodDayPagerText/);
  assert.match(detailWxml, /switchPrevFoodDay/);
  assert.match(detailWxml, /switchNextFoodDay/);
  assert.match(detailWxml, /activeFoodDay\.meals/);
  assert.match(detailWxml, /openFoodMealEditor/);
  assert.match(detailWxml, /openFoodItemEditor/);
  assert.match(detailWxml, /openFoodSupplyEditor/);
  assert.match(detailWxml, /保存后会计入负责人食材背负重量/);
  assert.match(detailWxml, /食材费用 元/);
  assert.match(detailWxml, /foodItemForm\.costYuan/);
  assert.match(detailWxml, /foodSupplyForm\.costYuan/);
  assert.match(detailWxml, /食品计划食材/);
  assert.match(detailWxml, /foodBudgetSummary/);
  assert.match(detailWxml, /budget-add-button/);
  assert.match(detailWxml, /bindtap="openBudgetEditor"/);
  assert.match(detailWxml, /budgetEditorVisible/);
  assert.match(detailWxml, /budget-editor-sheet/);
  assert.match(detailWxml, /bindtap="closeBudgetEditor"/);
  assert.match(
    detailWxml,
    /budgetEditorVisible[\s\S]*budgetLinkedSharedGearIndex[\s\S]*budgetCategory[\s\S]*budgetName[\s\S]*budgetQuantity[\s\S]*budgetUnitPriceYuan[\s\S]*budgetSplitMemberCount/,
  );
  assert.doesNotMatch(
    detailWxml,
    /selectedSection === 'budget'[\s\S]*class="editor-card"[\s\S]*添加预算[\s\S]*wx:for="\{\{budgetItems\}\}"/,
  );
  assert.match(detailTs, /buildFoodBudgetSummary/);
  assert.match(detailTs, /total_price_cents/);
  assert.match(detailTs, /openBudgetEditor/);
  assert.match(detailTs, /closeBudgetEditor/);
  assert.match(detailTs, /budgetEditorVisible:\s*false/);
  assert.match(budgetAddButtonBlock, /background:\s*var\(--brand-soft-bg\)/);
  assert.match(budgetEditorCardBlock, /background:\s*var\(--control-bg\)/);
  assert.doesNotMatch(detailWxml, /同步到公共装备/);
  assert.doesNotMatch(detailTs, /同步从公共装备/);
  assert.match(detailWxml, /跨餐公共食材/);
  assert.match(detailWxml, /confirmDeleteFoodItem/);
  assert.match(detailWxml, /confirmDeleteFoodSupply/);
  assert.match(detailWxml, /原关联装备已删除/);
  assert.doesNotMatch(detailWxml, /教学点|授课内容|授课人/);
  assert.match(detailWxml, /设置/);
  assert.match(detailWxml, /新增一天/);
  assert.doesNotMatch(detailWxml, /添加天/);
  assert.match(detailWxml, /估算规则/);
  assert.match(detailWxml, /routeEstimateSummaryText/);
  assert.match(detailWxml, /routeEstimateDetailText/);
  assert.match(detailWxml, /routeEstimateSheetVisible/);
  assert.match(detailWxml, /route-estimate-current/);
  assert.doesNotMatch(detailWxml, /class="route-estimate-card"/);
  assert.match(detailTs, /基础 Naismith/);
  assert.match(detailWxml, /routeEstimateRules/);
  assert.match(detailWxml, /catchtap="showRouteEstimateRuleInfo"/);
  assert.match(detailWxml, /bindtap="toggleRouteEstimateRule"/);
  assert.match(detailWxml, /routeUseHighAltitudeAdjustmentDraft/);
  assert.match(detailWxml, /routeStartAltitudeDraft/);
  assert.match(detailWxml, /saveRouteEstimateSettings/);
  assert.match(detailWxml, /segment\.altitudeText/);
  assert.match(detailWxml, /segment\.metricChips/);
  assert.match(detailWxml, /route-segment-chip/);
  assert.match(detailWxml, /route-segment-estimate/);
  assert.match(detailWxml, /routeEditorVisible/);
  assert.match(detailWxml, /section-sheet route-editor-sheet/);
  assert.match(detailWxml, /route-editor-trigger/);
  assert.match(detailWxml, /\+ 添加路段/);
  assert.match(detailWxml, /bindtap="openRouteEditor"/);
  assert.match(detailWxml, /catchtap="openEditRouteEditor"/);
  assert.match(detailWxml, /bindtap="closeRouteEditor"/);
  assert.match(detailWxml, /编辑路段/);
  assert.match(detailWxml, /保存路段/);
  assert.match(detailWxml, /bindtap="saveRouteSegment"/);
  assert.match(detailWxml, /基础信息/);
  assert.match(detailWxml, /路程数据/);
  assert.match(detailWxml, /路况补充/);
  assert.match(detailWxml, /例如：下撤路线 \/ 失温 \/ 暴雨/);
  assert.match(detailWxml, /例如：提前确认下撤口、集合点和备用交通/);
  assert.match(detailWxml, /例如：从 XX 垭口下撤到 XX 村，联系 XX 车辆/);
  assert.doesNotMatch(detailWxml, /补充信息/);
  assert.doesNotMatch(detailWxml, /routeBailoutRoute/);
  assert.match(detailWxml, /添加到当前行程日，保存后会自动收起。/);
  assert.doesNotMatch(detailWxml, /领队：/);
  assert.doesNotMatch(detailWxml, /添加路段估算/);
  assert.doesNotMatch(detailWxml, /route-editor-section-muted/);
  assert.match(detailTs, /updateTrip/);
  assert.match(detailTs, /date_label/);
  assert.match(detailTs, /deriveItineraryDayDate/);
  assert.doesNotMatch(detailTs, /dayInfoDateDirty/);
  assert.doesNotMatch(detailTs, /onDayInfoDateChange/);
  assert.doesNotMatch(detailTs, /clearDayInfoDate/);
  assert.doesNotMatch(detailTs, /fields\.date_label/);
  assert.match(detailTs, /metricChips/);
  assert.match(detailTs, /预估耗时/);
  assert.doesNotMatch(detailTs, /formula_estimate_minutes\} 分钟，最终/);
  assert.match(detailTs, /buildRouteEstimateRuleViews/);
  assert.match(detailTs, /formatRouteEstimateSummary/);
  assert.match(detailTs, /formatRouteAltitudeText/);
  assert.doesNotMatch(detailTs, /createTripSegmentAssignment/);
  assert.doesNotMatch(detailTs, /deleteTripSegmentAssignment/);
  assert.match(detailTs, /createTripFoodSupply/);
  assert.match(detailTs, /createTripFoodItem/);
  assert.match(detailTs, /updateTripFoodItem/);
  assert.match(detailTs, /updateTripFoodSupply/);
  assert.match(detailTs, /deleteTripFoodItem/);
  assert.doesNotMatch(detailTs, /sourceFoodItemId/);
  assert.doesNotMatch(detailTs, /sourceFoodSupplyId/);
  assert.doesNotMatch(detailTs, /isFoodManaged/);
  assert.match(detailTs, /createTripSafetyRisk/);
  assert.match(detailTs, /createTripRescueContact/);
  assert.match(detailTs, /createTripBudgetItem/);
  assert.match(detailTs, /createTripGoalItem/);
  assert.match(detailTs, /scope:\s*"team"/);
  assert.match(detailTs, /member_id:\s*null/);
  assert.doesNotMatch(detailTs, /goalScope/);
  assert.match(detailTs, /buildGoalViews/);
  assert.match(detailTs, /goalEditorVisible/);
  assert.match(detailWxml, /添加目标/);
  assert.match(detailWxml, /item\.titleText/);
  assert.match(detailWxml, /goal-add-button/);
  assert.match(detailWxml, /goal-add-button"[\s\S]*>\s*\+\s*<\/button>/);
  assert.match(detailWxml, /bindtap="openGoalEditor"/);
  assert.match(detailWxml, /goal-delete-button/);
  assert.match(detailWxml, /goal-delete-button[\s\S]*>\s*×\s*<\/button>/);
  assert.match(detailWxml, /goal-editor-sheet/);
  assert.match(detailWxml, /bindtap="closeGoalEditor"/);
  assert.match(goalAddButtonBlock, /width:\s*68rpx/);
  assert.match(goalAddButtonBlock, /margin-right:\s*auto !important/);
  assert.match(goalDeleteButtonBlock, /position:\s*absolute/);
  assert.match(goalDeleteButtonBlock, /top:\s*18rpx/);
  assert.match(goalDeleteButtonBlock, /right:\s*18rpx/);
  assert.match(goalDeleteButtonBlock, /width:\s*52rpx/);
  assert.doesNotMatch(detailWxml, /只记录本次计划的目标/);
  assert.doesNotMatch(detailWxml, /team 或 member/);
  assert.doesNotMatch(detailWxml, /个人目标/);
  assert.match(detailTs, /ROUTE_ESTIMATE_RULE_DETAIL/);
  assert.match(detailWxml, /activeItineraryDay/);
  assert.match(detailWxml, /itinerary-day-switcher/);
  assert.match(detailWxml, /itineraryDayPagerText/);
  assert.match(detailWxml, /catchtap="switchPrevItineraryDay"/);
  assert.match(detailWxml, /catchtap="switchNextItineraryDay"/);
  assert.match(detailWxml, /disabled="\{\{!canSwitchPrevDay\}\}"/);
  assert.match(detailWxml, /disabled="\{\{!canSwitchNextDay\}\}"/);
  assert.match(detailWxml, /bindtouchstart="onItineraryTouchStart"/);
  assert.match(detailWxml, /bindtouchend="onItineraryTouchEnd"/);
  assert.match(detailWxml, /itinerary-route-list/);
  assert.match(detailWxml, /点箭头或左右滑动切换行程日/);
  assert.match(
    detailWxml,
    /itinerary-day-title"[\s\S]*\{\{activeItineraryDay\.title\}\}/,
  );
  assert.doesNotMatch(detailWxml, /wx:for="{{itineraryDays}}"/);
  assert.doesNotMatch(detailWxml, /展开路段|收起路段/);
  assert.doesNotMatch(detailWxml, /D\{\{day\.dayIndex\}\}/);
  assert.match(detailTs, /formatItineraryDayTitle/);
  assert.match(detailTs, /activeItineraryDayId/);
  assert.match(detailTs, /activeItineraryDayIndex/);
  assert.match(detailTs, /canSwitchPrevDay/);
  assert.match(detailTs, /canSwitchNextDay/);
  assert.match(detailTs, /switchActiveItineraryDay/);
  assert.match(detailTs, /switchPrevItineraryDay/);
  assert.match(detailTs, /switchNextItineraryDay/);
  assert.match(detailTs, /itinerarySwipeStartPoint/);
  assert.doesNotMatch(detailTs, /itinerarySwipeStartX|itinerarySwipeStartY/);
  assert.doesNotMatch(detailTs, /toggleItineraryDay/);
  assert.match(detailTs, /openRouteEditor/);
  assert.match(detailTs, /openEditRouteEditor/);
  assert.match(detailTs, /closeRouteEditor/);
  assert.match(detailTs, /saveRouteSegment/);
  assert.match(detailTs, /updateRouteSegment/);
  assert.match(detailTs, /updateTripRouteSegment/);
  assert.match(detailTs, /routeEditorSegmentId:\s*""/);
  assert.match(detailTs, /routeEditorVisible:\s*false/);
  assert.match(detailTs, /leader_member_id:\s*null/);
  assert.match(detailTs, /bailout_route:\s*null/);
  assert.doesNotMatch(detailTs, /routeBailoutRoute/);
  assert.doesNotMatch(detailTs, /下撤：/);
  assert.doesNotMatch(detailTs, /routeLeaderIndex/);
  assert.doesNotMatch(detailTs, /onRouteLeaderChange/);
  assert.doesNotMatch(detailTs, /已指定领队/);
  assert.match(detailWxml, /catchtap="confirmDeleteItineraryDay"/);
  assert.match(detailWxml, /catchtap="confirmDeleteRouteSegment"/);
  assert.match(detailWxml, /class="route-segment-actions"/);
  assert.match(detailWxml, /class="edit-inline-button"/);
  assert.match(detailWxml, /class="route-segment-row"/);
  assert.match(detailWxml, /class="orphan-route-card"/);
  assert.match(detailTs, /createTripItineraryTimeSlot/);
  assert.match(detailTs, /deleteTripItineraryDay/);
  assert.match(detailTs, /deleteTripRouteSegment/);
  assert.match(detailTs, /orphanRouteSegments/);
  assert.match(itineraryDayHeadBlock, /display:\s*flex/);
  assert.match(itineraryDayHeadBlock, /justify-content:\s*space-between/);
  assert.match(itineraryHeadActionsBlock, /display:\s*flex/);
  assert.match(itineraryDaySwitcherBlock, /display:\s*flex/);
  assert.match(itineraryDaySwitchButtonBlock, /flex:\s*0 0 64rpx/);
  assert.match(itineraryDaySwitchDisabledBlock, /opacity:\s*0\.56/);
  assert.match(itineraryWorkbenchBlock, /margin-top:\s*18rpx/);
  assert.match(itineraryDayTitleRowBlock, /display:\s*flex/);
  assert.match(itineraryDayPagerBlock, /border-radius:\s*999rpx/);
  assert.match(itineraryDayPagerBlock, /text-align:\s*center/);
  assert.match(
    itineraryDayInfoActionBlock,
    /background:\s*var\(--brand-soft-bg\)/,
  );
  assert.match(itineraryDayInfoActionBlock, /white-space:\s*nowrap/);
  assert.match(itineraryDayActionsBlock, /flex-direction:\s*column/);
  assert.doesNotMatch(itineraryDayActionsBlock, /small-action/);
  assert.match(itinerarySwipeHintBlock, /text-align:\s*center/);
  assert.doesNotMatch(detailWxss, /\.itinerary-day-toggle/);
  assert.match(deleteInlineButtonBlock, /min-width:\s*88rpx/);
  assert.match(deleteInlineButtonBlock, /margin-top:\s*0 !important/);
  assert.match(
    deleteInlineButtonBlock,
    /background:\s*var\(--status-danger-bg\)/,
  );
  assert.doesNotMatch(deleteInlineButtonBlock, /width:\s*100%/);
  assert.match(
    itineraryDayBodyBlock,
    /border-top:\s*1rpx solid var\(--soft-border-color\)/,
  );
  assert.match(routeSegmentRowBlock, /display:\s*flex/);
  assert.match(routeSegmentRowBlock, /background:\s*var\(--control-bg\)/);
  assert.match(routeSegmentMainBlock, /min-width:\s*0/);
  assert.match(routeSegmentActionsBlock, /flex-direction:\s*column/);
  assert.match(editInlineButtonBlock, /background:\s*var\(--brand-soft-bg\)/);
  assert.match(editInlineButtonBlock, /white-space:\s*nowrap/);
  assert.match(routeSegmentChipBlock, /border-radius:\s*999rpx/);
  assert.match(routeSegmentChipBlock, /white-space:\s*nowrap/);
  assert.match(routeSegmentEstimateBlock, /color:\s*var\(--muted-color\)/);
  assert.match(routeEditorBlock, /margin-top:\s*18rpx/);
  assert.match(routeEditorTriggerBlock, /width:\s*100%/);
  assert.match(routeEditorTriggerBlock, /background:\s*var\(--brand-soft-bg\)/);
  assert.match(routeEditorSectionBlock, /background:\s*var\(--surface-color\)/);
  assert.doesNotMatch(detailWxss, /\.route-editor-section-muted/);
  assert.match(routeEditorSheetBlock, /max-height:\s*88vh/);
  assert.match(routeEditorScrollBlock, /max-height:\s*72vh/);
  assert.match(orphanRouteBlock, /border:\s*1rpx dashed var\(--border-color\)/);
  assert.match(routeEstimateCurrentBlock, /background:\s*var\(--control-bg\)/);
  assert.doesNotMatch(detailWxss, /\.route-estimate-card/);
  assert.match(routeEstimateEyebrowBlock, /font-size:\s*22rpx/);
  assert.match(routeEstimateRuleBlock, /display:\s*flex/);
  assert.match(routeEstimateRuleBlock, /background:\s*var\(--control-bg\)/);
  assert.match(
    routeEstimateRuleActiveBlock,
    /background:\s*var\(--brand-soft-bg\)/,
  );
  assert.match(routeRuleInfoBlock, /border-radius:\s*999rpx/);
  assert.match(detailWxml, /personal-gear-action-copy/);
  assert.match(detailWxml, /personal-gear-section-head/);
  assert.match(detailWxml, /personal-gear-add-button/);
  assert.match(detailWxml, /bindtap="openPersonalGearActionSheet"/);
  assert.match(detailWxml, /personalGearActionSheetVisible/);
  assert.match(detailWxml, /personal-gear-action-sheet/);
  assert.match(detailWxml, /bindtap="closePersonalGearActionSheet"/);
  assert.doesNotMatch(detailWxml, /carry-summary-card/);
  assert.match(detailWxml, /member-carry-summary/);
  assert.match(detailWxml, /公共 \{\{item\.sharedWeightText\}\}/);
  assert.match(detailWxml, /个人 \{\{item\.personalWeightText\}\}/);
  assert.match(detailWxml, /wx:if="\{\{item\.showFoodWeight\}\}"/);
  assert.match(detailWxml, /食材 \{\{item\.foodWeightText\}\}/);
  assert.match(detailWxml, /\{\{item\.totalWeightText\}\}/);
  assert.match(detailWxml, /wx:if="\{\{item\.canDelete\}\}"/);
  assert.match(detailWxml, /class="delete-inline-button member-delete-button"/);
  assert.match(detailWxml, /bindtap="confirmRemoveMember"/);
  assert.match(detailTs, /removeTripMember/);
  assert.match(detailTs, /已移除成员/);
  assert.match(detailTs, /会从本计划移除该成员/);
  assert.match(detailTs, /SOLO_DEFAULT_SECTIONS[\s\S]*"personal_gear"/);
  assert.match(
    detailTs,
    /SOLO_OPTIONAL_SECTIONS[\s\S]*section !== "shared_gear"/,
  );
  assert.match(detailTs, /sanitizeEnabledSectionsForTripType/);
  assert.match(detailTs, /fallbackSectionForTripType/);
  assert.match(detailTs, /buildMyGearViewForTripType/);
  assert.match(detailTs, /item\.source === "personal"/);
  assert.match(detailWxml, /导入装备清单/);
  assert.match(detailWxml, /导入我的装备/);
  assert.match(detailWxml, /bindtap="openImportSheet"/);
  assert.match(detailWxml, /bindtap="openMyGearImportSheet"/);
  assert.match(detailWxml, /bindtap="openSharedGearForm"/);
  assert.match(detailWxml, /新增需求项/);
  assert.match(detailWxml, /sharedSlotSheetVisible/);
  assert.match(detailWxml, /sharedSlotFormVisible/);
  assert.match(detailWxml, /bindtap="openSharedSlotSheet"/);
  assert.match(detailWxml, /bindtap="openSharedSlotImportSheet"/);
  assert.match(detailWxml, /手动填写具体装备/);
  assert.match(detailWxml, /填写具体装备/);
  assert.match(detailWxml, /保存具体装备/);
  assert.match(detailWxml, /需求分类/);
  assert.match(detailWxml, /保存后按数量和单件重量计入公共装备统计/);
  assert.doesNotMatch(detailWxml, /直接新增具体装备/);
  assert.doesNotMatch(detailWxml, /需求位分类/);
  assert.doesNotMatch(detailWxml, /绑定具体装备/);
  assert.match(detailWxml, /class="field-label">数量<\/view>/);
  assert.doesNotMatch(
    detailWxml,
    /planGearFormTarget === 'shared' \? '总需求量'/,
  );
  assert.match(detailWxml, /总需求量[\s\S]*sharedSlotPlannedQuantity/);
  assert.match(detailWxml, /总需求量[\s\S]*sharedSlotForm\.plannedQuantity/);
  assert.match(detailWxml, /catchtap="confirmMovePersonalGearToShared"/);
  assert.match(detailWxml, /sharedGearCategoryOptions/);
  assert.match(detailWxml, /bindtap="switchSharedGearCategory"/);
  assert.match(detailWxml, /shared-gear-filter-chip/);
  assert.match(detailWxml, /class="shared-gear-actions"/);
  assert.match(sharedGearRowBlock, /position:\s*relative/);
  assert.match(sharedGearRowBlock, /padding-right:\s*194rpx/);
  assert.match(sharedGearActionsBlock, /position:\s*absolute/);
  assert.match(sharedGearActionsBlock, /right:\s*22rpx/);
  assert.match(sharedGearActionsBlock, /width:\s*156rpx/);
  assert.match(detailWxml, /catchtap="confirmDeleteSharedGear"/);
  assert.doesNotMatch(detailWxml, /wx:if="\{\{!item\.isPlaceholder\}\}"/);
  assert.match(detailTs, /deleteTripSharedGearDemand/);
  assert.match(detailTs, /已删除需求项/);
  assert.match(detailWxml, /负责人（背负\/保管）/);
  assert.match(
    detailWxml,
    /归属：\{\{item\.sourceText\}\} · 来源：\{\{item\.createdByText\}\}/,
  );
  assert.match(personalGearActionCardBlock, /display:\s*flex/);
  assert.match(personalGearActionCardBlock, /align-items:\s*flex-start/);
  assert.match(personalGearActionCopyBlock, /flex:\s*1 1 auto/);
  assert.match(personalGearActionCopyBlock, /min-width:\s*0/);
  assert.match(personalGearActionsBlock, /display:\s*flex/);
  assert.match(personalGearActionsBlock, /flex-direction:\s*column/);
  assert.doesNotMatch(personalGearActionsBlock, /max-width:\s*380rpx/);
  assert.match(personalGearActionButtonBlock, /min-width:\s*174rpx/);
  assert.match(personalGearActionButtonBlock, /white-space:\s*nowrap/);
  assert.match(detailWxml, /importSheetVisible/);
  assert.match(detailWxml, /myGearImportSheetVisible/);
  assert.match(detailWxml, /planGearFormVisible/);
  assert.match(detailWxml, /新建装备清单/);
  assert.match(detailWxml, /手动添加计划装备/);
  assert.match(detailTs, /shared_gear_demand_templates/);
  assert.doesNotMatch(detailTs, /DEFAULT_SHARED_GEAR_SLOT_TEMPLATES/);
  assert.doesNotMatch(detailWxml, /打包清单 ID/);
  assert.doesNotMatch(detailWxml, /bindtap="openNewGearSheet"/);
  assert.doesNotMatch(detailWxml, /newGearSheetVisible/);
  assert.doesNotMatch(detailTs, /this\.data\.packingListId/);
  assert.match(detailTs, /listGearPackingLists/);
  assert.match(detailTs, /listGears/);
  assert.match(detailTs, /personalGearActionSheetVisible/);
  assert.match(detailTs, /openPersonalGearActionSheet/);
  assert.match(detailTs, /closePersonalGearActionSheet/);
  assert.match(detailTs, /createTripPersonalGearItem/);
  assert.match(detailTs, /source_gear_id: gear\.id/);
  assert.match(detailTs, /createTripSharedGearDemand/);
  assert.match(detailTs, /deleteTripPersonalGearItem/);
  assert.match(detailTs, /deleteTripSharedGearDemand/);
  assert.match(detailTs, /updateTripSharedGearDemand/);
  assert.match(detailTs, /sharedGearCategoryFilter/);
  assert.match(detailTs, /buildSharedGearCategoryOptions/);
  assert.match(detailTs, /filterSharedGearRows/);
  assert.match(detailTs, /buildMemberCarrySummary/);
  assert.match(
    detailTs,
    /buildMemberCarrySummary\(\s*detail,\s*member\.id,\s*enabled\.has\("food_plan"\)\s*,?\s*\)/,
  );
  assert.match(
    detailTs,
    /const foodWeightG = includeFoodWeight[\s\S]*\? sumFoodWeightForMember\(detail, memberId\)[\s\S]*: 0/,
  );
  assert.match(detailTs, /sumFoodWeightForMember/);
  assert.match(detailTs, /importTripPackingList/);
  assert.match(detailWxml, /编辑成员信息/);
  assert.match(detailWxml, /一键导入我的资料/);
  assert.match(detailWxml, /bindtap="openMemberEditor"/);
  assert.match(detailWxml, /bindtap="saveMemberProfile"/);
  assert.match(detailWxml, /bindtap="toggleMemberRoleOption"/);
  assert.match(detailWxml, /role-option-row/);
  assert.match(detailTs, /getOutdoorProfile/);
  assert.match(detailTs, /outdoorProfileToMemberFields/);
  assert.match(detailTs, /calculateAge/);
  assert.match(detailTs, /fields\.age = age/);
  assert.match(detailTs, /MEMBER_ROLE_OPTIONS/);
  assert.match(detailTs, /toggleRoleLabel/);
  assert.match(detailWxml, /年龄/);
  assert.match(detailWxml, /memberEditorForm\.age/);
  assert.match(detailWxml, /data-field="age"/);
  assert.doesNotMatch(detailWxml, /出生日期/);
  assert.doesNotMatch(detailWxml, /memberEditorForm\.birthDate/);
  assert.doesNotMatch(detailWxml, /bindchange="onMemberBirthDateChange"/);
  assert.match(personalGearAddButtonBlock, /padding:\s*0/);
  assert.match(personalGearAddButtonBlock, /line-height:\s*60rpx/);
  assert.match(memberCardActionsBlock, /flex-wrap:\s*wrap/);
  assert.match(memberCarrySummaryBlock, /flex-wrap:\s*wrap/);
  assert.match(memberCarryBreakdownBlock, /flex:\s*1 1 190rpx/);
  assert.match(sharedGearFilterChipBlock, /white-space:\s*nowrap/);
  assert.match(sharedGearFilterChipBlock, /background:\s*var\(--control-bg\)/);
  assert.match(
    sharedGearFilterActiveBlock,
    /background:\s*var\(--brand-color\)/,
  );
});

test("home page hides featured knots until the user accepts the disclaimer", () => {
  const wxml = read("pages/index/index.wxml");
  const wxss = read("pages/index/index.wxss");
  const ts = read("pages/index/index.ts");

  assert.match(ts, /getKnotDisclaimer/);
  assert.match(ts, /hasLocalKnotDisclaimerAcceptance/);
  assert.match(ts, /if \(!hasAccessToken\(\)\) \{/);
  assert.match(ts, /if \(!disclaimer\.accepted\) \{/);
  assert.match(ts, /this\.hideFeaturedSkills\(\)/);
  assert.match(ts, /await listKnots\(\{ offset: 0, limit: 3 \}\)/);
  assert.match(wxml, /wx:elif="{{featuredSkills\.length}}"/);
  assert.match(wxml, /item\.aliasText/);
  assert.match(wxss, /\.skill-alias/);
});

test("skills page gates knot list behind disclaimer acceptance", () => {
  const wxml = read("pages/skills/index.wxml");
  const wxss = read("pages/skills/index.wxss");
  const ts = read("pages/skills/index.ts");

  assert.match(ts, /getKnotDisclaimer/);
  assert.match(ts, /hasLocalKnotDisclaimerAcceptance/);
  assert.match(ts, /submitKnotDisclaimerAcceptance/);
  assert.match(ts, /enterKnotsList/);
  assert.match(ts, /rejectKnotDisclaimer/);
  assert.match(ts, /当前离线，请联网确认绳结免责声明后再查看/);
  assert.match(wxml, /knotDisclaimerVisible/);
  assert.match(wxml, /我已阅读并同意/);
  assert.match(wxml, /拒绝并退出/);
  assert.match(wxml, /item\.aliasText/);
  assert.match(wxml, /class="skill-alias"/);
  assert.match(
    wxml,
    /class="knot-search-input"[\s\S]*bindinput="onSearchInput"/,
  );
  assert.match(
    wxml,
    /class="knot-search-input"[\s\S]*bindconfirm="submitKnotSearch"/,
  );
  assert.doesNotMatch(
    wxml,
    /!loading && \(allKnots\.length \|\| hasActiveFilters \|\| categoryFilters\.length\)/,
  );
  assert.match(wxss, /\.disclaimer-mask/);
  assert.match(wxss, /\.disclaimer-content/);
  assert.match(wxss, /\.skill-alias/);
  assert.match(ts, /onSearchInput\(event: any\) \{[\s\S]*this\.applyFilters/);
  assert.match(ts, /submitKnotSearch\(\) \{[\s\S]*this\.loadKnots/);
  assert.match(ts, /clearKnotSearchTimer/);
  assert.match(read("utils/skill-utils.ts"), /aliasText/);
});

test("knot detail page shows safety notice before source description", () => {
  const wxml = read("pages/skills/detail/index.wxml");
  const wxss = read("pages/skills/detail/index.wxss");
  const ts = read("pages/skills/detail/index.ts");

  assert.ok(wxml.indexOf("安全提示") < wxml.indexOf("资料说明"));
  assert.match(wxml, /detailAliasText/);
  assert.match(wxml, /class="detail-alias"/);
  assert.match(wxss, /\.detail-alias/);
  assert.match(ts, /formatKnotAliasText\(detail\.aliases\)/);
  assert.match(wxml, /仅供绳结知识学习和非承重练习/);
  assert.match(
    wxml,
    /不得直接用于承载人体、攀登、救援、吊装、高空作业、航海安全/,
  );
  assert.match(wxml, /专业训练/);
  assert.match(wxml, /现场条件检查复核/);
  assert.doesNotMatch(wxml, /用途说明/);
  assert.match(wxss, /\.safety-card/);
  assert.match(wxss, /var\(--status-warning-bg\)/);
});

test("knot risk copy audit flags critical high-risk wording", () => {
  const script = path.resolve(
    appRoot,
    "../..",
    "scripts/audit-knot-risk-copy.mjs",
  );
  const manifest = {
    locale: "zh-CN",
    item_count: 1,
    media_count: 0,
    items: [
      {
        id: "firemans-chair-knot",
        title: "消防员椅结",
        summary: "具有两个可调锁定绳环的临时救援吊带。",
        description: "该绳结可用作救援安全带并支撑人员。",
        steps: [],
        categories: [],
        types: [],
      },
    ],
  };
  const result = spawnSync(
    process.execPath,
    [script, "--stdin", "--fail-on-critical"],
    {
      input: JSON.stringify(manifest),
      encoding: "utf8",
    },
  );
  assert.equal(result.status, 2);
  const summary = JSON.parse(result.stdout);
  assert.equal(summary.critical_count, 1);
  assert.equal(summary.text_risk_item_counts.rescue, 1);
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
  assert.match(wxml, /查看账号资料与户外资料/);
  assert.match(wxml, /account-logout-button/);
  assert.match(wxml, /登录 \/ 注册/);
  assert.doesNotMatch(wxml, /account-button danger/);
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
  assert.match(pageSource, /退出后会清除本机登录状态/);
  assert.doesNotMatch(pageSource, /navigateToGuestFallback/);
  assert.match(wxml, /绳结离线缓存/);
  assert.match(wxml, /意见反馈/);
  assert.doesNotMatch(wxml, /bindtap="openOutdoorProfile"/);
  assert.doesNotMatch(pageSource, /\/pages\/profile\/outdoor\/index/);
  assert.match(wxml, /产品路线图/);
  assert.match(wxml, /bindtap="openRoadmap"/);
  assert.match(pageSource, /openRoadmap/);
  assert.match(pageSource, /\/pages\/profile\/roadmap\/index/);
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
  assert.match(wxml, /scroll-y/);
  assert.match(wxml, /releaseNoteSections/);
  assert.match(wxml, /version-note-section-title/);
  assert.match(wxml, /version-note-number/);
  assert.match(pageSource, /buildReleaseNoteSectionViews/);
  assert.match(wxml, /最新/);
  assert.match(wxml, /bindtap="retryClientVersions"/);
  assert.match(wxml, /bindtap="closeVersionInfoModal"/);
  assert.doesNotMatch(wxml, /客户端版本更新由后台发布/);
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

test("profile roadmap page is registered and supports voting subscriptions", () => {
  const config = JSON.parse(read("app.json"));
  const wxml = read("pages/profile/roadmap/index.wxml");
  const wxss = read("pages/profile/roadmap/index.wxss");
  const ts = read("pages/profile/roadmap/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.ok(config.pages.includes("pages/profile/roadmap/index"));
  assert.match(wxml, /产品路线图/);
  assert.match(wxml, /statusFilterLabels/);
  assert.match(wxml, /bindchange="onStatusFilterChange"/);
  assert.match(wxml, /bindtap="toggleVote"/);
  assert.match(wxml, /bindtap="toggleSubscription"/);
  assert.match(wxml, /loginPrompt\.visible/);
  assert.match(pageSource, /listRoadmap/);
  assert.match(pageSource, /listMyRoadmap/);
  assert.match(pageSource, /loadRoadmapForGuestAccess/);
  assert.match(pageSource, /const publicResponse = await listRoadmap\(request\)/);
  assert.match(pageSource, /return await listMyRoadmap\(request\)/);
  assert.match(pageSource, /return publicResponse/);
  assert.match(pageSource, /voteRoadmapItem/);
  assert.match(pageSource, /unvoteRoadmapItem/);
  assert.match(pageSource, /subscribeRoadmapItem/);
  assert.match(pageSource, /unsubscribeRoadmapItem/);
  assert.match(pageSource, /showOfflineWriteBlockedToast/);
  assert.match(
    pageSource,
    /smart-packing-template|智能打包清单模板|RoadmapItem/,
  );
  assert.match(wxss, /\.roadmap-card/);
  assert.match(wxss, /\.roadmap-action-button/);
});

test("profile settings page owns account identity and password actions", () => {
  const appConfig = JSON.parse(read("app.json"));
  const wxml = read("pages/profile/settings/index.wxml");
  const ts = read("pages/profile/settings/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.ok(appConfig.pages.includes("pages/profile/settings/index"));
  assert.match(wxml, /settings-hero-title/);
  assert.match(wxml, /accountProfile\.displayName/);
  assert.match(wxml, /accountProfile\.emailText/);
  assert.doesNotMatch(wxml, /头像可回到“我的”页点击头像修改/);
  assert.match(wxml, /修改名称/);
  assert.match(wxml, /修改邮箱/);
  assert.match(wxml, /绑定邮箱/);
  assert.match(wxml, /修改密码/);
  assert.match(wxml, /户外资料/);
  assert.match(wxml, /户外经历/);
  assert.doesNotMatch(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /type="nickname"/);
  assert.match(wxml, /点这里后，在下方选择“用微信昵称”/);
  assert.match(wxml, /bindsubmit="saveWechatNickname"/);
  assert.match(wxml, /bindnicknamereview="onWechatNicknameReview"/);
  assert.match(wxml, /bindtap="openEmailBindingModal"/);
  assert.match(wxml, /bindtap="sendEmailBindingCode"/);
  assert.match(wxml, /bindtap="submitEmailBinding"/);
  assert.match(wxml, /bindtap="openPasswordModal"/);
  assert.match(wxml, /bindtap="openOutdoorProfile"/);
  assert.match(wxml, /bindtap="openOutdoorExperiences"/);
  assert.match(wxml, /bindtap="sendPasswordCode"/);
  assert.match(wxml, /bindtap="submitPasswordChange"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(pageSource, /updateWechatNickname\(nickname\)/);
  assert.doesNotMatch(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.match(pageSource, /sendBindEmailCode/);
  assert.match(pageSource, /bindEmailToCurrentAccount/);
  assert.match(pageSource, /sendPasswordResetCode/);
  assert.match(pageSource, /resetPassword/);
  assert.match(pageSource, /openEmailBindingModal/);
  assert.match(pageSource, /\/pages\/profile\/outdoor\/index/);
  assert.match(pageSource, /\/pages\/profile\/outdoor-experiences\/index/);
  assert.match(pageSource, /先绑定邮箱/);
  assert.match(pageSource, /buildAccountProfile/);
  assert.match(pageSource, /WechatNicknameSubmitEvent/);
});

test("profile outdoor page edits reusable trip member defaults", () => {
  const appConfig = JSON.parse(read("app.json"));
  const settingsWxml = read("pages/profile/settings/index.wxml");
  const wxml = read("pages/profile/outdoor/index.wxml");
  const ts = read("pages/profile/outdoor/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.ok(appConfig.pages.includes("pages/profile/outdoor/index"));
  assert.match(settingsWxml, /户外资料/);
  assert.match(wxml, /户外 ID/);
  assert.match(wxml, /身高 cm/);
  assert.match(wxml, /出生日期/);
  assert.match(wxml, /bindchange="onBirthDateChange"/);
  assert.match(wxml, /bindtap="clearBirthDate"/);
  assert.match(wxml, /血型/);
  assert.doesNotMatch(wxml, /年龄/);
  assert.doesNotMatch(wxml, /birthAgeText/);
  assert.match(wxml, /紧急联系人电话/);
  assert.match(wxml, /紧急联系人关系/);
  assert.match(wxml, /既往病/);
  assert.match(wxml, /过敏史/);
  assert.match(wxml, /过敏 \/ 伤病处理方法/);
  assert.match(wxml, /饮食习惯/);
  assert.match(wxml, /保险单号/);
  assert.match(wxml, /保险公司电话/);
  assert.doesNotMatch(wxml, /户外经验补充/);
  assert.doesNotMatch(wxml, /常走路线、露营经验、装备偏好或注意事项/);
  assert.match(pageSource, /getOutdoorProfile/);
  assert.match(pageSource, /updateOutdoorProfile/);
  assert.match(pageSource, /birth_date/);
  assert.match(pageSource, /formatLocalDate/);
  assert.doesNotMatch(pageSource, /formatAgeText/);
  assert.match(pageSource, /insurancePolicyNo/);
  assert.match(pageSource, /insurance_policy_no/);
  assert.doesNotMatch(pageSource, /experienceNote/);
  assert.doesNotMatch(pageSource, /experience_note/);
  assert.match(pageSource, /emergency_contact_relationship/);
  assert.match(pageSource, /medical_response_note/);
  assert.match(pageSource, /diet_preference/);
  assert.match(pageSource, /insurance_company_phone/);
  assert.match(pageSource, /showOfflineWriteBlockedToast/);
  assert.match(pageSource, /parseHeightCm/);
});

test("profile outdoor experiences page manages structured experience records", () => {
  const appConfig = JSON.parse(read("app.json"));
  const settingsWxml = read("pages/profile/settings/index.wxml");
  const settingsTs = read("pages/profile/settings/index.ts");
  const wxml = read("pages/profile/outdoor-experiences/index.wxml");
  const ts = read("pages/profile/outdoor-experiences/index.ts");
  const wxss = read("pages/profile/outdoor-experiences/index.wxss");
  const pageSource = `${wxml}
${ts}`;

  assert.ok(
    appConfig.pages.includes("pages/profile/outdoor-experiences/index"),
  );
  assert.match(settingsWxml, /户外经历/);
  assert.match(settingsTs, /openOutdoorExperiences/);
  assert.match(settingsTs, /\/pages\/profile\/outdoor-experiences\/index/);
  assert.match(wxml, /户外经历/);
  assert.match(wxml, /新增经历/);
  assert.match(wxml, /experience-card/);
  assert.match(wxml, /item\.dateText/);
  assert.match(wxml, /item\.metaText/);
  assert.match(wxml, /item\.sourceText/);
  assert.match(wxml, /summaryLines/);
  assert.match(wxml, /编辑户外经历|新增户外经历|\{\{editorTitle\}\}/);
  assert.match(wxml, /路线摘要/);
  assert.match(wxml, /装备经验/);
  assert.match(wxml, /食品经验/);
  assert.match(wxml, /预算摘要/);
  assert.match(wxml, /bindtap="saveExperience"/);
  assert.match(wxml, /catchtap="openEditEditor"/);
  assert.match(wxml, /catchtap="confirmDeleteExperience"/);
  assert.match(pageSource, /listOutdoorExperiences/);
  assert.match(pageSource, /createOutdoorExperience/);
  assert.match(pageSource, /updateOutdoorExperience/);
  assert.match(pageSource, /deleteOutdoorExperience/);
  assert.match(pageSource, /stellartrail_outdoor_experiences_refresh/);
  assert.match(pageSource, /showOfflineWriteBlockedToast/);
  assert.match(wxss, /\.experience-sheet/);
  assert.match(wxss, /\.experience-actions \.danger/);
});

test("trip index exposes past trip conversion to outdoor experiences", () => {
  const wxml = read("pages/trips/index.wxml");
  const ts = read("pages/trips/index.ts");
  const wxss = read("pages/trips/index.wxss");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /转为经历/);
  assert.match(wxml, /已记录/);
  assert.match(wxml, /trip\.canConvertToExperience/);
  assert.match(wxml, /trip\.convertedToExperience/);
  assert.match(wxml, /catchtap="convertPlanToExperience"/);
  assert.match(wxml, /loading="\{\{convertingTripId === trip\.id\}\}"/);
  assert.match(pageSource, /convertTripToOutdoorExperience/);
  assert.match(pageSource, /outdoor_experience_id/);
  assert.match(pageSource, /time_bucket === "past"/);
  assert.match(pageSource, /stellartrail_outdoor_experiences_refresh/);
  assert.match(wxss, /\.experience-recorded/);
  assert.match(wxss, /\.experience-convert-button/);
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

test("home gear summary keeps locked cards without guest browse prompt", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const wxss = read("pages/index/index.wxss");

  assert.doesNotMatch(wxml, /showLoginForGearSummary/);
  assert.doesNotMatch(wxml, /guest-inline/);
  assert.doesNotMatch(wxss, /\.guest-inline/);
  assert.match(ts, /LOCKED_GEAR_STATS/);
  assert.match(ts, /登录后可见/);
  assert.match(ts, /buildHeroStatusText/);
  assert.doesNotMatch(ts, /登录后快速记录装备/);
  assert.doesNotMatch(ts, /我的装备已保存/);
  assert.match(ts, /value: "—"/);
});

test("gear page logged-out and logged-in cards share surface tokens", () => {
  const wxml = read("pages/gears/index.wxml");
  const ts = read("pages/gears/index.ts");
  const wxss = read("pages/gears/index.wxss");
  const toolbarBlock = wxss.match(/\.gear-toolbar \{[\s\S]*?\n\}/)?.[0] ?? "";
  const toolbarSearchBlock =
    wxss.match(/\.toolbar-search \{[\s\S]*?\n\}/)?.[0] ?? "";
  const toolbarLoginFilterBlock =
    wxss.match(/\.toolbar-filter\.requires-login \{[\s\S]*?\n\}/)?.[0] ?? "";
  const toolbarAddBlock =
    wxss.match(/\.toolbar-add \{[\s\S]*?\n\}/)?.[0] ?? "";
  const toolbarAddIconBlock =
    wxss.match(/\.toolbar-add-icon \{[\s\S]*?\n\}/)?.[0] ?? "";
  const statsPanelBlock = wxss.match(/\.stats-panel \{[\s\S]*?\n\}/)?.[0] ?? "";
  const statsDetailBlock =
    wxss.match(/\.stats-detail-link \{[\s\S]*?\n\}/)?.[0] ?? "";
  const statsGridBlock = wxss.match(/\.stats-grid \{[\s\S]*?\n\}/)?.[0] ?? "";
  const gearFactsBlock = wxss.match(/\.gear-facts \{[\s\S]*?\n\}/)?.[0] ?? "";
  const sheetBlock = wxss.match(/\.filter-sheet \{[\s\S]*?\n\}/)?.[0] ?? "";
  const sheetChipBlock = wxss.match(/\.sheet-chip \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.doesNotMatch(wxml, /class="hero-card"/);
  assert.doesNotMatch(wxml, /class="filter-panel"/);
  assert.doesNotMatch(wxml, /class="search-card"/);
  assert.doesNotMatch(wxml, /class="gear-add-card"/);
  assert.doesNotMatch(wxml, /tab-card-title">列表范围/);
  assert.doesNotMatch(wxml, /class="tab-switch"/);
  assert.match(wxml, /class="gear-toolbar"/);
  assert.match(wxml, /class="toolbar-search-input"/);
  assert.match(wxml, /bindtap="openFilterSheet"/);
  assert.match(wxml, /\{\{!isLoggedIn \? 'requires-login' : ''\}\}/);
  assert.match(wxml, /class="toolbar-add"[\s\S]*bindtap="goCreate"/);
  assert.match(wxml, /<text class="toolbar-add-icon">\+<\/text>/);
  assert.match(wxml, /bindtap="handleEmptyAction"/);
  assert.match(ts, /emptyText: "登录后才能管理装备"/);
  assert.match(ts, /emptyActionText: "登录管理装备"/);
  assert.match(ts, /message: "登录后可以筛选和管理自己的装备。"/);
  assert.match(ts, /message: "登录后可以查看自己的装备统计。"/);
  assert.match(ts, /message: "登录后可以管理自己的打包清单。"/);
  assert.match(wxml, /class="filter-summary"/);
  assert.match(wxml, /\{\{activeFilterText\}\}/);
  assert.match(wxml, /class="stats-panel"/);
  assert.match(wxml, /class="stats-panel-header"/);
  assert.match(wxml, /bindtap="goStatsDetail"/);
  assert.match(wxml, /详细统计/);
  assert.match(wxml, /当前库存汇总/);
  assert.match(wxml, /class="quick-entry-grid"/);
  assert.match(wxml, /class="gear-list"/);
  assert.match(wxml, /class="filter-sheet-mask"/);
  assert.match(wxml, /filter-sheet-title">筛选装备/);
  assert.match(wxml, /bindtap="selectDraftCategory"/);
  assert.match(wxml, /bindtap="selectDraftStatus"/);
  assert.match(wxml, /bindtap="selectDraftSort"/);
  assert.match(wxml, /bindtap="applyFilters"/);
  assert.doesNotMatch(wxml, /去添加/);
  assert.ok(
    wxml.indexOf('class="quick-entry-grid"') <
      wxml.indexOf('class="stats-panel"'),
    "auxiliary entries should lead the logged-in content",
  );
  assert.ok(
    wxml.indexOf('class="stats-panel"') < wxml.indexOf('class="gear-toolbar"'),
    "search and filter toolbar should appear below compact stats",
  );
  assert.ok(
    wxml.indexOf('class="gear-toolbar"') <
      wxml.indexOf('class="filter-summary"'),
    "filter summary should appear below the search toolbar",
  );
  assert.ok(
    wxml.indexOf('class="filter-summary"') < wxml.indexOf('class="gear-list"'),
    "gear list should appear after the restored auxiliary information",
  );
  assert.doesNotMatch(wxml, /class="filter-pill"/);

  assert.match(toolbarBlock, /border: 1rpx solid var\(--border-color\)/);
  assert.match(toolbarBlock, /background: var\(--surface-color\)/);
  assert.match(toolbarBlock, /box-shadow: var\(--shadow-soft\)/);
  assert.match(toolbarSearchBlock, /background: var\(--control-bg\)/);
  assert.match(toolbarLoginFilterBlock, /background: var\(--brand-soft-bg\)/);
  assert.match(toolbarLoginFilterBlock, /color: var\(--brand-soft-text\)/);
  assert.match(toolbarAddBlock, /display: flex/);
  assert.match(toolbarAddBlock, /align-items: center/);
  assert.match(toolbarAddBlock, /justify-content: center/);
  assert.match(toolbarAddBlock, /padding: 0/);
  assert.match(toolbarAddBlock, /line-height: 1/);
  assert.match(toolbarAddIconBlock, /line-height: 1/);
  assert.match(statsPanelBlock, /padding: 20rpx/);
  assert.match(statsDetailBlock, /background: var\(--brand-soft-bg\)/);
  assert.match(
    statsGridBlock,
    /grid-template-columns: repeat\(2, minmax\(0, 1fr\)\)/,
  );
  assert.match(gearFactsBlock, /background: var\(--control-bg\)/);
  assert.match(sheetBlock, /width: 100%/);
  assert.match(sheetBlock, /border-radius: 34rpx 34rpx 0 0/);
  assert.match(sheetChipBlock, /background: var\(--control-bg\)/);
  assert.doesNotMatch(wxss, /\.hero-card\s*\{/);
  assert.doesNotMatch(wxss, /\.filter-panel\s*\{/);
  assert.doesNotMatch(wxss, /\.gear-add-card\s*\{/);
});

test("packing lists pages expose create select and checklist flows", () => {
  const appJson = read("app.json");
  const gearListWxml = read("pages/gears/index.wxml");
  const gearListTs = read("pages/gears/index.ts");
  const packingIndexWxml = read("pages/packing-lists/index.wxml");
  const packingIndexTs = read("pages/packing-lists/index.ts");
  const packingFormWxml = read("pages/packing-lists/form/index.wxml");
  const packingFormTs = read("pages/packing-lists/form/index.ts");
  const packingDetailWxml = read("pages/packing-lists/detail/index.wxml");
  const packingDetailTs = read("pages/packing-lists/detail/index.ts");
  const packingSelectWxml = read("pages/packing-lists/select-gears/index.wxml");
  const packingSelectTs = read("pages/packing-lists/select-gears/index.ts");

  assert.match(appJson, /pages\/packing-lists\/index/);
  assert.match(appJson, /pages\/packing-lists\/select-gears\/index/);
  assert.match(gearListWxml, /打包清单/);
  assert.match(gearListTs, /goPackingLists/);
  assert.match(packingIndexWxml, /新建第一份清单/);
  assert.match(packingIndexWxml, /wx:if="\{\{item\.metaText\}\}"/);
  assert.match(packingIndexTs, /listGearPackingLists/);
  assert.match(packingFormWxml, /清单名称必填；路线\/目的地和徒步时长可选。/);
  assert.match(packingFormWxml, /路线\/目的地（可选）/);
  assert.match(packingFormWxml, /徒步时长（可选）/);
  assert.match(packingFormTs, /createGearPackingList/);
  assert.match(packingFormTs, /select-gears\/index\?id=/);
  assert.match(packingFormTs, /returnTripId/);
  assert.match(packingDetailWxml, /bindtap="togglePacked"/);
  assert.match(packingDetailWxml, /已打包/);
  assert.match(packingDetailWxml, /wx:if="\{\{metaText\}\}"/);
  assert.match(packingDetailTs, /updateGearPackingItem/);
  assert.match(packingSelectWxml, /class="filter-panel"/);
  assert.match(packingSelectWxml, /filter-panel-title">筛选/);
  assert.match(packingSelectWxml, /filter-panel-hint">仅可用装备/);
  assert.match(packingSelectWxml, /filter-section-label">分类/);
  assert.match(packingSelectWxml, /class="filter-label">状态/);
  assert.match(packingSelectWxml, /class="filter-label">排序/);
  assert.doesNotMatch(packingSelectWxml, /class="filter-pill"/);
  assert.match(packingSelectWxml, /加入 \{\{selectedIds\.length\}\} 件装备/);
  assert.match(packingSelectTs, /addGearPackingItems/);
  assert.match(packingSelectTs, /importTripPackingList/);
  assert.match(packingSelectTs, /section=personal_gear/);
  assert.match(packingSelectTs, /showOfflineWriteBlockedToast/);
});

test("gear views expose direct delete action", () => {
  const listWxml = read("pages/gears/index.wxml");
  const listTs = read("pages/gears/index.ts");
  const detailWxml = read("pages/gears/detail/index.wxml");
  const detailTs = read("pages/gears/detail/index.ts");

  assert.doesNotMatch(listWxml, /tab === 'history'/);
  assert.match(listWxml, /catchtap="deleteItem"/);
  assert.doesNotMatch(detailWxml, /tab === 'history'/);
  assert.match(detailWxml, /bindtap="deleteItem"/);
  assert.match(listTs, /deleteGear/);
  assert.match(detailTs, /deleteGear/);
  assert.match(listTs, /删除自己的装备/);
  assert.match(detailTs, /删除自己的装备/);
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
