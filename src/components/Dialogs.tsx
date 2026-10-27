import { useEffect, useState } from "react";
import { Clapperboard, Sparkles, Wand2 } from "lucide-react";
import { Button, Field, InlineNotice, Modal } from "./ui";
import { useAppStore } from "@/store/useAppStore";
import type { GenerateScriptRequest } from "@/lib/types";

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
      setError("");
    }
  }, [open]);

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
    );
  };

  const llm = settings?.providers.find((p) => p.id === settings.defaults.llm);
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
              <small>Mock 编剧无需 API Key，可立即跑通全流程</small>
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
    setProviderId(settings?.defaults.llm ?? "");
  }, [open, snapshot, settings]);

  if (!snapshot) return null;
  const llms = settings?.providers.filter((p) => p.kind === "llm") ?? [];
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
