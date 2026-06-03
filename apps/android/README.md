# StellarTrail Android

StellarTrail Android 是原生 Kotlin + Jetpack Compose 客户端，复用仓库现有 Rust API 与共享 DTO 语义。

## 本地运行

环境要求：

- JDK 21
- Android SDK 36 / Build Tools 36.0.0
- 仓库默认 API 占位地址：`https://api.example.invalid`
- 仓库默认图片资源 / CORS 资源占位域名：`https://assets.example.invalid`

构建时会读取 Git 忽略的 `config.properties`，缺失时回退到 `config.example.properties`。本地联调可复制示例文件并把 API 地址改为 `http://10.0.2.2:8080`，模拟器即可访问宿主机服务。真机联调时也可在 Profile 页面临时修改 API Base URL。

```bash
cp apps/android/config.example.properties apps/android/config.properties
```

`config.properties` 支持以下键：

```properties
stellartrail.apiBaseUrl=https://api.example.invalid
stellartrail.assetsBaseUrl=https://assets.example.invalid
stellartrail.domainCandidates=
stellartrail.smsCodeCooldownSeconds=60
```

`stellartrail.domainCandidates` 格式为 `id|apiBaseUrl|assetsBaseUrl;id|apiBaseUrl|assetsBaseUrl`。生产构建通过 GitHub Actions Secrets 写入真实配置；仓库只保留示例占位值。

常用 Debug 验证：

```bash
./gradlew :apps:android:testDebugUnitTest :apps:android:lintDebug :apps:android:assembleDebug
```

## 功能范围

- 登录/注册：账号密码登录、验证码登录、手机号注册、手机号找回密码、本地 access/refresh token 会话加密存储与 401 自动续期。
- 装备库：分类/状态/搜索/排序筛选，列表分页，详情，新增，编辑，删除归档，历史恢复。
- 行程：行程列表、创建、详情、加入行程、成员与多类协作资料。
- 打包清单：出发物品准备入口。
- 技能：技能分类与绳结列表/详情，支持媒体资源和本地离线只读缓存。
- 首页：装备统计、近期行程、出发前检查和快速入口。
- Profile：当前用户、头像、手机号、主题模式、设置与帮助、缓存、反馈、关于和本地登出。

## 本地 Release 签名

Debug 构建不需要签名变量。Release 构建会直接读取以下环境变量，缺少任何一个都会失败：

```bash
export STELLARTRAIL_ANDROID_KEYSTORE_PATH=/private/tmp/stellartrail-android-release.jks
export STELLARTRAIL_ANDROID_KEYSTORE_PASSWORD='<KEYSTORE_PASSWORD>'
export STELLARTRAIL_ANDROID_KEY_ALIAS='stellartrail-release'
export STELLARTRAIL_ANDROID_KEY_PASSWORD='<KEY_PASSWORD>'

./gradlew :apps:android:assembleRelease
```

本地测试签名可使用临时 keystore；正式 keystore 必须由维护者安全保存，不写入仓库。

## CI Signed Release APK

`Android Release` workflow 会在以下场景构建 signed release APK：

- 合入 `main` 后，如果 Android、Gradle、Android release workflow 或相关 CI 脚本有变动，生成 main 构建 artifact。
- 指向 `main` 可达提交的 `vX.Y.Z` 或 `vX.Y` 标签进入仓库后，生成 GitHub Release 资产。

main 构建 artifact 文件名：

- `StellarTrail-main-<short_sha>-android-release.apk`
- `StellarTrail-main-<short_sha>-android-release.apk.sha256`

tag Release 资产文件名：

- `StellarTrail-<tag>-android-release.apk`
- `StellarTrail-<tag>-android-release.apk.sha256`

下载方式：

- main 合入构建：进入 GitHub 仓库的 **Actions** 页面，打开对应 `Android Release` run，在 **Artifacts** 下载 signed APK 和 SHA-256 文件。
- tag 发布构建：进入对应 GitHub Release 页面下载 APK 和 SHA-256 文件。

## GitHub Actions Secrets

使用仓库级 Actions Secrets，不使用 Environment Secrets。真实域名、keystore 和密码只进入 GitHub Secrets，不写入 tracked 文件。

### 1. 准备 Android release keystore

如果已有正式 keystore，使用已有文件，不重新生成。没有正式 keystore 时可生成一个，并由维护者离线保存：

```bash
keytool -genkeypair \
  -v \
  -keystore /private/tmp/stellartrail-android-release.jks \
  -storetype JKS \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -alias stellartrail-release
```

### 2. 设置真实域名 Secrets

优先从 stdin 或临时文件读取真实值，避免把真实值写入 shell history：

```bash
printf '%s' '<REAL_API_BASE_URL>' | gh secret set STELLARTRAIL_ANDROID_API_BASE_URL --repo rustella/StellarTrail
printf '%s' '<REAL_ASSETS_BASE_URL>' | gh secret set STELLARTRAIL_ANDROID_ASSETS_BASE_URL --repo rustella/StellarTrail
printf '%s' '<id|api|assets;id2|api2|assets2 or empty>' | gh secret set STELLARTRAIL_ANDROID_DOMAIN_CANDIDATES --repo rustella/StellarTrail
```

### 3. 设置签名 Secrets

```bash
base64 < /private/tmp/stellartrail-android-release.jks | tr -d '\n' | gh secret set STELLARTRAIL_ANDROID_RELEASE_KEYSTORE_BASE64 --repo rustella/StellarTrail
printf '%s' '<KEYSTORE_PASSWORD>' | gh secret set STELLARTRAIL_ANDROID_KEYSTORE_PASSWORD --repo rustella/StellarTrail
printf '%s' 'stellartrail-release' | gh secret set STELLARTRAIL_ANDROID_KEY_ALIAS --repo rustella/StellarTrail
printf '%s' '<KEY_PASSWORD>' | gh secret set STELLARTRAIL_ANDROID_KEY_PASSWORD --repo rustella/StellarTrail
```

### 4. 验证 Secrets 名称

只验证名称，不读取值：

```bash
gh secret list --repo rustella/StellarTrail
```

期望看到：

- `STELLARTRAIL_ANDROID_API_BASE_URL`
- `STELLARTRAIL_ANDROID_ASSETS_BASE_URL`
- `STELLARTRAIL_ANDROID_DOMAIN_CANDIDATES`
- `STELLARTRAIL_ANDROID_RELEASE_KEYSTORE_BASE64`
- `STELLARTRAIL_ANDROID_KEYSTORE_PASSWORD`
- `STELLARTRAIL_ANDROID_KEY_ALIAS`
- `STELLARTRAIL_ANDROID_KEY_PASSWORD`

## 验证说明

自动化验证覆盖 JVM 单测、Debug 构建、Android Lint 和本地临时签名 Release 构建。`connectedAndroidTest` 需要可用模拟器或真机，并需要本地/测试 API 服务配合，不作为默认 CI 阶段。
