# AI 漫剧生产痛点与 SaladCart 对策

调研时间：2026-10。来源包括行业报道、平台规则、供应商文档与创作者社区。市场与收入数字属于第三方估算，仅用于判断痛点优先级，不作为产品承诺。

## 优先级与产品映射

| # | 生产痛点 | 证据 / 现象 | SaladCart v0.1 对策 | 状态 |
| --- | --- | --- | --- | --- |
| 1 | **角色跨镜头漂移**：脸型、发型、服装、配色每抽一次就变 | 腾讯云文档将“角色漂移”列为 AI 漫剧核心难点；创作者社区把一致性称为“最大的头痛” ([腾讯云](https://developer.cloud.tencent.com/article/2755779), [Reddit](https://www.reddit.com/r/AI_VideoGenerator/comments/1p698s9/ai_video_tools_are_improving_fast_but_consistency/)) | 角色圣经 + `lockedTraits` 硬约束 + 定妆照/三视图；生成上下文自动注入；镜头默认尾帧续接；seed 与参考图随资产保存 | 已实现 |
| 2 | **剧本到分镜靠手拆**：整本小说到分集/场景/镜头工作量大，且没有镜头时长、机位、动作和结束状态 | 行业流程普遍是“剧本 → 场景/镜头 → 关键帧 → 图生视频” ([腾讯云 6 步流程](https://developer.cloud.tencent.com/article/2754229), [分镜实践](https://xjstoryboard.com/blog/?p=110)) | 结构化分集/场景/镜头；整本小说按章分块理解并跨章合并角色；一次生成镜头时长、机位、对白、声音与画面指令；镜头卡直接可改 | 已实现（长篇小说改编） |
| 3 | **“抽卡”失控**：一条可用镜头要 5–20 次尝试，额度与时间同时燃烧 | 创作者反馈反复抽卡与角色漂移会快速消耗额度 ([Reddit](https://www.reddit.com/r/aivideomaking/comments/1vxftgf/hello_everyone_im_looking_for_the_best_ai_video/))；行业出现专职“抽卡师” ([虎嗅](https://www.huxiu.com/article/4841498.html)) | 生成前上下文预览与成本估算；批量选择镜头排队；并发上限；失败重试；每次 take 保留版本、seed、模型与父资产 | 已实现 |
| 4 | **素材/版本混乱**：最新帧不知道在哪，旧素材被误用，交付靠聊天软件 | 行业文章建议 5 类素材命名、版本与归档规则，并指出“旧素材”造成返工 ([阿里云](https://developer.aliyun.com/article/1774992)) | 每项目独立 SQLite + 素材库；图片/视频/音频/字幕统一登记；来源、模型、seed、版本与父版本入库；素材可审核 | 已实现 |
| 5 | **后期装配最耗时**：字幕、配乐、对轨、镜头衔接把手工作品拖回剪辑软件 | 5 分钟成片仅音画同步就可能花 1–2 小时 ([SegmentFault](https://segmentfault.com/a/1190000048159497))；平台对字幕/时长有明确规格 ([火山引擎](https://developer.volcengine.com/articles/7631799169119682614)) | 按剧本顺序自动装配；粗剪/裁剪/重排；字幕轨；配乐轨；自动卡点；FFmpeg 烧字幕导出竖屏/横屏 MP4 | 已实现 |
| 6 | **多集/系列连续性**：60–120 集甚至千集，角色、场景、声音必须长期一致 | 行业报道显示单集约 1 分钟、60–120 集常见，IP 化后可达千集 ([虎嗅](https://www.huxiu.com/article/4800925.html), [钛媒体](https://www.tmtpost.com/8172188.html)) | 项目内多集结构；跨项目角色复制（复制素材，不共享实时数据）；风格圣经；连续性链接 | 已实现基础；系列模板与批量集数管理列入路线图 |
| 7 | **审片与交接**：几十人团队的批注散落在聊天里，版本没有锁 | 报道显示大型团队可达 100+ 人（30 美术 + 50 抽卡师）([财联社](https://www.cls.cn/detail/2482385))；素材文章提到微信交付与版本错用 ([阿里云](https://developer.aliyun.com/article/1774992)) | 镜头/素材状态（草稿、待审、已通过）；资产不可变版本；本地任务记录 | 基础状态已实现；评论、角色权限与共享审片列入路线图 |
| 8 | **平台交付规格**：9:16、1080×1920、时长、硬字幕/CC、安全区 | 抖音/快手竖屏与时长规则、YouTube Shorts ≤3 分钟、硬字幕与多语言交付要求 ([新浪/抖音规则](https://finance.sina.com.cn/wm/2026-05-31/doc-inhzwpyv2519666.shtml), [YouTube](https://support.google.com/youtube/answer/15424877)) | 9:16 / 16:9 / 1:1 / 4:5 预设；H.264 + AAC；字幕烧录；导出文件名与时长摘要 | 已实现；安全区叠加与多语言版本集在路线图 |
| 9 | **合规与溯源**：AI 漫剧备案、AI 标识、剧本/提示词/参数留存、音乐版权 | 2026 年起 AI 漫剧备案与 AI 标签要求，平台要求保留生成证据 ([TVOAO](https://www.tvoao.com/a/223713.aspx), [CAC 标识办法](https://www.cac.gov.cn/2025-03/14/c_1743654684782215.htm)) | 资产级 provider/model/prompt/seed/parent 溯源；音乐 license/license URL/attribution 字段；项目本地可审计；无遥测 | 溯源已实现；备案清单导出与 AI 元数据写入成片列入路线图 |
| 10 | **成本/ROI 不透明**：单分钟成本差异巨大，失败任务也算钱 | 行业报道给出每分钟数百到数万元的成本区间，以及较低爆款率 ([钛媒体](https://www.tmtpost.com/7905101.html), [36氪](https://36kr.com/p/3715043170857089)) | 每个任务保存成本估算；作业记录 provider/model/成本；上下文预览先看成本再生成；项目预算上限；Mock 本地零成本跑通流程 | 已实现估算、记录与预算上限；供应商账单同步、按可用秒 ROI 列入路线图 |

## 为什么先做这五件事

角色一致性（1）、剧本分镜（2）、抽卡编排（3）、素材版本（4）、后期装配（5）构成每天都会发生的主循环。它们共享同一个项目图：没有一致的镜头结构，就无法稳定注入角色；没有素材溯源，就无法比较 take；没有时间线，生成结果仍然要回到另一个软件交付。SaladCart 的 v0.1 先把这条主循环闭合。

## 音乐 / 自动卡点专项

| 能力 | 实现 |
| --- | --- |
| 正版曲库搜索 | Jamendo API；只展示允许下载的曲目；保存 CC 许可证、作者与来源 |
| AI 配乐 | Stability Audio 3、ElevenLabs Music；Mock 本地合成 100 BPM 免版权占位配乐 |
| 节拍分析 | FFmpeg 解码 → Rust 计算 onset envelope / 自相关 BPM / 相位 → 毫秒级 beat grid |
| 自动卡点 | 切点吸附到最近节拍（±180ms），最少两拍；保留源片段入出点上限 |
| 导出 | 配乐循环、音量控制、字幕烧录、H.264/AAC MP4 |
| 版权提示 | CC-BY 需署名，CC-BY-NC 禁止商用，AI 音乐条款随供应商变化；资产记录 license 与 attribution |

Suno 的公开 API 目前是合作方白名单，Udio 没有公开 API；这两个服务不做非官方包装。Jamendo/Freesound 的免费额度面向非商业应用，商业发行需要相应授权。

## 主要参考

- [AI 漫剧市场与产量（新华社）](https://english.news.cn/20260325/3169f5e692ca4d80890dd4400fdc31f7/c.html)
- [腾讯云：AI 漫剧 6 步工作流](https://developer.cloud.tencent.com/article/2754229)
- [腾讯云：角色漂移与一致性](https://developer.cloud.tencent.com/article/2755779)
- [阿里云：素材命名、版本与交付](https://developer.aliyun.com/article/1774992)
- [火山引擎：字幕与多语言交付规格](https://developer.volcengine.com/articles/7631799169119682614)
- [YouTube Shorts 时长](https://support.google.com/youtube/answer/15424877)
- [TikTok/抖音时长规则报道](https://finance.sina.com.cn/wm/2026-05-31/doc-inhzwpyv2519666.shtml)
- [CAC：AI 生成内容标识办法](https://www.cac.gov.cn/2025-03/14/c_1743654684782215.htm)
- [Stability Audio API](https://docs.stability.ai/api-reference/stable-audio/text-to-audio)
- [ElevenLabs Music API](https://elevenlabs.io/docs/api-reference/music/compose)
- [Jamendo API](https://developer.jamendo.com/v3.0/tracks)
- [Beat This（可选更高精度节拍模型）](https://github.com/danigb/beat-this-rs)

> 数据与第三方报道只用于产品取舍。成本估算不构成报价；真实费用以供应商账单为准。使用 CC 曲库或 AI 音乐前请核对当前许可证与商用范围。
