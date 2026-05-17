# StellarTrail Android

StellarTrail Android 是原生 Kotlin + Jetpack Compose 客户端，复用仓库现有 Rust API 与共享 DTO 语义。

## 本地运行

环境要求：

- JDK 21
- Android SDK 36 / Build Tools 36.0.0
- 默认 API 地址：`https://api.stellartrail.cn`
- 默认图片资源 / CORS 资源域名：`https://assets.stellartrail.cn`

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
