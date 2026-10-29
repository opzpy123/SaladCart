import { useEffect, useRef, useState } from "react";
import { BookOpen, Clapperboard, Sparkles, Upload, Wand2 } from "lucide-react";
import { Button, Field, InlineNotice, Modal } from "./ui";
import { formatError, invoke, isDesktop, pickNovelFile } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { GenerateScriptRequest, NovelPlan } from "@/lib/types";
import { enabledProviders, preferredProviderId } from "@/lib/providers";

const STYLE_PRESETS = [
  {
    id: "anime-cel",
    name: "动画赛璐璐",
    desc: "干净的赛璐璐上色、电影级构图，适合连续剧集。",
    colors: ["#202a1c", "#a9c765", "#e8e2cf"],
  },
  {
    id: "guofeng-ink",
    name: "国风水墨",
    desc: "宣纸留白与墨色层次，适合东方奇幻。",
    colors: ["#29251f", "#b89772", "#f2ead8"],
  },
  {
    id: "cinematic-3d",
    name: "电影级 3D",
    desc: "体积光与景深，角色材质统一，适合科幻。",
    colors: ["#102229", "#5fa6a0", "#cfe7e4"],
  },
  {
    id: "manga-noir",
    name: "漫画黑色",
    desc: "高反差网点、硬光剪影，适合悬疑犯罪。",
    colors: ["#141414", "#d8d3c4", "#d95d39"],
  },
];

const ASPECTS = [
  { id: "9:16", label: "竖屏 9:16", note: "抖音 / 快手 / Shorts" },
  { id: "16:9", label: "横屏 16:9", note: "B 站 / YouTube" },
  { id: "1:1", label: "方形 1:1", note: "信息流 / 预告" },
];

export function NewProjectDialog() {
  const open = useAppStore((s) => s.showNewProject);
  const setOpen = useAppStore((s) => s.setShowNewProject);
  const createProject = useAppStore((s) => s.createProject);
  const settings = useAppStore((s) => s.settings);
  const loading = useAppStore((s) => s.loading);
  const [name, setName] = useState("");
  const [premise, setPremise] = useState("");
  const [genre, setGenre] = useState("都市奇幻");
  const [tone, setTone] = useState("冷冽、诗意、克制");
  const [aspect, setAspect] = useState("9:16");
  const [episodes, setEpisodes] = useState(1);
  const [style, setStyle] = useState("anime-cel");
  const [generateAfter, setGenerateAfter] = useState(true);
  const [providerId, setProviderId] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    if (open) {
      setName("");
      setPremise("");
      setGenre("都市奇幻");
      setTone("冷冽、诗意、克制");
      setAspect("9:16");
      setEpisodes(1);
      setStyle("anime-cel");
      setGenerateAfter(true);
      setProviderId(
        preferredProviderId(
          settings?.providers ?? [],
          "llm",
          settings?.defaults.llm ?? "mock-llm",
        ),
      );
      setError("");
    }
  }, [open, settings]);

  const submit = async () => {
    if (!premise.trim()) {
      setError("先写下故事的核心设定，哪怕只有一句话。");
      return;
    }
    await createProject(
      {
        name: name.trim(),
        premise: premise.trim(),
        genre,
        tone,
        language: "zh-CN",
        aspectRatio: aspect,
        targetEpisodeSeconds: 90,
        stylePreset: style,
        styleBible: null,
      },
      generateAfter,
      episodes,
      providerId,
    );
  };

  const llms = enabledProviders(settings?.providers ?? [], "llm");
  const llm = llms.find((p) => p.id === providerId);
  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title="开一部新漫剧"
      subtitle="项目会独立存储剧本、角色、分镜、素材和剪辑时间线。生成时自动把它们编译成上下文。"
      wide
      footer={
        <>
          <span className="modal__footnote">
            {generateAfter ? (
              <>
                <Wand2 size={14} /> 使用 {llm?.label ?? "默认编剧"} 自动生成故事圣经与分镜
              </>
            ) : (
              "先建空项目，稍后再生成"
            )}
          </span>
          <Button variant="ghost" onClick={() => setOpen(false)}>
            取消
          </Button>
          <Button variant="primary" icon={<Sparkles size={16} />} loading={loading} onClick={submit}>
            创建并进入
          </Button>
        </>
      }
    >
      <div className="new-project-grid">
        <div className="new-project-main">
          <div className="form-grid form-grid--two">
            <Field label="项目名" hint="可以留空，生成后自动采用剧名">
              <input
                className="input"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="例如：雾港来信"
              />
            </Field>
            <Field label="类型">
              <input className="input" value={genre} onChange={(event) => setGenre(event.target.value)} />
            </Field>
          </div>
          <Field label="一句话立意" hint="主角、欲望、阻碍、一个反转钩子。悬疑、奇幻、都市、古风都可以。">
            <textarea
              className="textarea textarea--hero"
              value={premise}
              onChange={(event) => setPremise(event.target.value)}
              placeholder="在常年落雨的海临城，一个能触碰旧物读取记忆的女孩，发现一枚铜钥匙里存着自己的声音，而每次读取都会让她失去一段记忆。"
              rows={5}
            />
          </Field>
          {error ? <InlineNotice tone="error">{error}</InlineNotice> : null}
          <div className="form-grid form-grid--two">
            <Field label="气质">
              <input className="input" value={tone} onChange={(event) => setTone(event.target.value)} />
            </Field>
            <Field label="首季集数">
              <input
                className="input"
                type="number"
                min={1}
                max={20}
                value={episodes}
                onChange={(event) => setEpisodes(Number(event.target.value))}
              />
            </Field>
          </div>
          <Field
            label="编剧模型"
            hint={generateAfter ? "选择实际执行生成的模型；已配置 Key 的真实模型优先于 Mock" : "仅创建空项目"}
          >
            <select
              className="input"
              value={providerId}
              disabled={!generateAfter || llms.length === 0}
              onChange={(event) => setProviderId(event.target.value)}
            >
              {llms.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.label}
                  {provider.id.startsWith("mock-") ? " · 离线" : provider.apiKeySet ? " · 已配置 Key" : ""}
                </option>
              ))}
            </select>
          </Field>
          <Field label="交付画幅">
            <div className="aspect-picker">
              {ASPECTS.map((item) => (
                <button
                  type="button"
                  key={item.id}
                  className={`aspect-card ${aspect === item.id ? "is-active" : ""}`}
                  onClick={() => setAspect(item.id)}
                >
                  <span className={`aspect-shape aspect-shape--${item.id.replace(":", "-")}`} />
                  <strong>{item.label}</strong>
                  <small>{item.note}</small>
                </button>
              ))}
            </div>
          </Field>
          <label className="switch-row">
            <input
              type="checkbox"
              checked={generateAfter}
              onChange={(event) => setGenerateAfter(event.target.checked)}
            />
            <span>
              <strong>创建后立即生成剧本与分镜</strong>
              <small>
                {llm
                  ? `当前使用 ${llm.label}${
                      llm.id.startsWith("mock-")
                        ? "（离线，无需 Key）"
                        : llm.apiKeySet
                          ? "（已配置 Key）"
                          : "（未配置 Key）"
                    }`
                  : "未找到可用编剧模型"}
              </small>
            </span>
          </label>
        </div>
        <aside className="new-project-style">
          <div className="new-project-style__head">
            <Clapperboard size={16} />
            <span>视觉风格</span>
          </div>
          <div className="style-picker">
            {STYLE_PRESETS.map((preset) => (
              <button
                key={preset.id}
                type="button"
                className={`style-card ${style === preset.id ? "is-active" : ""}`}
                onClick={() => setStyle(preset.id)}
              >
                <span className="style-card__swatches">
                  {preset.colors.map((color) => (
                    <i key={color} style={{ background: color }} />
                  ))}
                </span>
                <span>
                  <strong>{preset.name}</strong>
                  <small>{preset.desc}</small>
                </span>
              </button>
            ))}
          </div>
          <div className="palette-note">
            <p className="eyebrow">项目独立存储</p>
            <p>每个项目都有自己的 SQLite、素材与成片目录。可以从其他项目导入角色和风格，但不会共享实时数据。</p>
          </div>
        </aside>
      </div>
    </Modal>
  );
}

export function GenerateScriptDialog() {
  const open = useAppStore((s) => s.showScriptDialog);
  const setOpen = useAppStore((s) => s.setShowScriptDialog);
  const snapshot = useAppStore((s) => s.snapshot);
  const settings = useAppStore((s) => s.settings);
  const generate = useAppStore((s) => s.generateScript);
  const [premise, setPremise] = useState("");
  const [genre, setGenre] = useState("");
  const [tone, setTone] = useState("");
  const [episodes, setEpisodes] = useState(1);
  const [scenes, setScenes] = useState(2);
  const [shots, setShots] = useState(3);
  const [notes, setNotes] = useState("");
  const [replace, setReplace] = useState(true);
  const [providerId, setProviderId] = useState("");

  useEffect(() => {
    if (!open || !snapshot) return;
    setPremise(snapshot.project.premise);
    setGenre(snapshot.project.genre);
    setTone(snapshot.project.tone);
    setEpisodes(1);
    setScenes(2);
    setShots(3);
    setNotes("");
    setReplace(snapshot.episodes.length > 0);
    setProviderId(
      preferredProviderId(
        settings?.providers ?? [],
        "llm",
        settings?.defaults.llm ?? "mock-llm",
      ),
    );
  }, [open, snapshot, settings]);

  if (!snapshot) return null;
  const llms = enabledProviders(settings?.providers ?? [], "llm");
  const hasContent = snapshot.episodes.length > 0;

  const submit = async () => {
    const request: Partial<GenerateScriptRequest> = {
      premise: premise.trim() || snapshot.project.premise,
      genre,
      tone,
      episodes,
      scenesPerEpisode: scenes,
      shotsPerScene: shots,
      styleNotes: notes,
      providerId: providerId || null,
      replaceExisting: replace,
    };
    await generate(request);
  };

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title={hasContent ? "重新生成剧本与分镜" : "生成剧本与分镜"}
      subtitle="一套结构化生成：故事圣经 → 角色 → 分集 → 场景 → 镜头。已有角色会按名字复用，不会覆盖你锁定的外观。"
      wide
      footer={
        <>
          <span className="modal__footnote">生成会作为后台任务运行，可随时切到其他页面。</span>
          <Button variant="ghost" onClick={() => setOpen(false)}>
            取消
          </Button>
          <Button variant="primary" icon={<Wand2 size={16} />} onClick={submit}>
            加入生成队列
          </Button>
        </>
      }
    >
      <div className="form-grid form-grid--one">
        <Field label="立意 / 故事核心">
          <textarea
            className="textarea"
            rows={3}
            value={premise}
            onChange={(event) => setPremise(event.target.value)}
          />
        </Field>
        <div className="form-grid form-grid--three">
          <Field label="类型">
            <input className="input" value={genre} onChange={(event) => setGenre(event.target.value)} />
          </Field>
          <Field label="气质">
            <input className="input" value={tone} onChange={(event) => setTone(event.target.value)} />
          </Field>
          <Field label="编剧模型">
            <select className="input" value={providerId} onChange={(event) => setProviderId(event.target.value)}>
              {llms.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.label}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <div className="form-grid form-grid--three">
          <Field label="集数">
            <input
              type="number"
              className="input"
              min={1}
              max={12}
              value={episodes}
              onChange={(event) => setEpisodes(Number(event.target.value))}
            />
          </Field>
          <Field label="每集场数">
            <input
              type="number"
              className="input"
              min={1}
              max={8}
              value={scenes}
              onChange={(event) => setScenes(Number(event.target.value))}
            />
          </Field>
          <Field label="每场镜头">
            <input
              type="number"
              className="input"
              min={1}
              max={10}
              value={shots}
              onChange={(event) => setShots(Number(event.target.value))}
            />
          </Field>
        </div>
        <Field label="额外风格要求" hint="例如：雨夜霓虹、低机位、角色不换装、每集 60 秒">
          <textarea className="textarea" rows={2} value={notes} onChange={(event) => setNotes(event.target.value)} />
        </Field>
        {hasContent ? (
          <label className="switch-row">
            <input type="checkbox" checked={replace} onChange={(event) => setReplace(event.target.checked)} />
            <span>
              <strong>替换现有分集与分镜</strong>
              <small>现有素材会保留在素材库，时间线切点会被清空。角色外观与参考图会保留。</small>
            </span>
          </label>
        ) : null}
      </div>
    </Modal>
  );
}

export function NovelAdaptDialog() {
  const open = useAppStore((s) => s.showNovelDialog);
  const setOpen = useAppStore((s) => s.setShowNovelDialog);
  const snapshot = useAppStore((s) => s.snapshot);
  const settings = useAppStore((s) => s.settings);
  const startAdaptation = useAppStore((s) => s.startNovelAdaptation);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [providerId, setProviderId] = useState("");
  const [maxEpisodes, setMaxEpisodes] = useState(0);
  const [notes, setNotes] = useState("");
  const [replace, setReplace] = useState(true);
  const [plan, setPlan] = useState<NovelPlan | null>(null);
  const [planning, setPlanning] = useState(false);
  const [importing, setImporting] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!open) return;
    setTitle("");
    setText("");
    setProviderId(settings?.defaults.llm ?? "");
    setMaxEpisodes(0);
    setNotes("");
    setReplace(true);
    setPlan(null);
    setError("");
  }, [open, settings?.defaults.llm]);

  useEffect(() => {
    if (!open) return;
    if (text.trim().length < 50) {
      setPlan(null);
      return;
    }
    const timer = setTimeout(async () => {
      setPlanning(true);
      try {
        const next = await invoke<NovelPlan>("preview_novel", {
          input: { title, text, sourcePath: null },
        });
        setPlan(next);
      } catch {
        setPlan(null);
      } finally {
        setPlanning(false);
      }
    }, 450);
    return () => clearTimeout(timer);
  }, [open, text, title]);

  if (!snapshot) return null;
  const llms = settings?.providers.filter((provider) => provider.kind === "llm") ?? [];

  const applyText = (content: string, name?: string) => {
    setText(content);
    if (!title.trim() && name) {
      setTitle(name.replace(/\.[^.]+$/, ""));
    }
  };

  const importNovel = async () => {
    if (!isDesktop) {
      fileInputRef.current?.click();
      return;
    }
    setImporting(true);
    try {
      const path = await pickNovelFile();
      if (!path) return;
      const content = await invoke<string>("read_novel_file", { path });
      applyText(content, path.split(/[\\/]/).pop());
    } catch (err) {
      setError(formatError(err));
    } finally {
      setImporting(false);
    }
  };

  const submit = async () => {
    if (text.trim().length < 50) {
      setError("先粘贴或导入正文，至少 50 个字符。");
      return;
    }
    setSubmitting(true);
    setError("");
    try {
      await startAdaptation(
        { title: title.trim(), text, sourcePath: null },
        {
          providerId: providerId || null,
          model: null,
          maxEpisodes,
          styleNotes: notes,
          replaceExisting: replace,
        },
      );
    } catch (err) {
      setError(formatError(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title="把整本小说改编成漫剧"
      subtitle="章节切分 → 分块理解 → 角色合并 → 分集、场景与分镜。长篇会自动分块，Mock 模型也能离线跑通。"
      wide
      footer={
        <>
          <span className="modal__footnote">单次最多 120 个分析块；超长小说请分卷。</span>
          <Button variant="ghost" onClick={() => setOpen(false)}>
            取消
          </Button>
          <Button
            variant="primary"
            icon={<BookOpen size={16} />}
            loading={submitting}
            disabled={planning || !text.trim()}
            onClick={() => void submit()}
          >
            开始改编
          </Button>
        </>
      }
    >
      <input
        ref={fileInputRef}
        type="file"
        accept=".txt,.md,.text"
        hidden
        onChange={async (event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (!file) return;
          applyText(await file.text(), file.name);
        }}
      />
      <div className="form-grid form-grid--two">
        <Field label="小说标题">
          <input
            className="input"
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="例如：雾港来信"
          />
        </Field>
        <Field label="导入文件">
          <Button variant="secondary" icon={<Upload size={15} />} loading={importing} onClick={() => void importNovel()}>
            选择 TXT / MD
          </Button>
        </Field>
      </div>
      <Field label="小说正文">
        <textarea
          className="textarea novel-textarea"
          rows={10}
          value={text}
          onChange={(event) => setText(event.target.value)}
          placeholder="粘贴完整小说。支持“第X章 / Chapter N”标题；没有标题会自动按段落切块。"
        />
      </Field>
      <div className="novel-plan">
        <span><strong>{text.trim().length.toLocaleString()}</strong> 字</span>
        <span><strong>{planning ? "…" : plan?.chapterCount ?? 0}</strong> 章</span>
        <span><strong>{planning ? "…" : plan?.chunkCount ?? 0}</strong> 个分析块</span>
        <span><strong>{planning ? "…" : plan?.estimatedCalls ?? 0}</strong> 次模型调用</span>
      </div>
      <div className="form-grid form-grid--three">
        <Field label="编剧模型">
          <select className="input" value={providerId} onChange={(event) => setProviderId(event.target.value)}>
            {llms.map((provider) => (
              <option key={provider.id} value={provider.id}>
                {provider.label}
              </option>
            ))}
          </select>
        </Field>
        <Field label="最大集数">
          <input
            type="number"
            className="input"
            min={0}
            max={60}
            value={maxEpisodes}
            onChange={(event) => setMaxEpisodes(Number(event.target.value))}
          />
        </Field>
        <label className="switch-row switch-row--compact switch-row--vertical">
          <input type="checkbox" checked={replace} onChange={(event) => setReplace(event.target.checked)} />
          <span>
            <strong>替换现有剧本</strong>
            <small>角色按名字复用</small>
          </span>
        </label>
      </div>
      <Field label="改编备注（可选）">
        <textarea
          className="textarea"
          rows={2}
          value={notes}
          onChange={(event) => setNotes(event.target.value)}
          placeholder="例如：保留原著对白，突出雨夜氛围，每集结尾留钩子。"
        />
      </Field>
      {error ? <InlineNotice tone="warn">{error}</InlineNotice> : null}
      <InlineNotice tone="info">
        正文会复制进当前项目；每 6,000 字左右调用一次模型，角色名与锁定特征会跨章节合并。
      </InlineNotice>
    </Modal>
  );
}
