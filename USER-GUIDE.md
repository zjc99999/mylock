# MyLock 封闭内测版 · 软件功能与操作说明（USER-GUIDE）

> **版本**：v1.1 · **日期**：2026-11-15 · **适用**：`MyLock-Beta-0.1.0-portable-unsigned.zip`（8 文件便携包，含本文）
> **v1.1 变更**：GUI 新增「新建保险柜」按钮（原 CLI-only 的 Step 1 已收进界面）、使用前必读提示、Recovery Key 醒目展示、人话错误预检；密文容器新增「提示性」防误删保护（隐藏+系统属性 / 删除警告 / 保险柜图标）。详见 §3 / §4 Step 1 / §10。
> **性质**：邀请制封闭内测版，**未经第三方安全审计、未代码签名**；≠ 产品发布 ≠ 五端验收 ≠ 第三方审计。
> **使用前请先读 [BETA-NOTICE.md](BETA-NOTICE.md)：请勿存放不可替代的唯一副本。**
> **本文所有命令、输出、错误串、限制均于 2026-11-15 用便携包内真实程序在本机实测得到，不是设想或计划。** 实测脚本见 [tools/probe-userguide.ps1](tools/probe-userguide.ps1)、[tools/probe-folder-workflow.ps1](tools/probe-folder-workflow.ps1)、[tools/probe-nested-folder.ps1](tools/probe-nested-folder.ps1) 与 [tools/probe-guide-numbers.ps1](tools/probe-guide-numbers.ps1)（均在 `%TEMP%` 模拟，绝不碰你的真实 `E:\testLock`）。

---

## 0. 先直接回答你最关心的那个问题

> **「我在 E 盘有个文件夹 `E:\testLock`，里面有些文档，我可否将此文件夹加密、解密？」**

**能，但不是「原地加密」，也不是点一下「加密 / 解密」按钮。** 准确说法是：

| 你以为的 | 实际做的 |
|---|---|
| 把 `E:\testLock` 原地变成加密文件夹 | ❌ 不会。MyLock 不改动你原来的文件夹 |
| 有个「加密」按钮 | ❌ 没有。写入解锁后的盘符时**自动**加密落盘 |
| 有个「解密」按钮 | ❌ 没有。「解锁」= 临时变回可见明文；「锁定」= 又变回密文 |

MyLock 的真实模型是 **「密文保险柜目录（vault）+ 解锁后映射成一个盘符」**：

1. 你先**新建一个空目录**当保险柜，例如 `E:\testLockVault`（⚠️ 必须是不存在的全新路径）。
2. 用口令**解锁**它，程序把它映射成一个盘符（默认 `Z:`）。
3. 此时 `Z:` 看起来就是个**普通文件夹**：把你 `E:\testLock` 里的东西（含子文件夹）整个拖进去、双击、编辑、另存为都行；写进去的内容会**自动加密**后落到 `E:\testLockVault`。
4. 点**锁定**，`Z:` 消失；`E:\testLockVault` 里只剩一堆看不懂的密文文件。**拷贝整个目录到 U 盘 / 移动硬盘，仍是一堆密文，不会自动变明文。**
5. 原来的 `E:\testLock` **始终是明文**，MyLock **不会替你删除**。要真正「只留密文」，得你自己把原明文安全删除。

> ### ⚠️ 最重要的一条警告（实测）
> **千万不要把建库命令指向 `E:\testLock` 本身。** `mylock.exe create <目录>` 若发现目标目录已存在，会**先清空该目录**再写入容器（内核 `write_vault` 里是 `remove_dir_all(root)`）。实测：对一个含 `原明文.txt` 的已存在目录执行 `create` 后，`原明文.txt` **没有了**（`ORIGINAL_STILL_EXISTS=False`）。
> **所以：建库永远用一个全新的空路径；把原文件夹「装进去」而不是「变成它」。**

---

## 1. 这个包里有什么

解压后**整个文件夹一起拷贝**（不要只拎单个 exe）。文件清单：

| 文件 | 用途 | 你会不会直接点它 |
|---|---|---|
| `mylock-desktop.exe` | **图形界面（GUI）**：**新建保险柜** / 解锁 / 锁定 / 结构校验 / 会话状态 / 端口探测 | ✅ 日常主用（含建库） |
| `WebView2Loader.dll` | GUI 的动态加载器，**必须与 exe 同目录** | 不点，但不能删 |
| `mylock.exe` | **独立恢复 CLI**：建库、导出、改口令、真吊销等 9 个命令 | ✅ 备用建库 / 排障 / 恢复用 |
| `mylock-serve.exe` | 独立 loopback WebDAV 服务（备用 / 排障） | 一般不用（GUI 已是进程内引擎） |
| `BETA-NOTICE.md` | 封闭内测须知与风险声明 | 先读 |
| `README-BETA.md` | 包内速览 | 先读 |
| `THIRD_PARTY_NOTICES.md` | 613 个第三方依赖的许可告知 | 备查 |
| `USER-GUIDE.md` | **本文**（详细功能与操作说明） | 就是这份 |

**两个不同的 WebView2 依赖（容易混）**：

| 依赖 | 是否随包 | 说明 |
|---|---|---|
| `WebView2Loader.dll`（加载器） | ✅ 随包 | 本程序用 GNU/MinGW 构建，动态链接此 DLL；缺了它 Windows 报「由于找不到 WebView2Loader.dll，无法继续执行代码」 |
| WebView2 **Evergreen Runtime**（浏览器内核） | ❌ 不随包，靠系统 | 一般 Windows 已随 Edge 装好；没装过的机器需自行安装（微软免费） |

> 首次运行弹出 SmartScreen「未知发布者 / Windows 已保护你的电脑」是**预期现象**（未买签名证书），由内测组织者确认后运行。

---

## 2. 五个核心概念

| 概念 | 含义 |
|---|---|
| **Vault（密文容器 / 根目录）** | 一个普通文件夹，里面是 `vault.meta`、`manifest`、`README.txt`、`d/` 等密文文件。它就是「保险柜本体」，**不依赖操作系统**，整目录拷走仍有效 |
| **解锁 / 锁定** | 解锁 = 用口令把容器临时变成可读写明文并映射成盘符；锁定 = 撤掉盘符、停掉服务，回到只有密文的状态 |
| **盘符映射（默认 `Z:`）** | 解锁后程序自动执行 `net use Z: http://127.0.0.1:4918/`，`Z:` 里就是明文视图。**这是 loopback WebDAV，不是 WinFsp/FUSE 驱动级挂载** |
| **主口令 / Recovery Key** | 两种凭据，二选一。建库时程序会打印一串 `E000-XXXX XXXX …` 的 **Recovery Key 展示码**，忘口令时用它解锁，**务必离线抄存** |
| **明文态 / 密文态** | 明文只在「解锁期间」存在；磁盘上的常态永远是密文。没有账号、没有云、没有遥测、没有 AI |

**M5 GUI 完善后**：GUI 已有「新建保险柜」按钮（把原 CLI-only 的 Step 1 收进界面，见 §3 / §4）；`mylock.exe create` 仍保留为备用/排障路径。**仍然没有**：「原地加密」、右键菜单加密、「加密/解密」按钮、账号登录、云同步、自动上报、崩溃遥测。

---

## 3. 图形界面（mylock-desktop.exe）详解

双击 `mylock-desktop.exe` 启动，窗口标题 `MyLock · 透明加密文件夹（M5 · 进程内引擎 · 封闭内测）`。

窗口最上方是一块**红框「⚠ 使用前必读」**（第一次用请逐条看）：讲清「不是原地加密 / 没有加密按钮 / 加密=拖进 Z: 盘 / 新建目标须全新空路径 / 抄存 Recovery Key / 未审计未签名勿存唯一副本」。

### 5 个输入框

| 输入框 | 填什么 | 默认 |
|---|---|---|
| **Vault 根目录** | 密文容器路径，如 `E:\testLockVault`（**新建时须为全新空路径**；解锁时须已建好） | 空 |
| **主口令** | 新建 / 解锁都用它 | 空 |
| **Recovery Key 展示码** | 忘口令时填；**与主口令二选一，填了口令就留空** | 空 |
| **映射盘符** | 单个字母，如 `Z`（也接受 `Z:` / `z`） | `Z` |
| **loopback 端口** | 本机监听端口 | `4918` |

### ① 第一次使用：新建保险柜

| 按钮 | 做什么 | 典型返回 |
|---|---|---|
| **新建保险柜**（橙色） | 先弹「使用前必要提示」确认框（把 5 条风险逐条念给你）→ 调内核建库（与 CLI 同一实现）→ 自动给容器加提示性保护 | 输出区显示建库结果 + `jti` + `key_epoch:0`；**下方绿色面板醒目显示 `E000-…` Recovery Key**（带「复制」按钮） |

建库会**拒绝**已存在且非空的目标目录（避免 `remove_dir_all` 误删）、拒绝云同步根目录、拒绝盘符根。**立刻离线抄存 Recovery Key**：程序不会再次完整显示它。

### ② 日常使用：5 个按钮

| 按钮 | 做什么 | 典型返回 |
|---|---|---|
| **解锁并映射**（主按钮） | 校验口令 → 进程内启动 loopback WebDAV → 自动 `net use` 映射盘符 → 幂等重放文件夹保护 | 「已解锁并映射 Z: → http://127.0.0.1:4918/」+ 文件夹保护状态 + 7 条安全边界 |
| **锁定并断开** | 先 `net use Z: /delete /y` 断开，再停服务 | 「已锁定…密文容器仍在原路径；拷贝走仍是保险柜，但不会自动变明文」 |
| **结构校验（不解锁）** | 解锁前只查结构与 KDF 参数，**不解密** | `{"phase":"pre-unlock","result":"OK","objects":N,"bytes":N}`（`N` 随容器内对象数/字节数变，实测真实值见 §4 Step 5） |
| **会话状态** | 查当前是否有挂载会话 | `会话运行中：true / false` |
| **端口探测** | 查端口是否被占用 | 「端口可用」/「端口已被占用，请换一个端口」 |

> **错误会如实显示且尽量说人话**。直接拿一个还没建库的路径去点「解锁并映射」，现在会得到「路径 … 下还没有保险柜（找不到 vault.meta）。请先点『新建保险柜』…」这种指引，而不再是底层串 `E_FORMAT: io: 系统找不到指定的路径。 (os error 3)`（该串只在内核更深处出现时才保留）。口令错会显示「主口令或 Recovery Key 不正确（AES-KW 完整性校验失败）」+ 底层 `E_AUTH_FAILED` 便于诊断，不会像早期版本那样「口令错了还说已启动」。

图形界面现在**能新建容器**（见上），但**不改口令、不导出、不真吊销**——这些仍在命令行里（§6 / §7）。

---

## 4. ★ 完整实操：把 `E:\testLock` 装进保险柜

假设 `E:\testLock` 里有：

```
E:\testLock\
├─ 会议记录.txt          「机密：季度预算 123 万」
├─ report.docx
└─ 子文件夹\
   └─ 照片说明.txt      「子目录里的机密」
```

### Step 0 · 准备（别跳过）

1. **先备份** `E:\testLock`（本版本未经审计、不保证数据不丢失）。
2. 选一个**全新的、不存在的**容器路径：`E:\testLockVault`。
3. 确认它**不在** OneDrive / Dropbox / Google Drive / iCloud / 坚果云 / 百度网盘 / 阿里云盘 等**云同步目录**里 —— 程序会拒绝（`E_SYNC_ROOT`），见 §9。
4. **推荐直接双击 `mylock-desktop.exe`**（GUI 就能建库，无需再开命令行）；命令行仅作备用。

### Step 1 · 新建保险柜（**GUI 主路径**）

1. 双击 `mylock-desktop.exe`，先看顶部**红框「⚠ 使用前必读」**。
2. 填：**Vault 根目录** = `E:\testLockVault`（全新空路径），**主口令** = `My-Demo-Pw-2026`，Recovery Key 留空。
3. 点橙色 **「新建保险柜」** → 弹出「使用前必要提示」确认框（逐条念风险：不是原地加密 / 目标会被清空 / 拒绝云同步根 / 抄存 RK / 勿存唯一副本）→ 点确定。
4. 输出区显示 `jti` / `key_epoch:0` / 建库结果，**下方绿色面板醒目显示 `E000-…` Recovery Key**（带「复制」按钮）。此时容器已自动加上保险柜图标与提示性保护。

> **立刻把 `E000-…` 那串 Recovery Key 抄到纸上 / 离线保存。** 它是忘口令时的唯一救命凭据，程序不会再次完整显示。此时 `E:\testLockVault` 已是一个**空保险柜**（`objects : 0`）。

**（备用）命令行建库**：GUI 不可用时，在便携包目录 `cmd` 里跑，效果完全相同：

```bat
mylock.exe create E:\testLockVault --password "My-Demo-Pw-2026"
```

真实输出（格式一致，值每次随机）：

```
created      : E:\testLockVault
objects      : 0
jti          : a76394c5-e8e7-4f94-a52c-58334ef5b86e
key_epoch    : 0
recovery_key : E000-M5IN RIZS D3SH WKEW NAP4 RAOI PNNA AVAU G2RF YDZP C2U3 EK6E JMPQ 9021
(请离线抄存 Recovery Key；换口令/换 RK 为零吊销，真吊销用 rotate-keys)
```

> 注意：**CLI `create` 不会自动加文件夹图标/隐藏保护**（那是桌面壳 `create_vault` 的职责）；下次用 GUI 解锁时会自动幂等补齐（见 §10）。

**（可选）建库时顺带放几个平铺文件**：`--file` 可重复，但**只能跟文件、不能跟文件夹**，且会**摊平**（只取文件名、不保留子目录）：

```bat
mylock.exe create E:\testLockVault --password "My-Demo-Pw-2026" --file E:\testLock\会议记录.txt --file E:\testLock\report.docx
```

实测 `--file` 传目录会失败：`E_FORMAT: read …\子文件夹: 拒绝访问。 (os error 5)`。**带子目录的文件夹请走 Step 3 的拖拽法。**

### Step 2 · 解锁映射（沿用 Step 1 已打开的 GUI 窗口）

1. 若 Step 1 用的是 GUI，窗口还开着，直接接着用；否则双击 `mylock-desktop.exe`。
2. 填：**Vault 根目录** = `E:\testLockVault`，**主口令** = `My-Demo-Pw-2026`，Recovery Key 留空，盘符 `Z`，端口 `4918`。
3. 点 **「解锁并映射」**（第一次解锁还会自动幂等重放文件夹保护）。

成功后输出区会显示「已解锁并映射 `Z:` → `http://127.0.0.1:4918/`」并列出安全边界。此时打开「此电脑」会多出一个 `Z:` 盘。

> 底层实际发生：程序在**自己的进程内**启动 loopback-only WebDAV（只监听 `127.0.0.1`），再自动执行 `net use Z: http://127.0.0.1:4918/ /persistent:no`。实测 `net use` 返回 `The command completed successfully.`；即使 Windows 的 `WebClient` 服务显示 Stopped，本机映射仍可成功（这是本机环境事实，非产品保证）。

### Step 3 · 把文件夹内容放进去（**这一步就是「加密」**）

在资源管理器里打开 `E:\testLock`，**把里面的内容全选复制、粘贴到 `Z:\`**（即把内容放到 `Z:` 根，而不是再套一层 `testLock`）。子目录会保留：

```
Z:\会议记录.txt
Z:\report.docx
Z:\子文件夹\照片说明.txt
```

实测在 `Z:` 里建子目录、写嵌套文件、读回全部正常：

```
MKDIR=OK   WRITE=OK
READ_TOP=机密：季度预算 123 万
READ_NESTED=子目录里的机密
Z_ITEM=Z:\子文件夹
Z_ITEM=Z:\会议记录.txt
Z_ITEM=Z:\子文件夹\照片说明.txt
```

> **也可以直接把整个 `E:\testLock` 文件夹拖进 `Z:\`**，形成 `Z:\testLock\会议记录.txt`、`Z:\testLock\子文件夹\照片说明.txt` 这样的子树 —— 同样实测通过（此时共 5 条记录：2 个目录 + 3 个文件）。两种放法都行，区别只在 `Z:` 里多一层目录名。下文 Step 4/5 的**文件结构是示意**，`verify` 的对象数 / 字节数随「放法 + 实际文件内容」变化，故 §4 Step 5 给出的 JSON 是一次**明确标注**的真实实测值，不是对本示例的精算。

**你不需要点任何「加密」按钮** —— 写进 `Z:` 的内容会自动分块加密后落到 `E:\testLockVault\d\` 下。此后在 `Z:` 里双击、编辑、另存为、删除、重命名都像普通文件夹一样，改动会自动写回密文容器（严格 copy-on-write，未改动的文件复用原密文）。

### Step 4 · 锁定（**这一步就是「解密态结束」**）

回到 GUI 点 **「锁定并断开」**（或命令行 `net use Z: /delete /y`）。`Z:` 消失。

此时 `E:\testLockVault` 里**只有密文**，实测结构：

```
E:\testLockVault\
├─ README.txt        (132B)
├─ vault.meta        (648B)
├─ vault.meta.bak    (648B)
├─ manifest          (791B)
└─ d\DU\DUPQZ4QL52TNCWGX5ISWIHQNAA  (105B)   ← 以 Base32 命名的密文对象
   d\H3\H3SSSDBEPM3BJBABIYLTVCYL7U  ( 76B)
   d\R6\R66WFF3JZL5OBERVQNWWO7N4UQ  ( 97B)
```

实测对整个容器扫描明文片段 `季度预算` / `子目录里的机密`，命中数 **`PLAINTEXT_HITS=0`**。

> **用 GUI「新建保险柜」建的容器**，除了上面的密文，还多了几个「保护性装饰」文件：`desktop.ini` / `MyLockVault.ico`（默认被 `+s +h` 隐藏，资源管理器里看不到）、以及一份**可见**的 `⚠ 请勿删除.txt` 警示；文件夹图标变成保险柜、删除/移动时会弹「系统文件夹」警告。它们**只是 shell 层提示，不是防删/权限机制，不保证数据不丢失**（详见 §10）。密文校验（§4 Step 5 的 `verify`）会忽略这些根级装饰文件，所以不影响保险柜本身。

### Step 5 · 验证「拔盘 / 拷走还是保险柜」

把**整个** `E:\testLockVault` 目录拷到 U 盘或另一路径，然后：

```bat
mylock.exe info   E:\testLockVault
mylock.exe verify E:\testLockVault --json
```

真实输出（`jti` / `objects` / `bytes` 每次随机/随内容变，以下为 2026-11-15 一次真实实测值）：

```
magic        : MYVAULT
format       : 1
cipher       : XCHACHA20-POLY1305
key_epoch    : 0
jti          : a76394c5-e8e7-4f94-a52c-58334ef5b86e
kdf          : alg=argon2id m=65536 t=3 p=4 salt_len=16B
gc_pending   : false
(公开头不含用户数据；未解密、未使用凭据)
```

```
{"phase":"pre-unlock","result":"OK","objects":3,"bytes":278}
(阶段1 只检结构与 KDF 参数；verify 绿不能证明是最新版，也不检恶意篡改)
```

拷走的目录仍是保险柜；在另一台机器上用**同一套程序 + 同一口令 / Recovery Key** 即可解锁。**但它不会自己变回明文** —— 必须解锁。

> `info` 不需要口令（只看公开头）；`verify` 也不解锁、不解密，只做结构与 KDF 硬校验。**verify 绿 ≠ 是最新版，也不检恶意篡改。**

### Step 6 · 自己删除原明文（MyLock 不会替你做）

`E:\testLock` 从始至终都是明文。**要真正「只留密文」，请你自行把 `E:\testLock` 安全删除**（普通删除后仍可能被恢复；本产品不提供安全擦除）。这一步是用户责任。

### Step 7 · 「解密 / 取回」= 再解锁，从 `Z:` 拷出来

重复 Step 2 解锁 → 打开 `Z:` → 把文件 / 文件夹复制回 `E:\` 任意位置。**这条路保留完整目录层级**，是推荐的正常取回方式。

> ⚠️ **CLI 的 `export` 会摊平目录层级**（见 §5），不要指望它还原 `子文件夹\照片说明.txt` 这种结构。

---

## 5. 「解密 / 取回文件」的三条路径

| 路径 | 命令 / 操作 | 目录层级 | 适用 |
|---|---|---|---|
| **① 正常：解锁后从 `Z:` 拷出** ✅ 推荐 | GUI 解锁 → 资源管理器复制 | **保留**（含子文件夹） | 日常 |
| **② 紧急整库导出**：`mylock.exe export` | 见下 | ❌ **摊平** | 桌面壳不可用时的兜底 |
| **③ 单对象抢救**：`export-object` | 见下 | 单文件 | `manifest` 损坏时 |

### ② `export`（完整解锁 → 导出全部明文对象）

```bat
mylock.exe export E:\testLockVault --out E:\Recovered --password "My-Demo-Pw-2026"
```

真实输出（用两个平铺文件实测）：

```
exported     : E:\Recovered\report.docx (19B)
exported     : E:\Recovered\会议记录.txt (30B)
records      : 2
(明文导出后其生命周期脱离本产品保护；请自行确保落盘位置安全)
```

**实测限制（重要，别踩坑）**：对含子目录的容器执行 `export`，目录层级**不会还原**。实测把整个文件夹拖进 `Z:` 形成 `Z:\testLock\…` 子树后，`export` 得到的是 **5 个平铺条目、没有目录树**：

```
decrypted\report.docx   (9B)
decrypted\testLock      (0B)   ← 目录被导成一个 0 字节「文件」
decrypted\会议记录.txt  (29B)
decrypted\子文件夹      (0B)   ← 目录被导成一个 0 字节「文件」
decrypted\照片说明.txt  (21B)   ← 嵌套文件被摊到顶层
```

因为 `export` 按对象的 `name` 平铺写入（内核 `full_open` 把 `name → Vec<u8>` 放进 `BTreeMap`，CLI 再 `out.join(name)` 写文件），同名文件还会互相覆盖。**所以：`export` 是「把内容抢救出来」的兜底手段，不是「还原文件夹树」；要树就用 ① 从 `Z:` 拷。**

> 导出的明文已脱离本产品保护，落在哪、留多久、怎么删，都是你的责任。

### ③ `export-object`（manifest 损坏时凭 object_id 直接解密单个对象）

```bat
mylock.exe export-object E:\testLockVault --object DUPQZ4QL52TNCWGX5ISWIHQNAA --out E:\one.bin --password "My-Demo-Pw-2026"
```

`--object` 是 `d\` 下那些 Base32 文件名。它**不读 manifest**，仍完成全头 `versionMac` 认证，是容器元数据损坏时的最后一根救命稻草。

---

## 6. 命令行（`mylock.exe`）9 个命令速查

`mylock.exe --help` 打印完整用法。逐条说明：

| 命令 | 作用 | 需要凭据 | 实测要点 |
|---|---|---|---|
| `info <root>` | 打印公开头（format/cipher/kdf/key_epoch/jti） | ❌ | `magic : MYVAULT`、`cipher : XCHACHA20-POLY1305` |
| `verify <root> [--json]` | 解锁前结构检查 + KDF 硬校验 | ❌ | `{"phase":"pre-unlock","result":"OK","objects":N,"bytes":M}` |
| `create <root> --password <pw> [--file <f> ...]` | 新建容器（**会清空已存在目录**） | 设新口令 | 打印 `recovery_key : E000-…` |
| `export <root> --out <dir> (--password\|--recovery)` | 完整解锁 + 导出全部明文 | ✅ | **摊平目录**，见 §5 |
| `export-object <root> --object <id> --out <f> (--password\|--recovery)` | 单对象抢救 | ✅ | 不读 manifest |
| `change-password <root> --password <old> --new <new>` | 换主口令 | 旧口令 | **零吊销**，见 §7 |
| `rotate-recovery <root> --password <pw>` | 换 Recovery Key | 口令 | **零吊销**，打印新展示码 |
| `rotate-keys <old> <new> --password <pw> --new <newpw>` | **真吊销**：全量重加密到新容器 | 旧口令 | 见 §7 |
| `mount <root>` | **明确拒绝**（透明挂载归桌面层） | — | 提示改用 GUI / `mylock-serve` |

**凭据写法**：`--password <pw>`（主口令，UTF-8）或 `--recovery "E000-XXXX …"`（展示码，自动校验校验位），**两者只能给一个**；`--new <newpw>` 是新口令。口令含空格请用引号括起来。

---

## 7. 口令、恢复码与「真吊销」

### 忘口令 → 用 Recovery Key

GUI：把「主口令」留空，在「Recovery Key 展示码」填建库时抄下的 `E000-…` 串，点解锁。
CLI：`mylock.exe export E:\testLockVault --out E:\Recovered --recovery "E000-XXXX XXXX …"`

实测展示码带校验位，写错会报 `recovery display invalid`。

### `change-password` / `rotate-recovery` 是「零吊销」

```bat
mylock.exe change-password E:\testLockVault --password "My-Demo-Pw-2026" --new "Another-Pw-2026"
```

真实输出会**明确警告**：

```
changed      : password slot re-wrapped
ZERO-REVOCATION: 持有旧 vault.meta + 旧口令者仍可解当前数据；
                 旧 Recovery Key 也仍有效。真吊销须 rotate-keys + 物理销毁旧介质。
```

**含义**：换口令只是重包了口令槽，**没有重新加密数据**。任何事先拿到旧 `vault.meta` + 旧口令（或旧 Recovery Key）的人，仍能解开当前容器。这两个命令**不等于**「让旧凭据失效」。

### `rotate-keys` 才是「真吊销」

```bat
mylock.exe rotate-keys E:\testLockVault E:\testLockVault-v2 --password "Another-Pw-2026" --new "Fresh-Pw-2026"
```

真实输出：

```
rotated      : E:\testLockVault-v2
jti          : 8c2d9432-2c5d-45e2-a1eb-2cb2e237a4f6
recovery_key : E001-BWEL 6YMU TBUZ J6OO ESZL EDM7 U6TK QXML LUX7 RUFQ EIUB T5SW KGLQ 59A1
TRUE REVOCATION: 旧口令 / 旧 RK / 旧 Master 均打不开新容器。
ACTION REQUIRED: 物理销毁旧介质 E:\testLockVault 及其所有副本（含云盘/备份）。
                 本命令没有、也不会替你删除旧目录。
```

实测用旧口令打开新容器：`E_AUTH_FAILED: aes-kw unwrap: integrity check failed`（EXIT=1）。
**注意 `E001` 前缀 = `key_epoch` 递增了；新容器是全新 jti / Master / salt / RK。但旧目录必须由你物理销毁（含所有备份），否则旧副本仍能被旧凭据解开。**

---

## 8. 拷到移动硬盘 / 备份

- **备份 = 拷贝整个 `vault` 目录**（`vault.meta` + `vault.meta.bak` + `manifest` + `README.txt` + `d/`）。它是静态普通文件目录，**不依赖操作系统**，拷到 U 盘 / 移动硬盘 / 另一台电脑仍是保险柜，实测 `PLAINTEXT_HITS=0`。
- 到另一台机器解锁需要：**同一套程序（或兼容内核）+ 主口令或 Recovery Key**。密文不会自己变明文。
- **不要**把 vault 放在云同步根目录内直接挂载 —— 程序路径组件若命中 `onedrive` / `dropbox` / `google drive` / `iclouddrive` / `坚果云` / `nutstore` / `sharepoint` / `pcloud` / `mega.nz` / `box sync` / `seafile` / `百度网盘` / `阿里云盘` / `同步空间` 等标记，会返回 `E_SYNC_ROOT` 并拒绝。
- 想用云盘备份，请**同步整个密文容器**，而不是在同步根里挂载明文盘符。

---

## 9. 常见问题与排错（错误串均为实测）

| 现象 | 原因 | 怎么办 |
|---|---|---|
| 「由于找不到 WebView2Loader.dll，无法继续执行代码」 | 只拷了 exe，没把整个文件夹拷走 | **整目录拷贝**，确保 `WebView2Loader.dll` 与 exe 同目录 |
| 「路径 … 下还没有保险柜（找不到 vault.meta）。请先点『新建保险柜』」 | 拿一个还没建库的路径直接点了「解锁并映射」 | 先点「新建保险柜」（GUI 主路径）或备用 `mylock.exe create`；旧版底层串 `E_FORMAT: io: 系统找不到指定的路径。 (os error 3)` 已被这个人话提示取代 |
| 「新建保险柜」报「目标目录已存在且非空」 | 指向了已有内容的目录 | 换全新空路径；或先把原内容移走 —— 程序**拒绝**清空非空目录，避免误删 |
| SmartScreen「未知发布者」 | 未买签名证书 | 预期现象，组织者确认后运行 |
| `E_AUTH_FAILED: aes-kw unwrap: integrity check failed`（EXIT=1） | 口令 / Recovery Key 错 | 核对凭据；忘了就用 Recovery Key |
| 「端口 4918 已被占用」 | 别的程序占了端口 | 点「端口探测」确认，换一个端口（如 51918）再解锁 |
| `E_SYNC_ROOT` 拒绝在云同步目录内挂载 | vault 路径命中同步目录标记 | 把 vault 移到本地盘 / 离线介质（见 §8） |
| 「已有挂载会话在运行，请先『锁定并断开』」 | 一个 GUI 同时只挂一个 vault | 先锁定再开另一个 |
| 「`--password` 与 Recovery Key 只能提供其中一个」 | 两个框都填了 | 留空其中一个（GUI 里口令模式务必让 Recovery 框为空） |
| 「自动映射盘符失败」 | `net use` 被系统策略 / 残留映射挡住 | 按输出提示手动执行 `net use Z: http://127.0.0.1:4918/ /persistent:no` |
| `E_FORMAT: read …: 拒绝访问。 (os error 5)` | `--file` 传了文件夹而不是文件 | `--file` 只接文件；带子目录请用 `Z:` 拖拽法 |
| **`create` 后原文件不见了** | 把 `create` 指向了已存在的非空目录 | 见 §0 警告：目标必须是全新空路径；原文件需从备份恢复 |
| `verify` 显示 OK 但仍不放心 | `verify` 只检结构 + KDF 参数 | 正常；它**不检恶意篡改、不证明是最新版** |
| 解锁后别的程序也能读 `Z:` | loopback WebDAV 无 per-request 认证 | 见 §10，这是设计取舍，务必知悉 |

---

## 10. 能力与边界（诚实清单）

**能做的**
- 本地优先、零知识、无账号、无云、无遥测、无 AI 的密文容器。
- 解锁后像普通文件夹一样读写（含中文名、子目录），写入自动加密、严格 copy-on-write。
- 整目录拷到移动介质仍是保险柜；可用 Recovery Key 兜底、可真吊销 `rotate-keys`。
- 独立恢复 CLI 即使桌面壳不可用也能打开容器（纯 Rust，不依赖 WebView2）。

**不能 / 未做（逐条如实）**
- ❌ **不是** WinFsp/FUSE 驱动级挂载；是 loopback-only WebDAV（OS 原生 `net use`）。
- ❌ **WebDAV loopback 无 per-request 认证**：解锁期间**任何本机进程**都能访问 `Z:` 挂载点。
- ❌ **仅在「未解锁态」防得住合法采集者；解锁态只缓解、不消除风险**；库外已存在的明文 / 临时副本，锁定后**不保证消失**；不防已攻陷设备；口令熵是安全上限。
- ❌ **不就地加密、不删原件**；`create` 会清空已存在的目标目录（§0）。
- ❌ **CLI `export` 摊平目录层级**、同名覆盖（§5）；要还原文件夹树请从 `Z:` 拷出。
- ❌ **文件夹的保险柜图标 / 隐藏属性 / 删除警告只是「提示性」shell 层保护，不是防删 / 权限（ACL）机制，不保证数据不丢失**：真正删掉整目录或 `vault.meta` / `manifest` / `d\` 里任何一个，保险柜照样失效。A 级依据 [Microsoft Learn：desktop.ini 文件夹自定义](https://learn.microsoft.com/en-us/windows/win32/shell/how-to-customize-folders-with-desktop-ini)：`attrib +s` 启用 `Desktop.ini` 特殊行为、隐藏/系统属性、省略 `ConfirmFileOp=0` 以保留「删除系统文件夹」警告、`IconFile`+`IconIndex=0` 给保险柜图标；`tools/probe-attrib.ps1` 实测 `+s +h` 不阻止程序化读写、但 `manifest` 每次写回被原子 rename 覆盖会丢属性，故**每次解锁幂等重放**。防误删的最终手段仍是**备份 + 请勿存唯一副本**。
- ❌ **形状泄漏未 100% 消除**；用户数据零明文 ≠ 零元数据；XChaCha20-Poly1305 非 IETF 标准；内部评审 ≠ 第三方审计。
- ❌ G4 五端一致性只完成 Rust/Windows 单端；**Android / iOS 端未实现**，本包只有 Windows 桌面 + CLI。
- ❌ 未代码签名、未经第三方安全审计、**未产 NSIS/MSI 安装包**、SmartScreen 警告属预期。
- ❌ 封闭内测 ≠ 公开发布 / 商店上架；合规事实为 **2026-11-15 抓取时点值**；**推迟 ≠ 豁免**（EAR 出口分类、PIPL/GDPR controller 定性、MPL 源码可得义务不因规模小 / 免费 / 不公开而消失）；真要上架或收费那天 A（审计）/ B（律师签发）/ C（签名包）/ D（商店申报）四道闸门**一个不能跳**。

---

## 11. 建议的内测测试清单（「怎么测试」）

按顺序勾选，覆盖正常路径 + 失败路径 + 拷贝路径：

- [ ] 1. GUI「新建保险柜」在全新空路径建库，读懂使用前必读提示、确认框，**抄存绿色面板里的 Recovery Key**；`info`（或 GUI 结构校验）能看到 `MYVAULT` / `XCHACHA20-POLY1305` / `argon2id`；文件夹应显示保险柜图标、里面有可见 `⚠ 请勿删除.txt`。
- [ ] 2. `verify --json` 返回 `result:OK`。
- [ ] 3. GUI 解锁并映射 → 「此电脑」出现 `Z:`。
- [ ] 4. 把含**子文件夹 + 中文文件名**的 `E:\testLock` 整个拖进 `Z:`；逐层读回内容一致。
- [ ] 5. 在 `Z:` 里新建 / 改名 / 删除 / 另存为一个大文件（>2 MB），再读回一致。
- [ ] 6. 点「锁定并断开」→ `Z:` 消失；打开 `E:\testLockVault` 只看到 `vault.meta` / `manifest` / `d\…` 密文。
- [ ] 7. **错误口令**解锁 → 得到 `E_AUTH_FAILED`，不是「已启动」。
- [ ] 8. **Recovery Key**（口令框留空）解锁成功。
- [ ] 9. 把整个 `E:\testLockVault` 拷到 U 盘 / 另一路径 → `verify` 仍 OK、明文片段命中 0；换路径用同口令能解锁。
- [ ] 10. `change-password` 后旧口令打不开、新口令能打开（并读懂「零吊销」警告）。
- [ ] 11. `rotate-keys` 到新容器 → 旧口令打不开新容器、新口令能打开；确认旧目录需自行销毁。
- [ ] 12. CLI `export` 兜底导出成功（并**亲眼确认目录被摊平**，理解 §5 的限制）。
- [ ] 13. 故意制造异常：端口占用 → 「端口探测」提示；把 vault 放进名字含 `OneDrive` 的目录 → `E_SYNC_ROOT` 拒绝。
- [ ] 14. 只拷 exe 不带 DLL → 复现缺 DLL 报错；整目录拷贝 → 正常弹窗。

> 反馈只走**主动渠道**（访谈 / 问卷 / 主动发消息）。本包**无埋点、无遥测、无崩溃上报、无账号、无云同步**。

---

## 12. 文档索引

| 文档 | 内容 |
|---|---|
| [USER-GUIDE.md](USER-GUIDE.md) | 本文 v1.1：功能与操作说明、完整示例（含 GUI 新建保险柜 / 文件夹提示性保护） |
| [BETA-NOTICE.md](BETA-NOTICE.md) | 封闭内测须知、风险、NO-GO 四道闸门 |
| [README-BETA.md](dist/MyLock-Beta-0.1.0/README-BETA.md) | 包内速览（打包脚本生成） |
| [m5-report.md](m5-report.md) | M5 研发收口总报告（含 §1.5 WebView2Loader.dll 热修复） |
| [产品分析与概要设计.md](产品分析与概要设计.md) | 主设计文档 v1.9 |
| [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) | 第三方依赖许可告知 |
| [m4-step4-report.md](m4-step4-report.md) | 合规事实调研（A/A⁻/B/C 分级） |
| [android-feasibility.md](android-feasibility.md) | Android 可行性（本轮不产 APK） |
| [security-whitepaper.md](security-whitepaper.md) | 安全白皮书草案（待律师 / 审计） |

> 本手册是**操作说明**，不是合规结论、不是法律意见、不是第三方审计报告、不是已通过认证；草案未经律师签发不得对外发布。
