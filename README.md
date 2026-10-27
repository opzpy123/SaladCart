# 沙拉车 SaladCart

> 一台为 AI 漫剧和 AI 短剧创作者打造的原生桌面工作站。剧本、角色、分镜、视频、配乐与剪辑在同一个项目里自动流转。

SaladCart 使用 **Tauri 2 + Rust + React** 构建，原生支持 macOS 与 Windows。每个项目拥有独立的 SQLite 数据库、素材目录与成片目录。项目内的剧本、角色、风格圣经、分镜、配乐与生成任务互相引用；生成视频时自动组装上下文，先预览再付费。

![SaladCart native workstation](docs/assets/native-dashboard.png)

## 为什么是它

浏览器生成器只解决“生成一次”，SaladCart 解决一部漫剧的循环：

1. **从立意到分镜**：LLM 一次产出故事圣经、角色、分集、场景和镜头（机位、镜头、时长、对白、声音）。
2. **角色锁定**：角色圣经 + 定妆照/三视图 + 硬性特征锁，生成每个镜头时自动注入。
3. **上下文可见**：发送前看到最终提示词、角色参考、连续性、画幅与成本。上下文编译器是产品核心。
4. **批量抽卡**：持久化任务队列、并发上限、失败重试、每次 take 的版本、seed、模型与父资产。
5. **配乐与卡点**：搜索正版曲库或 AI 生成配乐；Rust 原生节拍分析；切点自动吸附到节拍。
6. **粗剪与交付**：自动按剧本装配、字幕、配乐、9:16 / 16:9 / 1:1 / 4:5，FFmpeg 导出 MP4。
7. **项目隔离与复用**：每个项目独立存储；从其他项目导入角色与风格会复制素材，不建立实时依赖。

## 快速开始

前置：Node.js ≥ 20.19、pnpm、Rust stable、FFmpeg。

```bash
pnpm install
pnpm dev            # 浏览器演示桥接（无需 Key，含完整演示项目）
pnpm dev:tauri      # 原生桌面开发模式
pnpm test           # 前端测试
pnpm test:rust      # Rust 测试
pnpm build:tauri    # 打包当前平台
pnpm seed:demo      # 向本机工作区注入一部完整演示项目（剧本/角色/视频/配乐）
```

FFmpeg：macOS `brew install ffmpeg`；Windows `winget install Gyan.FFmpeg`。也可以把可执行文件路径填入设置，或设置 `SALADCART_FFMPEG`。未安装 FFmpeg 时剧本与图片生成仍可用，视频与导出会给出可操作的错误。

## 已接入的供应商

| 能力 | 适配器 |
| --- | --- |
| 编剧 / LLM | Mock、任意 OpenAI 兼容接口（OpenAI、DeepSeek、Qwen/DashScope、Ollama） |
| 文生图 / 角色参考 | Mock、OpenAI Images、Gemini / Imagen、Replicate、ComfyUI |
| 视频 | Mock（本地 FFmpeg）、Kling、Runway、Luma、MiniMax、Veo、Replicate、fal.ai |
| 音乐 | Mock（本地合成 100 BPM）、Stability Audio、ElevenLabs Music；Jamendo 正版曲库搜索 |

密钥只存在 macOS Keychain / Windows Credential Manager，不进入项目、日志或渲染层。详细配置与已知接口差异见 [docs/provider-setup.md](docs/provider-setup.md)。

**没有 Key 也能完整跑通**：Mock 编剧会生成中文分镜，Mock 画师生成品牌化占位画面，Mock 视频通过 FFmpeg 生成本地片段，Mock 配乐合成可卡点的 100 BPM 音轨。浏览器演示模式会自动加载「雾港来信」；原生端可在概览点「载入演示项目」，或用 `pnpm seed:demo` 注入完整示例。

## 项目数据

```
macOS:   ~/Library/Application Support/com.saladcart.studio
Windows: %APPDATA%\com.saladcart.studio
  settings.json
  projects/<uuid>/
    project.db
    project.json
    assets/{images,videos,audio}
    thumbs/
    exports/
  trash/
```

项目可整体迁移。删除项目时移入 `trash/`，不直接永久删除。

## 验证

当前 v0.1.0 的自动化验证：

- Rust（11 项）：项目隔离、任务读写、Mock 剧本生成、剧本落库、角色绑定、上下文编译器、完整演示项目、时间线自动装配、节拍吸附、节拍检测、Mock WAV、真实 FFmpeg MP4 渲染。
- 前端（6 项）：时间码/时长工具、浏览器演示桥接、上下文编译、Jamendo 曲库搜索、节拍分析/卡点。
- 手动：Chromium 走查 Dashboard、剧本、角色、分镜、剪辑、音乐、设置；原生 `.app` 启动验证。
- 跨平台：GitHub Actions 矩阵包含 macOS arm64 / x64 与 Windows x64 构建；未配置 Apple/Windows 签名证书，因此产物为未签名版本。

```bash
pnpm test
SALADCART_FFMPEG=$(command -v ffmpeg) cargo test --manifest-path src-tauri/Cargo.toml
```

## 发布状态

`v0.1.0` 的源码已提交到仓库。macOS arm64 未签名 DMG 在 `release/SaladCart_0.1.0_aarch64.dmg`，校验值、安装提示与已知边界见 [docs/release-0.1.0.md](docs/release-0.1.0.md)。Windows 构建在 GitHub Actions 矩阵中，但本次会话未执行 Windows 验证；没有推送 tag，也没有创建公开 GitHub Release（本机未安装/登录 `gh`）。

## 产品研究

AI 漫剧的真实生产痛点、证据来源与逐项对策见 [docs/pain-points.md](docs/pain-points.md)。架构与阶段决策见 [docs/architecture.md](docs/architecture.md)。

## 已知边界（v0.1）

- 尚未集成评论、角色权限、共享审片与备案清单导出；资产的 provider/model/prompt/seed/父版本与音乐 license/attribution 已入库，为后续合规导出打底。
- 自动卡点使用轻量 Rust DSP，不含 downbeat ML；可后续接入 Beat This 做可选高精度分析。
- Kling / Luma / MiniMax / Veo 等视频 API 的路径、模型名与账号区域会变化；适配器保留 provider options 覆盖与原始错误，不做假成功。
- 未包含口型同步、声音克隆与多轨 NLE；v0.1 聚焦从生成到可交付粗剪的闭环。
- 发布产物未签名/未公证，生产分发前需配置 Apple Developer 与 Windows 代码签名证书。

## License

Source code © SaladCart. Third-party provider terms and music licences apply separately.
