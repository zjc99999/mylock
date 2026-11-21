# MyLock · Android 最小可用 —— 可行性评估与绑定路径设计

> **文档性质**：可行性评估 + 工程设计（**不是已实现、不是产品发布、不是第三方审计**）。
> **本轮范围**：用户授权「零额外开支完成软件研发 + 小范围试用」中的「Android 最小可用」子项；**不要求产出 APK，本轮也不产出可侧载 APK**。
> **唯一写入文件**：本文档。未修改 `core/**/*.rs`、`desktop/**`、`产品分析与概要设计.md` 及任何既有报告。
> **事实抓取时点**：2026-11-15（合规与版本类事实均为该时点值，非永久有效）。
> **工作目录**：`D:\Projects\dsh\mylock`（Rust workspace 位于 `core/`，桌面壳 `desktop/src-tauri` 为 Tauri 2）。

## 分级定义（全文适用）

| 级别 | 含义 |
|---|---|
| **A** | 一手官方页面 + 精确引文（URL 为官方域名，正文逐字核对） |
| **A⁻** | 官方 PDF，或官方页面的忠实转载 / 官方中文机翻页（正文语义一致，措辞经翻译） |
| **B** | 工程判断 / 设计推荐 / 推理结论（由 A、A⁻ 事实推导，非官方原文） |
| **C** | 常识性推测（本文尽量不用；用到时显式标注） |
| **本地** | 仓库内文件事实，无外部 URL，以 `path:line` 形式标注来源，不与 A/A⁻/B/C 混淆 |

> **抓取方法说明**：`web_fetch` 对 `developer.android.com/*`、`source.android.com/*`、`github.com/*`、`raw.githubusercontent.com/*` 持续返回 `TypeError: fetch failed`；对 `developer.apple.com/documentation/security/...` 首次仅返回标题。上述页面凡本报告引用者，均已改用 **ego-browser 打开官方页读取 body 正文** 补抓，故按 A / A⁻ 计。确实未拿到的条目集中列于「附录 A · NOT RETRIEVED 清单」，**本报告不对其内容作任何断言，绝不臆造**。

---

## §0 结论先行

### 0.1 本轮能交付什么

- **一份可执行的工程设计**：Android 构建的精确前置、`cargo tauri android init` / `android build` 流程、三个内核绑定路径的对比与推荐、「生物暗格」的正确密钥学形态、零现金前置清单、G4 验收判据。
- **一份诚实的环境盘点**：本机当前缺什么、哪些步骤零现金、哪些必须用户自行安装、哪些明确暂缓。
- **一条冻结的 A 级架构约束**（见 §2.4）：**跨端主密钥必须是可移植软件密钥**，硬件密钥（Secure Enclave / Android Keystore）只能在本机 wrap 它。此约束不因 Android 端加入而改变。
- **全部事实带 URL 与分级**，抓取失败的条目显式标 `NOT RETRIEVED`。

### 0.2 本轮不能交付什么

- **不产出任何 APK / AAB**，也不产出可侧载包。原因不是设计不成立，而是本机不具备构建链（见 §0.3、§5.1）：无 Android SDK / NDK / gradle / adb / sdkmanager / emulator，环境变量全 unset，无 Android rust target。
- **不实现代码**：不碰 `core/**/*.rs`、不碰 `desktop/**`、不新建 Android 工程。
- **不做 iOS / TestFlight**：Apple Developer Program 为 **$99/年**（A，https://developer.apple.com/programs/），本轮「零额外开支」目标下明确暂缓（见 §4.4）。
- **不写法律结论**：§5.4 仅转述官方 FAQ / 官方页面文字，不构成合规意见。

### 0.3 NO-GO 的准确边界

- 当前 **NO-GO 只约束「公开发布 / 上架分发」**，不约束设计、不约束本机内测、不约束源码研发。
- 依据（A）：Tauri 官方签名页明确「To publish on the Play Store, you need to sign your app with a digital certificate. Android App Bundles and APKs must be signed before being uploaded for distribution.」—— https://v2.tauri.app/distribute/sign/android/ ；Google Play 页明确「To distribute Android apps in the Play Store you must create a Play Console developer account.」—— https://v2.tauri.app/distribute/google-play/ 。**签名与开发者账号是「分发」前置，不是「构建/自测」前置**（B，由上两句推导）。
- 因此：小范围试用只需 `--apk` 侧载包（本机自签即可），而 `--aab` + 上传密钥 + Play Console 账号属于公开发布链路，**本轮整体后置**。

---

## §1 Tauri 2 Android 构建前置

### 1.1 官方前置组件（A）

来源：https://v2.tauri.app/start/prerequisites/ （页面标注 *Last updated: Aug 20, 2026*），Android 段逐字：

> 「Download and install Android Studio from the Android Developers website」
> 「Set the JAVA_HOME environment variable」
> 「Use the SDK Manager in Android Studio to install the following: Android SDK Platform / Android SDK Platform-Tools / NDK (Side by side) / Android SDK Build-Tools / Android SDK Command-line Tools」
> 「Selecting "Show Package Details" ... Only install older versions if necessary」
> 「Set ANDROID_HOME and NDK_HOME environment variables.」

同页 Windows 段另列：Microsoft C++ Build Tools（勾选 **Desktop development with C++**）、Edge WebView2、VBSCRIPT（仅 MSI 安装器相关）（A，同上 URL）。

NDK 侧官方定义（A⁻，中文机翻，最后更新 2026-03-06）：https://developer.android.com/ndk/guides
> 「原生开发套件 (NDK) 是一套工具，使您能够在 Android 应用中使用 C 和 C++ 代码」
> 「Android Studio 编译原生库的默认构建工具是 CMake」
> 「Java 代码随后可以通过 Java 原生接口 (JNI) 框架调用原生库中的函数」
> 「Gradle 使用构建脚本将源代码导入……并将原生库（SO 文件）打包到 APK 中」
> 「如需为您的应用编译和调试原生代码，您需要以下组件：Android 原生开发套件 (NDK) / CMake / LLDB」

### 1.2 精确的 SDK / NDK 版本要求（诚实边界）

**必须先说清楚一个容易臆造的点**：Tauri 官方前置页 **只列出组件名，不给精确的 SDK Platform API level，也不绑定某个精确 NDK 版本号**（A，https://v2.tauri.app/start/prerequisites/ 全文核对）。它写的是「Use the SDK Manager ... to install ... NDK (Side by side)」，并提示 `Show Package Details` 里「Only install older versions if necessary」。因此：

| 项目 | 精确值 | 来源 / 分级 | 备注 |
|---|---|---|---|
| NDK 最新 LTS | **r30**，`android { ndkVersion "30.0.16248370" }` | A⁻，https://developer.android.com/ndk/downloads （最后更新 2026-09-10，中文机翻） | 这是「NDK 下载页给的当前 LTS 精确版本」，**不是 Tauri 强制绑定的版本** |
| NDK Windows 64 位包 | `android-ndk-r30-windows.zip`，727980788 字节，SHA1 `9bf167a1985fa7d4a036186b78f702eab9179408` | A⁻，同上 | 官方同时写「在大多数情况下，使用 Android SDK 管理器来安装 NDK 会更轻松」 |
| SDK Command-line Tools（Windows） | `commandlinetools-win-15859902_latest.zip`，155.7 MB，SHA-256 `90ae805d20434428bffcb699c290860f19bb5f66a67e6b330067e3de801fb04a` | A⁻，https://developer.android.com/studio#command-line-tools | 精确下载 URL 见 §4.2 |
| Android SDK Platform API level | **官方前置页未给精确数值** | A（「未给」这一事实），https://v2.tauri.app/start/prerequisites/ | 由 SDK Manager 选择；**本报告不臆造 API 33/34/35 之类数字** |
| gradle 版本 | **本报告未抓取到 Tauri 生成的 Android 工程所固定的 gradle 版本号 → NOT RETRIEVED** | 附录 A | gradle 由 `tauri android init` 生成的工程 wrapper 提供（B，由 https://v2.tauri.app/reference/cli/ 的 `android init`「Initialize Android target in the project」推导） |

> 结论（B）：**可精确引用的是「NDK r30 / `30.0.16248370`」与「cmdline-tools 包名 + SHA」**；SDK Platform 的 API level 与 gradle 精确版本在本轮资料中未由官方页给出，实施时以 `sdkmanager --list` 与生成工程的 `gradle-wrapper.properties` 为准，不在设计文档里编造。

### 1.3 需要的 Rust target（A）

来源：https://v2.tauri.app/start/prerequisites/ ，原文：

> 「Add the Android targets with rustup:」
> `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`

四个 target 与 `cargo tauri android build` 的 `-t/--target` 取值一一对应（A，https://v2.tauri.app/reference/cli/ ）：

| rust target | `android build --target` 取值 | 架构 |
|---|---|---|
| `aarch64-linux-android` | `aarch64` | ARM64（真机主流） |
| `armv7-linux-androideabi` | `armv7` | ARMv7 |
| `i686-linux-android` | `i686` | x86 32 位（模拟器） |
| `x86_64-linux-android` | `x86_64` | x86_64（模拟器） |

CLI 页原文（A）：`android build` 的 `-t/--target [<TARGETS>...]` possible values 为 `aarch64, armv7, i686, x86_64`，**默认 all**；另有 `--split-per-abi`、`--apk`、`--aab`、`--open`、`--ci`。

本机现状（本地 / 用户授权材料）：`rustup target list --installed` **只有 `x86_64-pc-windows-gnu`**，上述四个 Android target 一个都没有（§5.1 复核）。

### 1.4 `cargo tauri android init` / `android build` 流程（A）

来源：https://v2.tauri.app/reference/cli/ （`android` 子命令全文，ego-browser 实抓）与 https://v2.tauri.app/distribute/google-play/ 。

**命令表**（A，https://v2.tauri.app/reference/cli/ ）：

| 子命令 | 官方释义 |
|---|---|
| `android init` | 「Initialize Android target in the project」 |
| `android build` | 「Build your app in release mode for Android and generate APKs and AABs」 |
| `android dev` | 开发模式（含 `--open`「Open Android Studio instead of trying to run on a connected device」） |
| `android run` | 运行到设备 |

**`android init` 细节**（A，同上）：Usage 形如 `cargo tauri android init [OPTIONS]`；选项含 `--ci`、`--skip-targets-install`（原文「Skips installing rust toolchains via rustup」）、`-c/--config`。生成目录由 `TAURI_ANDROID_PROJECT_PATH` 指向，**默认 `<project>/src-tauri/gen/android`**（A，https://v2.tauri.app/reference/environment-variables/ ：「Path of the tauri android project, usually will be <project>/src-tauri/gen/android.」）。平台专属配置文件 `tauri.android.conf.json` 默认被查找合并（A，https://v2.tauri.app/reference/cli/ ）。

**推荐流程（B，由 A 事实编排）**：

```bat
:: 0) 前置：装好 SDK/NDK/JDK，设好 JAVA_HOME / ANDROID_HOME / NDK_HOME，四个 rust target 已 add
cargo tauri android init

:: 1) 开发自测（真机或模拟器）
cargo tauri android dev

:: 2) 出可侧载 APK（小范围试用用这个，本机自签即可）
cargo tauri android build --apk --target aarch64

:: 3) 出 AAB（仅公开发布上架时才需要；本轮后置）
cargo tauri android build --aab
```

依据（A）：https://v2.tauri.app/distribute/google-play/ 给出 `npm run tauri android build -- --aab`（AAB）与 `--apk`（「生成可侧载 APK」），并写明「After running `tauri android init`」；同页 `versionCode = major*1000000 + minor*1000 + patch`。签名密钥配置（A，https://v2.tauri.app/distribute/sign/android/ ）：`keytool -genkey -v -keystore ~/upload-keystore.jks -keyalg RSA -keysize 2048 -validity 10000 -alias upload`，写入 `[project]/src-tauri/gen/android/keystore.properties`（password / keyAlias / storeFile），并警告「Keep the keystore file private; don't check it into public source control!」。**该签名链路只在上架时需要**（B）。

### 1.5 环境变量与 Windows 注意事项

| 变量 | 官方写法 / 事实 | 来源 / 分级 |
|---|---|---|
| `ANDROID_HOME` | `export ANDROID_HOME="$HOME/Android/Sdk"`（官方给的是 Linux 示例） | A，https://v2.tauri.app/start/prerequisites/ |
| `NDK_HOME` | `export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)"` | A，同上 |
| `JAVA_HOME` | 「Set the JAVA_HOME environment variable」；官方示例路径 `/opt/android-studio/jbr`（Android Studio 自带 JBR） | A，同上 |
| PATH | 「请设置命令搜索路径以加入 ANDROID_HOME/tools、ANDROID_HOME/tools/bin 和 ANDROID_HOME/platform-tools」；cmdline-tools 实际位于 `android_sdk/cmdline-tools/version/bin/` | A⁻，https://developer.android.com/tools （最后更新 2026-09-25） |
| `TAURI_ANDROID_PROJECT_PATH` | 「usually will be <project>/src-tauri/gen/android」 | A，https://v2.tauri.app/reference/environment-variables/ （*Last updated: Jan 30, 2026*） |
| `sdkmanager` | 「用于查看、安装、更新和卸载 Android SDK 的软件包」；platform-tools 含 adb「还可以用它在设备上安装 APK」；build-tools 含 apksigner / zipalign / AAPT2 | A⁻，https://developer.android.com/tools |

**Windows 专属注意（A / A⁻ / B）**：

1. Windows 需 Microsoft C++ Build Tools（**Desktop development with C++**）、Edge WebView2、VBSCRIPT（仅 MSI 安装器）（A，https://v2.tauri.app/start/prerequisites/ Windows 段）。
2. 官方 `ANDROID_HOME` / `NDK_HOME` 示例是 bash 语法与 Linux 路径，**Windows 上应改写为「系统/用户环境变量」而非 `.bashrc`**（B，由 A 事实推导）；`%ANDROID_HOME%\ndk\<version>` 对应 `NDK_HOME`（B）。
3. `android dev` 的 `--host` 选项官方注明「**On Windows we use the public network address by default.**」（A，https://v2.tauri.app/reference/cli/ ）——即 Windows 上开发服务器默认绑公网地址，真机调试时注意网络暴露面（B，提示性判断）。
4. Tauri 移动端插件页含「**Android 16KB Memory Pages**」章节（A，https://v2.tauri.app/develop/plugins/develop-mobile/ ）——Rust `.so` 需按 16KB 页对齐构建，实施时须核对（B）。
5. 本机已存在 `java 26.0.1`（本地复核，§5.1），但 **JAVA_HOME 未设置**（`echo` 为空），需显式指向 JDK/JBR 根目录（B）。

### 1.6 逐项「是否免费」清单（核心：零额外开支可达成）

| 项目 | 是否免费（零现金） | 依据 / 分级 |
|---|---|---|
| Android Studio（含 SDK Manager、自带 JBR/JDK） | ✅ 免费下载 | A⁻，https://developer.android.com/studio#command-line-tools 提供安装包；「Android Studio 中包含命令行工具」 |
| Android SDK **Command-line Tools**（仅命令行，无需 Studio） | ✅ 免费下载 | A⁻，同上：「如果您不需要 Android Studio，可以从上面下载基本的 Android 命令行工具。您可以使用随附的 sdkmanager 来下载其他 SDK 软件包。」 |
| Android SDK Platform / Platform-Tools / Build-Tools | ✅ 免费（sdkmanager 下载） | A⁻，https://developer.android.com/tools ；A，Tauri 前置页列为 SDK Manager 安装项 |
| **NDK (Side by side) + CMake + LLDB** | ✅ 免费下载 | A，https://v2.tauri.app/start/prerequisites/ 列为 SDK Manager 安装项；A⁻，https://developer.android.com/ndk/downloads 提供独立 ZIP |
| **四个 Rust target** | ✅ 免费（`rustup target add`，零现金下载） | A，https://v2.tauri.app/start/prerequisites/ 「Add the Android targets with rustup」 |
| JDK / `JAVA_HOME` 所需 JDK | ✅ 免费（Android Studio 自带 JBR；本机另有 java 26.0.1） | A，Tauri 前置页示例 `/opt/android-studio/jbr`；本地复核 |
| gradle | ✅ 免费（随 `tauri android init` 生成工程 wrapper） | B，由 https://v2.tauri.app/reference/cli/ `android init` 推导；**精确版本 NOT RETRIEVED** |
| `cargo tauri android init` / `build --apk` 侧载包 | ✅ 免费（本机自签即可） | A，https://v2.tauri.app/distribute/google-play/ `--apk`；A，https://v2.tauri.app/distribute/sign/android/ 签名仅「publish」前置 |
| adb / 真机 USB 调试 | ✅ 免费（platform-tools 内含 adb） | A⁻，https://developer.android.com/tools |
| **Google Play Console 开发者账号** | ❌ 非零现金 / 且属公开发布链路，本轮后置 | A，https://v2.tauri.app/distribute/google-play/ 「you must create a Play Console developer account」；**具体注册费金额 NOT RETRIEVED，不臆造** |
| **Apple Developer Program（$99/年）+ TestFlight** | ❌ **$99/年**，**只影响 iOS** | A，https://developer.apple.com/programs/ ：「$99 annual membership」「Invite testers and manage beta builds with TestFlight」「With TestFlight, you can invite up to 10,000 external users」；同页另有「A free Apple developer account gives you access to tools like beta versions of Xcode」「Test your apps directly on your own devices using Xcode」 |

> **§1 结论（B）**：Android 构建链的**全部软件前置均可零现金取得**（SDK cmdline-tools、NDK、四个 Rust target、JDK、gradle wrapper、adb）。唯一需要花钱的两个账号里，Play Console 属「公开发布」而非「构建/内测」，Apple $99/年**只影响 iOS**——故本轮「零额外开支」目标在 **明确暂缓 iOS/TestFlight + 不上架** 的前提下成立。

---

## §2 内核绑定路径对比

### 2.1 本地内核事实（写设计前必须锚定）

- `core/crates/mylock-core/Cargo.toml`（本地）：依赖 `argon2 = "0.6.0"`、`aes-kw 0.3.1`、`chacha20poly1305 0.11.0`、`hkdf 0.13.0`、`hmac 0.13.0`、`sha2 0.11.1`、`serde_json`、`base64`、`data-encoding`、`hex`、`zeroize`、`getrandom 0.4.3`；**无 tokio**（纯同步内核，本地事实）。
- `tests/vectors/CONTRACT.md`（本地）：G4 目标「**五端 + 恢复 CLI 逐字节一致**」；归档 `tests/vectors/test-vectors.json`（13499 B，SHA-256 `07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939`，CRLF×190），含 **30 derived_outputs + 13 negative_vectors**，G4 指纹 `d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa`；硬 KDF 约束 `argon2id(m=65536, t=3, p=4)`，`p=1`（libsodium `crypto_pwhash`）无法复现，跨端解锁将得 `E_AUTH_FAILED`；容器 `format = 1`。
- 同文件 §3 诚实状态表（本地）：端 1 Rust 内核(Windows)✅、2 恢复 CLI(Windows)✅、3 Windows 桌面层✅、4 macOS🟡CI 配置未实跑、5 Linux🟡CI 配置未实跑、**6 Android🔴未实现**、7 iOS🔴未实现。Android 激活条件：① 有实现 ② CI/设备跑同一份向量 ③ Android 上 p=4 Argon2id 可用，否则须评审、**不得静默改 p=1**。

### 2.2 三条路径

**路径 A · JNI（`jni` crate，手写桥接）**
- 事实（A，https://docs.rs/jni/latest/jni/ ，jni 0.22.4，MIT/Apache-2.0，页面日期 2026-11-14）：「This crate provides a (mostly) safe way to interface with Java and Kotlin using the Java Native Interface (JNI).」；要求 `crate-type = ["cdylib"]`、`extern "system"` 函数、`#[unsafe(no_mangle)]`、符号名如 `Java_HelloWorld_hello`、Kotlin/Java 侧 `System.loadLibrary("mylib")`；该页引用 JNI 规范 https://docs.oracle.com/en/java/javase/11/docs/specs/jni/design.html 。
- 事实（A⁻，https://developer.android.com/ndk/guides ）：「Java 代码随后可以通过 Java 原生接口 (JNI) 框架调用原生库中的函数」「Gradle …… 将原生库（SO 文件）打包到 APK 中」。

**路径 B · UniFFI（`.udl` / proc-macro，生成 Kotlin 绑定）**
- 事实（A，https://mozilla.github.io/uniffi-rs/latest/ ）：「UniFFI is a tool that automatically generates foreign-language bindings targeting Rust libraries.」「UniFFI requires you to describe your interface via either proc-macros or in an Interface Definition Language (based on WebIDL) file.」「UniFFI comes with full support for Kotlin, Swift and Python」「consolidating business logic in a single Rust library while targeting multiple platforms」。
- 事实（A，https://mozilla.github.io/uniffi-rs/0.27/tutorial/udl_file.html ）：UDL 例 `namespace math { u32 add(u32 a, u32 b); };`；「The `namespace` directive: it will be the name of your Kotlin/Swift package. It must be present in any udl file」；「It's also possible to use Rust procmacros to describe your interface and you can avoid UDL files entirely」。
- 事实（A，https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html ）：「UniFFI allows you to define your function signatures and type definitions directly in your Rust code, avoiding the need to duplicate them in a UDL file」「`#[uniffi::export]`」「call the `uniffi::setup_scaffolding!()` macro」「⚠ Warning ⚠ This facility is relatively new」「If you use both UDL and proc-macro generation, then your crate name must match the namespace in your UDL file」；该页含 Kotlin/Gradle 集成章节。

**路径 C · Tauri 2 mobile plugin / `#[tauri::command]`（Rust 侧直接暴露给 WebView）**
- 事实（A，https://v2.tauri.app/develop/calling-rust/ ）：`#[tauri::command]` 定义 Rust 函数、`.invoke_handler(tauri::generate_handler![...])`、`#[cfg_attr(mobile, tauri::mobile_entry_point)]`、前端 `invoke('my_custom_command')`；并明确 lib.rs 里的 command 不能 `pub`（E0255），单独模块须 `pub`。
- 事实（A，https://v2.tauri.app/develop/plugins/develop-mobile/ ）：「A Tauri plugin for Android is defined as a Kotlin class that extends `app.tauri.plugin.Plugin` and is annotated with `app.tauri.annotation.TauriPlugin`. Each method annotated with `app.tauri.annotation.Command` can be called by Rust or JavaScript.」「Tauri uses Kotlin by default」；`plugin android init` 引导移动端库；desktop.rs / mobile.rs 拆分、共享逻辑放 lib.rs；含「Android 16KB Memory Pages」章节。

### 2.3 对比表（B）

| 维度 | A · JNI 手写 | B · UniFFI | C · Tauri `#[tauri::command]` / mobile plugin |
|---|---|---|---|
| 与既有 Tauri 2 桌面壳复用 | 低（另起 Kotlin/JNI 层） | 中（生成 Kotlin 库，UI 仍需自接） | **高**（同一 `#[tauri::command]` + WebView 前端，desktop/mobile 共享 `lib.rs`） |
| 密码学实现份数 | 风险最高：手写桥接易顺手在 Kotlin 侧重写加解密（B） | 低（Kotlin 只调 Rust） | **低**（Rust 侧直接暴露，前端不碰字节） |
| 与恢复 CLI 共享 KAT/向量契约 | 需自建第二套调用面 | **天然**：同一 `mylock-core` crate 喂 CLI 与 Kotlin 绑定 | **天然**：同一 `mylock-core` crate 喂 CLI 与 command |
| 适配纯同步无 tokio 内核 | 需自己管线程/生命周期 | 需自己管线程/生命周期 | Tauri command 可同步返回（B，由 `#[tauri::command]` 形态推导） |
| 绑定维护成本 | 高（`extern "system"` + `no_mangle` 符号 + `System.loadLibrary`） | 中（UDL 或 proc-macro + scaffolding + Gradle 集成） | **低**（宏 + `generate_handler!`） |
| 官方对「相对新」的提示 | — | proc-macro 页自标「⚠ Warning ⚠ This facility is relatively new」（A） | — |

### 2.4 推荐与理由（B）

**推荐：以「路径 C（Tauri 2 `#[tauri::command]` / mobile plugin）」为 Android UI 主绑定路径；「路径 B（UniFFI）」作为可选补充；「路径 A（JNI 手写）」不作为主路径。**

理由（逐条对应题干约束）：

1. **复用既有壳**：项目桌面壳已是 Tauri 2（`desktop/src-tauri`），路径 C 的 `desktop.rs`/`mobile.rs` 拆分 + 共享 `lib.rs` 正是官方移动端插件结构（A，https://v2.tauri.app/develop/plugins/develop-mobile/ ），改动面最小（B）。
2. **避免重复实现密码学**：`mylock-core` 是纯 Rust、无 tokio 的同步内核（本地事实），且 `tests/vectors/CONTRACT.md` 要求 G4「五端 + 恢复 CLI 逐字节一致」。密码学必须**只有 `mylock-core` 一份实现**；路径 C 让 Rust command 直接返回/接收容器字节，Kotlin/WebView 只做「选目录、解锁、展示、锁定」，**不碰密文格式**（B）。JNI 手写桥接最容易在 Kotlin 侧顺手重写一层加解密，直接威胁 30 derived + 13 negative 的逐字节一致性（B）。
3. **共享同一套 KAT/向量契约**：恢复 CLI（`core/crates/mylock-cli`）与移动端都应链接同一个 `mylock-core` crate；UniFFI 的官方定位「consolidating business logic in a single Rust library while targeting multiple platforms」（A，https://mozilla.github.io/uniffi-rs/latest/ ）正好支持「同一内核同时喂 CLI 与 Kotlin」。若未来出现「不经过 Tauri WebView 的原生 Kotlin 界面」需求，UniFFI（proc-macro `#[uniffi::export]` + `uniffi::setup_scaffolding!()`，A，https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html ）是干净补充；但 proc-macro 官方自标「relatively new」（A，同上），故**本轮不作为主路径**（B）。
4. **JNI 不推荐**：需 `cdylib` + `extern "system"` + `#[unsafe(no_mangle)]` + `Java_*` 符号 + `System.loadLibrary`（A，https://docs.rs/jni/latest/jni/ ），是三条里手工面最大、与 Tauri 壳最不贴合的一条（B）。NDK 侧 JNI 事实仅说明「Java 可通过 JNI 调原生库」（A⁻，https://developer.android.com/ndk/guides ），**不构成必须手写 JNI 的理由**（B）。

> 架构不变量（B，由 §2.1 本地事实推导）：无论选哪条绑定路径，**密码学运算只能发生在 `mylock-core` 内**；Android 端新增代码只负责 (a) 选择 vault 目录 (b) 取得/解锁主密钥 (c) 调用 `mylock-core` 的同步 API (d) 生命周期与锁定。**任何在 Kotlin/Java 层重新实现 Argon2id / AES-KW / XChaCha20-Poly1305 的设计都应被评审否决**。

### 2.5 冻结的 A 级约束：跨端主密钥必须是可移植软件密钥

这是本项目「多端同钥」的**硬约束，A 级，不因 Android 端加入而放宽**：

- **Apple 官方（A，https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave ，ego-browser 抓完整正文）**，逐字：
  > 「**Can't encode preexisting keys. You must use the Secure Enclave to create the keys. Not having a mechanism to transfer plain-text key data into or out of the Secure Enclave is fundamental to its security.**」

  同页另载（A）：「The Secure Enclave is a hardware-based key manager that's isolated from the main processor」「Works only with NIST P-256 elliptic curve keys」「`kSecAttrAccessibleWhenUnlockedThisDeviceOnly` ... you can access the associated keychain item only on the device that created it」「`kSecAttrTokenIDSecureEnclave`」「`SecKeyCreateRandomKey`」「biometryAny flag」。

- **Android / AOSP（A，https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec ，ego-browser 精确命中）**，逐字：
  > 「**NOTE: The key material of the generated symmetric and private keys is not accessible. The key material of the public keys is accessible.**」

  旁证（A，https://source.android.com/docs/security/features/keystore/features ，最后更新 2026-10-29）：「Creation of keys, resulting in private or secret key material that is accessible only to the secure environment」「**Hardware-based keys that can never be extracted from the device**」「KeyMint supports import of: Asymmetric key pairs in DER-encoded PKCS#8 ... Symmetric keys as raw bytes」「Tag::USER_SECURE_ID ... Tag::AUTH_TIMEOUT」。另（A，https://source.android.com/docs/security/features/keystore ，*Hardware-backed Keystore*，最后更新 2026-09-30）：「keyblobs ... contain the secret key material, encrypted so Keystore can store them but not use or reveal them」「KeyMint trusted app (TA) ... This app has access to the raw key material, and validates all of the access control conditions on keys before allowing their use.」「keys that can be used only if the user has authenticated (authentication bound keys)」「Android 12 also includes a new version of the keystore system daemon, rewritten in Rust and known as keystore2」。

- **推导（B，但由上述 A 级引文强制）**：既然 Secure Enclave「无法把明文密钥数据转入/转出」、Android Keystore 生成的对称/私钥「密钥材料不可访问」且「永不可从设备提取」，那么**硬件密钥天生是「本机密钥」**。MyLock 的 vault 必须能在 Windows 桌面、恢复 CLI、Android、（未来）macOS/Linux/iOS 上解同一份密文（`format = 1`，30 derived + 13 negative 逐字节一致，本地事实），因此：
  - **跨端主密钥必须是「可移植软件密钥」**（由口令经 `argon2id(m=65536,t=3,p=4)` 派生，或由该口令派生的密钥再 wrap 的密钥层级），它必须能被导出/重建/在多端复用；
  - **硬件密钥（Secure Enclave / Android Keystore / StrongBox）只能在本机 wrap 这份软件主密钥**，即「设备本地便利解锁层」，**绝不能成为 vault 的主密钥本身**；
  - **口令兜底必做**：换机、重装、多端首次加入、生物识别不可用/被撤销时，必须能用口令重新派生出同一把软件主密钥。否则一旦设备丢失或 Keystore 密钥因生物特征变更失效，vault 永久不可解——这与「私人数据保险柜 + 独立恢复 CLI」的产品定位直接冲突（B）。

---

## §3 「生物暗格」最小形态

### 3.1 正确形态：BiometricPrompt → Keystore 密钥 → wrap 可移植软件主密钥

**分级事实链**：

1. **生物识别是「二级认证」，不是密钥**（A，https://source.android.com/docs/security/features/biometric ，最后更新 2026-09-30）：
   > 「Biometrics offer a more convenient, but potentially less secure way of confirming your identity with a device. Under the tiered authentication model, primary authentication (that as, knowledge-factor based modalities such as PIN, pattern, and password) provides the highest level of security. Biometrics are in the secondary tier of authentication」

   同页（A）：「**Make sure that raw biometric data or derivatives (such as templates) are never accessible from outside the secure isolated environment (such as the TEE or Secure Element). All stored data must be encrypted with a device-specific key known only to the Trusted Execution Environment (TEE).**」「Biometric acquisition, enrollment, and recognition must occur inside the secure isolated environment」。

2. **生物特征绝不能当密钥**（B，由上句 + §2.5 推导）：生物特征不出 TEE、不可导出、不可撤销、不可跨设备复现，既不满足「可移植」，也不满足「逐字节一致」的跨端解密需求；把它当密钥等于把 vault 永久锁死在单台设备的传感器上。

3. **BiometricPrompt 只能解锁一个 Keystore 密钥**（A⁻，https://developer.android.com/identity/sign-in/biometric-auth ，最后更新 2026-10-12，中文机翻）：
   > 「您可以使用将加密技术整合到生物识别验证工作流程中，使用 CryptoObject 的实例。该框架支持以下加密对象：Signature、Cipher、Mac 和 KeyAgreement。」
   > 「用户使用生物识别提示成功进行身份验证后，应用即可执行加密操作。例如，如果您使用 Cipher 对象进行身份验证，应用便可以使用 SecretKey 对象执行加密和解密。」
   > 「生成使用以下 KeyGenParameterSpec 配置的密钥」「每次使用时进行身份验证密钥」「允许回退到非生物识别凭据」

   同页（A⁻）：`BiometricManager.Authenticators` 的 `BIOMETRIC_STRONG`（Class 3）/ `BIOMETRIC_WEAK`（Class 2）/ `DEVICE_CREDENTIAL`；`setAllowedAuthenticators()`、`canAuthenticate()`、`ACTION_BIOMETRIC_ENROLL`；「您不能在……同时调用 setNegativeButtonText() 和 setAllowedAuthenticators(... or DEVICE_CREDENTIAL)」；Android 10（API 29）及以下不支持 `DEVICE_CREDENTIAL` 与 `BIOMETRIC_STRONG` 组合，改用 `KeyguardManager.isDeviceSecure()`。AOSP 表格（A，https://source.android.com/docs/security/features/biometric ）进一步给出：`BIOMETRIC_STRONG`(Class 3) → Keystore(time-based / operation-based key) **Yes**；`BIOMETRIC_WEAK`(Class 2) → **No / No**；`DEVICE_CREDENTIAL` → Yes / Yes。⇒ **最小可用应选 `BIOMETRIC_STRONG`，或 `BIOMETRIC_STRONG | DEVICE_CREDENTIAL` 作回退**（B，由 A 表格推导）。

4. **被解锁的 Keystore 密钥去 wrap 软件主密钥**（A⁻，https://developer.android.com/training/articles/keystore ，中文机翻）：
   > 「在密钥进入密钥库后，可以将它们用于加密操作，而密钥材料仍不可导出」
   > 「密钥材料永不进入应用进程」
   > 「为密钥启用此功能时，其密钥材料永远不会暴露于安全硬件之外」
   > 「要求进行用户身份验证才能使用密钥」
   > 「借助 Android 9（API 级别 28）及更高版本，您能够利用 ASN.1 编码密钥格式将已加密密钥安全导入密钥库……生成目的为 **PURPOSE_WRAP_KEY** 的密钥对」「setIsStrongBoxBacked()」

**正确数据流（B，由上述 A/A⁻ 事实编排）**：

```
口令 (用户记忆, 可移植)
   │ argon2id(m=65536, t=3, p=4)  ← mylock-core, argon2 crate, 显式 parallelism=4
   ▼
可移植软件主密钥 MK  ──────────────► 解 vault 容器 (format=1, 30/13 向量逐字节一致)
   ▲
   │ AES-KW / wrap（本机一次性）
   │
Android Keystore 密钥 K_dev
   • KeyGenParameterSpec: setUserAuthenticationRequired(true)
   • PURPOSE_WRAP_KEY 或 Cipher + CryptoObject
   • BIOMETRIC_STRONG（+ DEVICE_CREDENTIAL 回退）
   • K_dev 永不出安全硬件；仅「解锁后才允许用」
   ▲
   │ BiometricPrompt.authenticate(promptInfo, CryptoObject(Cipher))
   ▼
用户的指纹/面容（只在 TEE 内比对，永不出 TEE，绝不作为密钥）
```

> 关键点（B）：**BiometricPrompt 成功 ≠ 拿到生物特征，而是「拿到 K_dev 的使用权」**；用 K_dev 解出 MK 后，`mylock-core` 才用 MK 解容器。K_dev 是「本机便利层」，MK 才是「跨端主密钥」——与 §2.5 冻结约束一致。设备丢失 / K_dev 失效时，回到口令路径重新派生 MK，vault 仍可解（口令兜底，B）。

### 3.2 严禁事项（清单，B 级设计约束，依据为上述 A/A⁻）

1. **严禁把生物特征（模板 / 特征值 / 哈希）当作或派生为 vault 密钥**——生物特征不可导出、不可跨端、不可撤销（A，AOSP biometric 页 + Apple Secure Enclave 页）。
2. **严禁把 Android Keystore / StrongBox 生成的密钥当作跨端主密钥**——「The key material of the generated symmetric and private keys is not accessible」（A，KeyGenParameterSpec）、「Hardware-based keys that can never be extracted from the device」（A，keystore/features）。
3. **严禁在 Kotlin/Java 层重新实现密码学**（见 §2.4），包括不自行拼 Argon2 / AES-KW / ChaCha20-Poly1305。
4. **严禁静默把 `p=4` 改成 `p=1`**（本地，`tests/vectors/CONTRACT.md`）：Android 上若 `argon2` crate 的 `parallelism=4` 不可用，必须走评审，**不得**回退到 libsodium `crypto_pwhash` 的 `p=1`，否则跨端解锁得 `E_AUTH_FAILED`。
5. **不把 Keystore 密钥 / 生物特征 / 明文 MK 写入任何日志、遥测、崩溃上报**（B，与 §3.5「不做遥测」一致）。

### 3.3 与桌面同钥（必须逐字节一致）

| 要素 | 冻结值 / 做法 | 来源 / 分级 |
|---|---|---|
| vault 密文格式 | **同一份 `format = 1` 容器**；Android 不引入新容器版本、不改字节布局 | 本地，`tests/vectors/CONTRACT.md`（B 推导：否则 G4 不成立） |
| KDF | **`argon2id(m=65536, t=3, p=4)`** | 本地，`tests/vectors/CONTRACT.md`；`core/crates/mylock-core/Cargo.toml` 有 `argon2 = "0.6.0"` |
| 实现方式 | **用 `argon2` crate 显式 `parallelism=4`**；**不可用 libsodium `crypto_pwhash`（其 `p=1`）** | 本地契约 + B（题干约束）；`p=1` 无法复现 30 derived，跨端得 `E_AUTH_FAILED`（本地契约明文） |
| 向量契约 | 同一份 `tests/vectors/test-vectors.json`（SHA-256 `07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939`，G4 指纹 `d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa`） | 本地，`tests/vectors/CONTRACT.md` |
| 密码学实现份数 | **仅 `mylock-core` 一份**；恢复 CLI 与 Android 共用同一 crate | B（§2.4） |

> 同钥判据（B）：桌面端用 MK 解出的明文字节，与 Android 端用「口令重新派生 / K_dev 解 wrap 得到」的同一 MK 解出的明文字节，**必须逐字节相同**；这正是 G4 的 30 derived + 13 negative 要守的东西（本地契约）。

### 3.4 最小可用范围（MVP，B）

严格限定为四步闭环，**不扩权**：

1. **选 vault 目录**：通过 Android SAF（系统文件选择器）选定一个已由桌面端创建 / 或待创建的 vault 目录（B；具体 API 本轮未抓官方页 → 不引 URL，标 B 设计意图）。
2. **解锁**：`BiometricPrompt`（`BIOMETRIC_STRONG` + `DEVICE_CREDENTIAL` 回退）解锁 `K_dev` → 解 wrap 得 MK；生物不可用时走**口令兜底**直接 `argon2id(m=65536,t=3,p=4)` 派生 MK（B，§2.5 强制）。
3. **浏览 / 导入文件**：调用 `mylock-core` 同步 API，列出目录条目、导入本地文件到 vault（加密写入 `format = 1` 容器）；**不解密到磁盘明文、不做缩略图缓存外泄**（B）。
4. **锁定**：清掉进程内 MK / 解密句柄，`K_dev` 保持「需认证」，UI 回到解锁前态（B）。

> 本轮 **不实现**上述任何一步的代码，仅定义形态与边界（§0.2）。

### 3.5 明确不做的事（B，题干硬性范围）

| 不做 | 理由 |
|---|---|
| **不做云同步** | 本地优先、零知识；引入同步即引入账号/服务端/密钥托管，超出「小范围试用」且直接冲击「零知识」定位 |
| **不做账号** | 无服务端、无身份体系；口令 + 可移植软件密钥已足够多端同钥 |
| **不做遥测** | 私人数据保险柜；不采集任何使用数据、崩溃日志、生物/密钥材料 |
| **不做 iOS / TestFlight** | Apple Developer Program **$99/年**（A，https://developer.apple.com/programs/ ）违反本轮「零额外开支」；且 Secure Enclave 仅本机 wrap 的 A 级约束已冻结，iOS 需单独一轮 |
| 不做公开发布 / 上架 | 需 Play Console 账号 + 上传签名（A，§0.3），属 NO-GO 边界 |
| 不产出 APK | 本机无 SDK/NDK/gradle/adb（§5.1），且题干明示「不要求产出 APK」 |

---

## §4 零成本前置清单

> 全部条目**零现金**。分两类：**§4.1 本机能做**（当前环境即可执行，无需新增许可）；**§4.2 需要用户自行安装**（免费，但需人工、磁盘、时间、接受许可条款）。**本轮未实测任何下载/安装动作**（诚实边界，§5.3）。

### 4.1 本机能做（当前即可，零现金）

1. **交付设计与评估文档**（已完成，即本文件）。
2. **环境盘点**（已在 §5.1 复核执行）：
   ```bat
   echo %ANDROID_HOME% & echo %ANDROID_SDK_ROOT% & echo %ANDROID_NDK_HOME% & echo %JAVA_HOME% & echo %NDK_HOME%
   java -version
   rustup target list --installed
   ```
   注意（本地复核）：当前 pwsh 会话 PATH 下 `rustup` / `cargo` **未被识别**（用户授权材料记录已装 cargo 1.100.0、rustup 已有 `x86_64-pc-windows-gnu`）⇒ 后续 rust 命令须在**已正确配置 Rust PATH 的终端**执行（B）。
3. **添加四个 Rust target**（免费、零现金下载；A，https://v2.tauri.app/start/prerequisites/ ）：
   ```bat
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```
4. **下载免费 ZIP 到本机磁盘**（仅下载，不安装）：Android SDK Command-line Tools 与独立 NDK ZIP，URL 见 §4.2；**本轮未实测下载**（诚实边界）。
5. **设置环境变量**（Windows「系统属性 → 环境变量」，非 bash 语法）：`JAVA_HOME`、`ANDROID_HOME`、`NDK_HOME`、`PATH` 追加 cmdline-tools/bin 与 platform-tools（B，由 §1.5 A/A⁻ 事实改写为 Windows 形式）。

### 4.2 需要用户自行安装（免费，但需人/磁盘/时间）

**精确下载 URL（A⁻，https://developer.android.com/studio#command-line-tools ，ego-browser 读取 `a[href]` 得到精确 href）**：

- Android SDK Command-line Tools（Windows，仅命令行，无需 Android Studio）：
  `https://dl.google.com/android/repository/commandlinetools-win-15859902_latest.zip`
  （包 `commandlinetools-win-15859902_latest.zip`，155.7 MB，SHA-256 `90ae805d20434428bffcb699c290860f19bb5f66a67e6b330067e3de801fb04a`；官方原话「Android Studio 中包含命令行工具。如果您不需要 Android Studio，可以从上面下载基本的 Android 命令行工具。您可以使用随附的 sdkmanager 来下载其他 SDK 软件包。」）
- Android Studio（Windows，可选；含 SDK Manager 与自带 JBR）：
  `https://edgedl.me.gvt1.com/android/studio/install/2026.2.2.9/android-studio-rabbit2-patch1-windows.exe`（1.5 GB，SHA-256 `00e98c1c...43b`，同页）
- NDK（推荐走 SDK Manager 的「NDK (Side by side)」，A，https://v2.tauri.app/start/prerequisites/ ；独立 ZIP 见 A⁻，https://developer.android.com/ndk/downloads ：「最新 LTS 版本 (r30)」`android { ndkVersion "30.0.16248370" }`，Windows 包 `android-ndk-r30-windows.zip`，727980788 字节，SHA1 `9bf167a1985fa7d4a036186b78f702eab9179408`；下载 URL 形态 `https://dl.google.com/android/repository/android-ndk-<version>-<platform>.zip`，同页给出 macOS 形态 `https://dl.google.com/android/repository/android-ndk-<version>-darwin.zip`）

**安装命令序列（B，由 A/A⁻ 事实编排；Windows 路径写法为适配改写）**：

```bat
:: 1) 解压 cmdline-tools 到 %ANDROID_HOME%\cmdline-tools\latest\（官方：android_sdk/cmdline-tools/version/bin/，A⁻ developer.android.com/tools）
:: 2) 用 sdkmanager 安装 SDK 组件（组件名来自 A: Tauri prerequisites）
sdkmanager --licenses
sdkmanager "platform-tools" "platforms;android-<API>" "build-tools;<version>" "ndk;30.0.16248370" "cmake;<version>" "lldb;<version>"

:: 3) 设置环境变量（Windows）
::    JAVA_HOME   = <Android Studio>\jbr   或已有 JDK 根目录
::    ANDROID_HOME= <解压根>\Sdk
::    NDK_HOME    = %ANDROID_HOME%\ndk\30.0.16248370
::    PATH       += %ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools

:: 4) Rust 侧（在已配置 rustup 的终端）
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

:: 5) 生成 Android 工程并出侧载包
cargo tauri android init
cargo tauri android build --apk --target aarch64
```

> 说明（B / 诚实）：`platforms;android-<API>` 与 `build-tools;<version>`、`cmake;<version>`、`lldb;<version>` 的**精确版本号官方前置页未给**，实施时以 `sdkmanager --list` 输出为准；**本报告不臆造这些数字**（见 §1.2）。`--licenses` 用于接受 SDK 许可条款（A⁻，https://developer.android.com/tools 列出 sdkmanager 可「查看、安装、更新和卸载」包）。

### 4.3 安装完成后如何验证（命令序列）

| 验证项 | 命令 | 期望（B，未来态，非当前态） |
|---|---|---|
| Rust target 齐备 | `rustup target list --installed` | 含 `aarch64-linux-android`、`armv7-linux-androideabi`、`i686-linux-android`、`x86_64-linux-android`（A，https://v2.tauri.app/start/prerequisites/ ） |
| JDK 可用 | `java -version` | 本机已为 `java version "26.0.1" 2026-04-21` / `Java(TM) SE Runtime Environment (build 26.0.1+8-34)`（本地复核，当前即通过）；`JAVA_HOME` 须已设置 |
| 环境变量 | `echo %ANDROID_HOME%` / `%NDK_HOME%` / `%JAVA_HOME%` | 三者非空且指向真实目录（当前全为空，§5.1） |
| SDK 包齐备 | `sdkmanager --list_installed` | 含 Platform / Platform-Tools / Build-Tools / NDK(Side by side) / CMake / LLDB（A/A⁻，§1.1） |
| 设备连通 | `adb devices` | 列出真机或模拟器（adb 属 platform-tools，A⁻，https://developer.android.com/tools「还可以用它在设备上安装 APK」） |
| 端到端构建 | `cargo tauri android build --apk` | 生成可侧载 APK（A，https://v2.tauri.app/distribute/google-play/ `--apk`）；`--aab` 留给上架（同页） |

> **当前状态（§5.1 复核）**：上表除 `java -version` 外**均未通过**——四个 Android target 未装、五个环境变量全 unset、无 adb、无 SDK 目录、无 `.git`。因此 §4.3 是「安装完成后的验收脚本」，**不是已完成的证明**。

### 4.4 明确暂缓（非零现金 / 非本轮）

| 暂缓项 | 事实 | 分级 |
|---|---|---|
| **iOS / TestFlight** | 「Join the Apple Developer Program」「**$99 annual membership**」「Invite testers and manage beta builds with TestFlight」「With TestFlight, you can invite up to 10,000 external users」；另有「A free Apple developer account gives you access to tools like beta versions of Xcode」「Test your apps directly on your own devices using Xcode」 | A，https://developer.apple.com/programs/ |
| 同上（iOS 构建前置） | 「**macOS Only**」「iOS development requires Xcode and is only available on macOS」「`rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim`」 | A，https://v2.tauri.app/start/prerequisites/ |
| Play Console 上架 | 「To distribute Android apps in the Play Store you must create a Play Console developer account.」；上传签名见 §0.3 | A，https://v2.tauri.app/distribute/google-play/ 、https://v2.tauri.app/distribute/sign/android/ ；**具体注册费金额 NOT RETRIEVED** |

> 结论（B）：**$99/年只影响 iOS/TestFlight，不影响 Android 构建与小范围侧载试用**；本轮在「零额外开支」下明确暂缓 iOS，故 §1.6 的「全链免费」判断成立。

---

## §5 验收判据与诚实边界

### 5.1 本机现实（用户给定 + pwsh 复核，如实写入）

- 无 Android SDK / NDK / gradle / adb / sdkmanager / emulator（用户授权材料）。
- `ANDROID_HOME` / `ANDROID_SDK_ROOT` / `ANDROID_NDK_HOME` / `JAVA_HOME` / `NDK_HOME` **全部 unset**（pwsh `echo` 复核为空）。
- 常见 SDK 目录 `C:\Users\zjc_9\AppData\Local\Android\Sdk`、`C:\Android\Sdk`、`C:\Program Files\Android\Android Studio` **全部 `Test-Path = False`**（pwsh 复核）。
- `rustup target list --installed` 仅 `x86_64-pc-windows-gnu`，**无任何 Android rust target**；cargo 1.100.0（用户授权材料）。
- `java -version` = `java version "26.0.1" 2026-04-21`、`Java(TM) SE Runtime Environment (build 26.0.1+8-34)`（pwsh 复核，Java 存在）。
- 当前 pwsh 会话 PATH 下 `rustup` / `cargo` **报「not recognized」**（与「已装 cargo 1.100.0」并存 → 属 PATH 配置问题，非未安装；B）。
- **无 `.git` 仓库**（用户授权材料）。
- ⇒ **本轮不产出可侧载 APK，只交付设计 + 零成本前置清单**（§0.2）。

### 5.2 G4 验收判据（Android 加入后必须跑）

**G4 = 「五端 + 恢复 CLI 逐字节一致」**（本地，`tests/vectors/CONTRACT.md`）。**目前只完成 Rust/Windows 单端**：同文件 §3 状态表为 —— 端 1 Rust 内核(Windows) ✅、端 2 恢复 CLI(Windows) ✅、端 3 Windows 桌面层 ✅、端 4 macOS 🟡（CI 配置未实跑）、端 5 Linux 🟡（CI 配置未实跑）、**端 6 Android 🔴 未实现**、端 7 iOS 🔴 未实现；§6 自述「不是第三方安全审计/安全认证；本机只实测 Windows 路径」。

**Android 端激活的硬判据（本地契约 + B 编排）**：

1. 在 Android 真机 / CI 上跑**同一份** `tests/vectors/test-vectors.json`（SHA-256 `07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939`）；
2. **30 derived_outputs 逐字节全中**（30/30）；
3. **13 negative_vectors 全部按预期拒绝**；
4. **G4 指纹命中** `d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa`；
5. **Android 上 `p=4` Argon2id 必须可用**（`argon2` crate 显式 `parallelism=4`）；若不可用须评审，**不得静默改 `p=1`**（本地契约明文：`p=1` 无法复现、跨端解锁得 `E_AUTH_FAILED`）；
6. 生物暗格路径（K_dev wrap MK）与口令兜底路径解出的 MK **必须相同**，且解出的容器明文与桌面端逐字节一致（B，§3.3）。

> 未完成 1–6 之前，Android 端在图 / 状态表里保持 🔴「未实现」，**不得因「设计已写」而改标为已实现**（B，诚实边界）。

### 5.3 本报告不是什么

- **不是已实现**：未写任何 Rust / Kotlin / Tauri 代码，未执行 `cargo tauri android init`，未生成工程，未产出 APK/AAB（§0.2、§5.1）。
- **不是产品发布**：无签名、无 Play Console、无侧载包；公开发布链路整体后置（§0.3）。
- **不是第三方审计 / 安全认证**：本地契约 §6 自述「不是第三方安全审计/安全认证；本机只实测 Windows 路径」；本报告亦不含任何渗透测试、形式化验证、密码学评审结论（B）。
- **不是法律意见**：§5.4 仅转述官方 FAQ / 官方页面文字（A），不构成合规意见或法律结论。
- **未实测下载/安装**：§4 的所有下载 URL 与命令序列来自官方页（A/A⁻），但**本轮未在本机执行下载、解压、sdkmanager、gradle、adb 或构建**（诚实边界）。精确 SDK Platform API level / gradle 版本 / Play Console 注册费金额均未由官方页取得 → 见附录 A，不臆造。

### 5.4 合规事实（2026-11-15 抓取时点值，仅事实转述，不写法律结论）

**MPL 2.0 源码可得义务**（A，https://www.mozilla.org/en-US/MPL/2.0/FAQ/ ）：

> Q5：「The MPL only creates obligations for you if you want to distribute the software outside your organization.」
> Q10：「You must make available the MPL-licensed portions of the source code... and inform the recipients how they can obtain such source code (Section 3.2).」
> Q8：「You must inform the recipients where they can get the source for the MPLed code (Section 3.2).」
> Q17：「distribute 指交付副本，不含 make available over the web without delivery of a copy」。
> 该页自述 FAQ「is not the license... not a substitute for... seeking legal advice」。

**EAR 出口分类**（A，https://www.bis.gov/regulations/ear ）：

> 「This tool simplifies access to the EAR, 15 CFR Parts 730-774」
> 「Export Control Classification Numbers (ECCNs) within the ... Commerce Control List, Supplement No. 1 to Part 774」
> 「Sourced directly from the eCFR」
> 「This resource is not an official legal edition of the CFR... if you are relying on it for legal research, please consult the official editions」
> 同页列有 `/licensing/classify-your-item`、`/learn-support/encryption-controls` 链接。

> 以上为**官方页面文字的事实转述**；具体分类、许可、义务判定**不在本报告结论范围内**，且该 BIS 页面自述「not an official legal edition」（A）。

### 5.5 推迟 ≠ 豁免（B，基于 §5.4 A 级事实的提醒）

- 「本轮不上架、内测规模小、软件免费、零额外开支」**都不等于**出口分类与源码可得义务消失：EAR 页面把 ECCN 放在「Commerce Control List, Supplement No. 1 to Part 774」并指向 `classify-your-item` / `encryption-controls`（A，https://www.bis.gov/regulations/ear ）；MPL FAQ Q10/Q8 把「make available the MPL-licensed portions of the source code」与「inform the recipients」写成分发时的义务（A，https://www.mozilla.org/en-US/MPL/2.0/FAQ/ ）。
- 同时又须如实记录 FAQ Q5 的边界：「The MPL only creates obligations for you if you want to distribute the software outside your organization.」（A，同页）——**组织内 / 未分发**与**对外分发**是不同状态，本轮属前者，但「暂缓对外分发」是**推迟动作**，不是**义务消灭**。
- 结论（B）：**推迟 ≠ 豁免**。一旦未来执行公开发布、对外交付副本、或涉及加密物品出口，上述事项须在发布前单独复核；合规事实以 **2026-11-15 抓取时点值**为准，届时须重新核对官方页面（A 页面均带「Last updated / not an official legal edition」提示）。

---

## 附录 A · 资料抓取记录与 NOT RETRIEVED 清单

### A.1 web_fetch 直接成功（A）

`https://v2.tauri.app/distribute/sign/android/` · `https://v2.tauri.app/distribute/google-play/` · `https://v2.tauri.app/reference/cli/` · `https://v2.tauri.app/develop/calling-rust/` · `https://v2.tauri.app/develop/plugins/develop-mobile/` · `https://mozilla.github.io/uniffi-rs/latest/` · `https://mozilla.github.io/uniffi-rs/0.27/tutorial/udl_file.html` · `https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html` · `https://docs.rs/jni/latest/jni/` · `https://www.mozilla.org/en-US/MPL/2.0/FAQ/` · `https://www.bis.gov/regulations/ear`

### A.2 web_fetch 失败 → ego-browser 实抓成功（按 A / A⁻ 计，非 NOT RETRIEVED）

| URL | web_fetch 结果 | 补抓方式 | 最终分级 |
|---|---|---|---|
| `https://v2.tauri.app/start/prerequisites/` | 未直抓 | ego-browser 全文 | A |
| `https://v2.tauri.app/reference/cli/`（android 子命令详情） | 直抓仅命令表 | ego-browser 全文 | A |
| `https://v2.tauri.app/reference/environment-variables/` | 未直抓 | ego-browser | A |
| `https://developer.android.com/ndk/guides` | `TypeError: fetch failed` | ego-browser | A⁻（中文机翻） |
| `https://developer.android.com/ndk/downloads` | `TypeError: fetch failed` | ego-browser | A⁻（中文机翻） |
| `https://developer.android.com/studio#command-line-tools` | `TypeError: fetch failed` | ego-browser（含 `a[href]` 精确下载 URL） | A⁻ / A |
| `https://developer.android.com/tools` | `TypeError: fetch failed` | ego-browser | A⁻ |
| `https://developer.android.com/identity/sign-in/biometric-auth` | `TypeError: fetch failed` | ego-browser | A⁻ |
| `https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec` | `TypeError: fetch failed` | ego-browser（Bing 精确短语定位后打开） | A |
| `https://developer.android.com/training/articles/keystore` | `TypeError: fetch failed` | ego-browser | A⁻ |
| `https://source.android.com/docs/security/features/biometric` | `TypeError: fetch failed` | ego-browser | A |
| `https://source.android.com/docs/security/features/keystore` | 首次超时 | ego-browser `waitUntil:'commit'` 重试 | A |
| `https://source.android.com/docs/security/features/keystore/features` | `TypeError: fetch failed` | ego-browser（英文版） | A |
| `https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave` | 仅返回标题 | ego-browser 完整正文 + JSON tutorial 页 | A |
| `https://developer.apple.com/programs/` | 首次未抓 | ego-browser | A |

### A.3 NOT RETRIEVED（明确标注，绝不臆造）

| 条目 | 结果 | 处置 |
|---|---|---|
| `https://v2.tauri.app/distribute/mobile/android/` | **HTTP 404**（该 URL 不存在） | 未引用其任何内容；Android 前置改由 `https://v2.tauri.app/start/prerequisites/` 覆盖（A） |
| `https://raw.githubusercontent.com/.../prerequisites.mdx` 及 `https://github.com/*` | `TypeError: fetch failed`（Node 端 fetch 对 github/raw 持续失败） | **未引用 GitHub 原始 mdx 的任何事实**；Tauri 相关内容全部取自 `v2.tauri.app` 官方页（A） |
| `https://mozilla.github.io/uniffi-rs/` 与 `.../tutorial/udl_file.html`（首次） | 仅返回重定向壳 | 改抓 `.../latest/` 与 `.../0.27/tutorial/udl_file.html` 成功（A），不据重定向壳断言 |
| **Tauri 官方前置页对「精确 SDK Platform API level / NDK 绑定版本 / gradle 版本」** | 官方页**未给出**精确数值（非抓取失败，是「官方未写」） | §1.2 标「官方未给精确值」，实施以 `sdkmanager --list` / 生成工程 `gradle-wrapper.properties` 为准，不编造数字 |
| **Google Play Console 开发者账号具体注册费金额** | 未抓取 / 未断言 | §1.6、§4.4 只写「需创建账号」（A），**不写具体金额** |
| **Android SAF 具体 API 官方文档页** | 未抓取 | §3.4 仅标 B 设计意图，不附 URL、不引官方条文 |

### A.4 本地文件（非外部 URL，标「本地」）

`tests/vectors/CONTRACT.md`（G4 契约、30+13、指纹、Argon2id 参数、§3 状态表、§6 声明）· `tests/vectors/test-vectors.json`（归档 SHA-256）· `core/crates/mylock-core/Cargo.toml`（依赖清单、无 tokio）· workspace 内 5 个 Cargo.toml（`mylock-cli` / `mylock-mount` / `mylock-core` / `desktop/src-tauri` / 根）。

---

## 附录 B · 一手 URL 索引（按分级）

**A（一手官方页 + 精确引文）**

- https://v2.tauri.app/start/prerequisites/
- https://v2.tauri.app/reference/cli/
- https://v2.tauri.app/reference/environment-variables/
- https://v2.tauri.app/develop/calling-rust/
- https://v2.tauri.app/develop/plugins/develop-mobile/
- https://v2.tauri.app/distribute/sign/android/
- https://v2.tauri.app/distribute/google-play/
- https://mozilla.github.io/uniffi-rs/latest/
- https://mozilla.github.io/uniffi-rs/0.27/tutorial/udl_file.html
- https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html
- https://docs.rs/jni/latest/jni/
- https://docs.oracle.com/en/java/javase/11/docs/specs/jni/design.html （jni crate 页引用的 JNI 规范）
- https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec
- https://source.android.com/docs/security/features/biometric
- https://source.android.com/docs/security/features/keystore
- https://source.android.com/docs/security/features/keystore/features
- https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave
- https://developer.apple.com/programs/
- https://www.mozilla.org/en-US/MPL/2.0/FAQ/
- https://www.bis.gov/regulations/ear

**A⁻（官方中文机翻 / 忠实转载）**

- https://developer.android.com/ndk/guides
- https://developer.android.com/ndk/downloads
- https://developer.android.com/studio#command-line-tools
- https://developer.android.com/tools
- https://developer.android.com/identity/sign-in/biometric-auth
- https://developer.android.com/training/articles/keystore

**B（工程判断 / 推荐 / 编排）**：§0.3 边界推导、§1.2 结论、§1.4 流程编排、§1.6 结论、§2.3 对比表、§2.4 推荐与理由、§2.5 推导、§3.1 数据流、§3.2 严禁清单、§3.4/§3.5 MVP 与不做清单、§4 命令序列与验证期望、§5.2 判据编排、§5.5 推迟≠豁免。

**C**：本文未使用（凡推测处均降级为 B 并给出 A/A⁻ 依据，或显式标 NOT RETRIEVED）。
