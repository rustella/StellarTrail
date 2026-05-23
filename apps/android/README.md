# StellarTrail Android

StellarTrail Android 是原生 Kotlin + Jetpack Compose 客户端，复用仓库现有 Rust API 与共享 DTO 语义。

## 本地运行

环境要求：

- JDK 21
- Android SDK 36 / Build Tools 36.0.0
- 默认 API 地址：`https://api.example.invalid`
- 默认图片资源 / CORS 资源域名：`https://assets.example.invalid`
- 生产环境会在首次 API 请求前按顺序探测 `api.example.invalid`、`api-alt1.example.invalid`、`api-alt2.example.invalid` 的 `/healthz`，首个可用域名族会同时决定 API 和资源域名；本地调试地址会跳过探测。

构建时会读取 Git 忽略的 `config.properties`，缺失时回退到 `config.example.properties`。本地联调可复制示例文件并把 API 地址改为 `http://10.0.2.2:8080`，模拟器即可访问宿主机服务。真机联调时也可在 Profile 页面临时修改 API Base URL。

```bash
./gradlew :apps:android:assembleDebug
./gradlew :apps:android:testDebugUnitTest
./gradlew :apps:android:lintDebug
```

## 功能范围

- 登录/注册：用户名或邮箱 + 密码；邮箱验证码请求；本地 access/refresh token 会话加密存储与 401 自动续期。
- 装备库：分类/状态/搜索/排序筛选，列表分页，详情，新增，编辑，删除归档，历史恢复。
- 技能：技能分类与绳结列表/详情，使用 `X-StellarTrail-Locale` 请求头。
- 首页：装备统计、分类摘要、快速入口。
- Profile：当前用户、主题模式、API Base URL、本地登出。

## 验证说明

自动化验证覆盖 JVM 单测、Debug 构建与 Android Lint。`connectedAndroidTest` 需要可用模拟器或真机，并需要本地/测试 API 服务配合，不作为默认 CI 阶段。

## 发布 / APK 下载

Android 发布默认使用指向 `main` 可达提交的 `vX.Y.Z` 标签（脚本也兼容 `vX.Y` 形式）。标签随合入或推送进入 `main` 后，`Android Release` workflow 会检测尚未创建 GitHub Release 的新标签，构建 Debug APK，生成发布说明，并上传到对应 Release。

默认下载资产为 `StellarTrail-<tag>-android-debug.apk` 与 `StellarTrail-<tag>-android-debug.apk.sha256`；GitHub Release 页面是 APK 的标准下载位置。当前产物是 Debug APK，因为仓库尚未配置 release signing；如需正式签名 APK，需要先添加 GitHub Secrets 与 Gradle 签名配置，再切换到 release 构建。
