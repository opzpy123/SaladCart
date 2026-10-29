import { useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  AudioLines,
  Check,
  Clock3,
  Download,
  FileJson,
  Film,
  Mic,
  Music,
  Pause,
  Play,
  Plus,
  Scissors,
  Search,
  Sparkles,
  Trash2,
  Volume2,
  Wand2,
  Waves,
} from "lucide-react";
import { assetUrl, formatDuration, formatTimecode, relativeTime } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { MusicTrack, Timeline, TimelineClip, TimelineClipInput } from "@/lib/types";
import {
  Button,
  EmptyState,
  Field,
  IconButton,
  InlineNotice,
  SectionHeader,
  Segmented,
  StatusPill,
  cx,
} from "@/components/ui";
import { Editable } from "@/components/Editable";

type InspectorTab = "clip" | "subs" | "music" | "export";

const FORMATS = [
  { id: "9:16", label: "竖屏 9:16", width: 1080, height: 1920 },
  { id: "16:9", label: "横屏 16:9", width: 1920, height: 1080 },
  { id: "1:1", label: "方形 1:1", width: 1080, height: 1080 },
  { id: "4:5", label: "竖版 4:5", width: 1080, height: 1350 },
];

function toInputs(clips: TimelineClip[]): TimelineClipInput[] {
  return clips.map((clip) => ({
    shotId: clip.shotId,
    assetId: clip.assetId,
    inMs: clip.inMs,
    outMs: clip.outMs,
    label: clip.label,
    muted: clip.muted,
    transition: clip.transition,
  }));
}

export function EditorScreen() {
  const snapshot = useAppStore((s) => s.snapshot);
  const updateTimeline = useAppStore((s) => s.updateTimeline);
  const autoAssemble = useAppStore((s) => s.autoAssemble);
  const autoBeatSync = useAppStore((s) => s.autoBeatSync);
  const analyzeBeats = useAppStore((s) => s.analyzeBeats);
  const searchMusic = useAppStore((s) => s.searchMusic);
  const importMusic = useAppStore((s) => s.importMusicTrack);
  const generateMusic = useAppStore((s) => s.generateMusic);
  const attachMusic = useAppStore((s) => s.attachMusic);
  const exportTimeline = useAppStore((s) => s.exportTimeline);
  const exportProvenance = useAppStore((s) => s.exportProvenance);
  const settings = useAppStore((s) => s.settings);
  const musicTracks = useAppStore((s) => s.musicTracks);
  const musicSearching = useAppStore((s) => s.musicSearching);
  const musicAnalysis = useAppStore((s) => s.musicAnalysis);
  const musicGenerating = useAppStore((s) => s.musicGenerating);

  const [index, setIndex] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);
  const [tab, setTab] = useState<InspectorTab>("clip");
  const [search, setSearch] = useState("");
  const [musicPrompt, setMusicPrompt] = useState("雨夜、悬疑、克制的钢琴与低频脉冲");
  const [musicDuration, setMusicDuration] = useState(24);
  const [instrumental, setInstrumental] = useState(true);
  const [musicProvider, setMusicProvider] = useState("mock-music");
  const [approvedOnly, setApprovedOnly] = useState(false);
  const musicProviders = useMemo(
    () =>
      (settings?.providers ?? []).filter(
        (provider) => provider.kind === "music" && provider.id !== "jamendo-music",
      ),
    [settings],
  );
  const [exportName, setExportName] = useState("");
  const [burnSubtitles, setBurnSubtitles] = useState(true);
  const [fps, setFps] = useState(30);
  const [volume, setVolume] = useState(0.8);
  const videoRef = useRef<HTMLVideoElement>(null);
  const audioRef = useRef<HTMLAudioElement>(null);
  const timeline = snapshot?.timeline;

  const clips = timeline?.clips ?? [];
  const selected = clips[index] ?? null;
  const selectedAsset = snapshot?.assets.find((a) => a.id === selected?.assetId) ?? null;
  const musicAsset = snapshot?.assets.find((a) => a.id === timeline?.musicAssetId) ?? null;
  const audioAssets = snapshot?.assets.filter((a) => a.kind === "audio") ?? [];
  const projectName = snapshot?.project.name ?? "SaladCart";
  const totalMs = clips.reduce((sum, clip) => sum + clip.durationMs, 0);
  const allProjectShots = snapshot?.episodes.flatMap((e) => e.scenes.flatMap((s) => s.shots)) ?? [];
  const voiceClips = useMemo(() => {
    const result: Array<{ start: number; duration: number; label: string; assetId: string }> = [];
    let cursor = 0;
    for (const clip of clips) {
      if (clip.shotId) {
        const shot = allProjectShots.find((item) => item.id === clip.shotId);
        const asset = shot?.voiceAssetId
          ? snapshot?.assets.find((item) => item.id === shot.voiceAssetId)
          : undefined;
        if (shot && asset) {
          result.push({
            start: cursor,
            duration: asset.durationMs ?? shot.durationMs,
            label: `${shot.title} · ${asset.name}`,
            assetId: asset.id,
          });
        }
      }
      cursor += clip.durationMs;
    }
    return result;
  }, [clips, allProjectShots, snapshot?.assets]);
  const durationSec = Math.max(1, Math.ceil(totalMs / 1000));
  const pxPerSec = 26;
  const trackWidth = Math.max(760, durationSec * pxPerSec);

  const startOfClip = useMemo(() => {
    return clips.slice(0, index).reduce((sum, clip) => sum + clip.durationMs, 0);
  }, [clips, index]);

  useEffect(() => {
    if (index > clips.length - 1) setIndex(Math.max(0, clips.length - 1));
  }, [clips.length, index]);

  useEffect(() => {
    if (timeline?.musicVolume !== undefined) setVolume(timeline.musicVolume);
  }, [timeline?.musicVolume]);

  useEffect(() => {
    setExportName(`${projectName}-${timeline?.aspectRatio ?? "9x16"}-${new Date().toISOString().slice(0, 10)}.mp4`);
  }, [projectName, timeline?.aspectRatio]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !selected) return;
    const handleTime = () => {
      const absolute = startOfClip + Math.max(0, video.currentTime * 1000 - selected.inMs);
      setCurrentTime(absolute);
      if (video.currentTime * 1000 >= selected.outMs + 30) {
        if (index < clips.length - 1) setIndex(index + 1);
        else video.pause();
      }
    };
    video.addEventListener("timeupdate", handleTime);
    return () => video.removeEventListener("timeupdate", handleTime);
  }, [selected, startOfClip, index, clips.length]);

  if (!snapshot || !timeline) return null;

  const activeSubtitle = timeline.subtitles.find(
    (subtitle) => currentTime >= subtitle.startMs && currentTime <= subtitle.endMs,
  );

  const patchClips = (next: TimelineClip[], extra: Partial<Timeline> = {}) =>
    void updateTimeline({ clips: toInputs(next), ...extra });

  const moveClip = (direction: -1 | 1) => {
    if (!selected) return;
    const target = index + direction;
    if (target < 0 || target >= clips.length) return;
    const next = [...clips];
    [next[index], next[target]] = [next[target], next[index]];
    setIndex(target);
    patchClips(next);
  };

  const deleteClip = () => {
    if (!selected) return;
    const next = clips.filter((_, i) => i !== index);
    setIndex(Math.max(0, index - 1));
    patchClips(next);
  };

  const trimClip = (inMs: number, outMs: number) => {
    if (!selected) return;
    const next = clips.map((clip, i) =>
      i === index ? { ...clip, inMs, outMs, durationMs: Math.max(1, outMs - inMs) } : clip,
    );
    patchClips(next);
  };

  const changeFormat = (id: string) => {
    const format = FORMATS.find((f) => f.id === id);
    if (!format) return;
    void updateTimeline({
      aspectRatio: format.id,
      width: format.width,
      height: format.height,
      fps,
    });
  };

  const applyMusic = (assetId: string) => void attachMusic(assetId, volume);

  const saveVolume = () => void updateTimeline({ musicVolume: volume });

  const exportNow = () =>
    void exportTimeline({
      width: timeline.width,
      height: timeline.height,
      fps,
      burnSubtitles,
      musicAssetId: timeline.musicAssetId ?? null,
      filename: exportName,
    });

  return (
    <div className="editor-page">
      <header className="workspace-head editor-head">
        <div>
          <p className="eyebrow">EDIT / 剪辑与交付</p>
          <h1>让切点落在鼓点上。</h1>
          <p className="workspace-head__sub">
            粗剪、字幕、配乐与卡点都在这里完成；导出在后台渲染，不遮挡创作。
          </p>
        </div>
        <div className="workspace-head__actions">
          <select className="input input--compact" value={timeline.aspectRatio} onChange={(e) => changeFormat(e.target.value)}>
            {FORMATS.map((format) => (
              <option key={format.id} value={format.id}>
                {format.label}
              </option>
            ))}
          </select>
          <label className="toolbar-check" title="只装配已通过审核的镜头">
            <input
              type="checkbox"
              checked={approvedOnly}
              onChange={(e) => setApprovedOnly(e.target.checked)}
            />
            <span>仅已通过</span>
          </label>
          <Button variant="secondary" icon={<Scissors size={15} />} onClick={() => void autoAssemble(approvedOnly)}>
            重新装配
          </Button>
          <Button
            variant="secondary"
            icon={<Waves size={15} />}
            onClick={() => void autoBeatSync()}
            disabled={!timeline.beatGridMs?.length}
            title={!timeline.beatGridMs?.length ? "先在右侧分析配乐节拍" : "让切点对齐节拍"}
          >
            自动卡点
          </Button>
          <Button variant="primary" icon={<Download size={15} />} onClick={exportNow} disabled={!clips.length}>
            导出 MP4
          </Button>
        </div>
      </header>

      <div className="editor-top">
        <div className="editor-stage-wrap">
          <div
            className="editor-stage"
            style={{ aspectRatio: `${timeline.width} / ${timeline.height}` }}
          >
            {selectedAsset ? (
              selectedAsset.kind === "video" ? (
                <video
                  key={selected?.id}
                  ref={videoRef}
                  src={assetUrl(selectedAsset.absPath)}
                  poster={assetUrl(selectedAsset.thumbPath)}
                  controls
                  preload="metadata"
                  onLoadedMetadata={(event) => {
                    if (selected && selected.inMs > 0) event.currentTarget.currentTime = selected.inMs / 1000;
                  }}
                  onEnded={() => {
                    if (index < clips.length - 1) setIndex(index + 1);
                  }}
                />
              ) : (
                <img src={assetUrl(selectedAsset.absPath)} alt={selected?.label ?? ""} />
              )
            ) : (
              <EmptyState
                icon={<Film size={26} />}
                title="时间线是空的"
                description="先为分镜生成视频，再一键按剧本顺序装配粗剪。"
                action={
                  <Button variant="primary" icon={<Scissors size={15} />} onClick={() => void autoAssemble()}>
                    自动装配
                  </Button>
                }
              />
            )}
            {activeSubtitle ? <div className="subtitle-overlay">{activeSubtitle.text}</div> : null}
            <div className="stage-hud">
              <span>{selected?.label ?? "未选择片段"}</span>
              <span>{formatTimecode(currentTime)} / {formatTimecode(totalMs)}</span>
            </div>
          </div>
          <div className="stage-strip">
            <span><i className="dot dot--green" /> {clips.length} 个片段</span>
            <span>{formatDuration(totalMs)}</span>
            <span><Mic size={13} /> {voiceClips.length} 条配音</span>
            <span>{timeline.width}×{timeline.height} · {timeline.fps}fps</span>
            <span className={timeline.musicAssetId ? "" : "muted"}>
              <Music size={13} /> {musicAsset?.name ?? "无配乐"}
            </span>
          </div>
        </div>

        <aside className="panel editor-inspector">
          <Segmented
            size="sm"
            value={tab}
            onChange={(value) => setTab(value)}
            options={[
              { value: "clip", label: "片段" },
              { value: "subs", label: "字幕" },
              { value: "music", label: "配乐" },
              { value: "export", label: "导出" },
            ]}
          />

          {tab === "clip" ? (
            selected ? (
              <div className="editor-tab">
                <SectionHeader title="片段属性" subtitle={selected.label} />
                <div className="mini-preview">
                  {selectedAsset ? (
                    selectedAsset.kind === "video" ? (
                      <video src={assetUrl(selectedAsset.absPath)} muted />
                    ) : (
                      <img src={assetUrl(selectedAsset.absPath)} alt="" />
                    )
                  ) : null}
                </div>
                <div className="form-grid form-grid--two">
                  <Field label="入点（ms）">
                    <input
                      className="input"
                      type="number"
                      value={selected.inMs}
                      onChange={(e) => trimClip(Number(e.target.value), selected.outMs)}
                    />
                  </Field>
                  <Field label="出点（ms）">
                    <input
                      className="input"
                      type="number"
                      value={selected.outMs}
                      onChange={(e) => trimClip(selected.inMs, Number(e.target.value))}
                    />
                  </Field>
                </div>
                <Field label="片段名">
                  <Editable
                    value={selected.label}
                    onSave={(label) =>
                      patchClips(clips.map((clip, i) => (i === index ? { ...clip, label } : clip)))
                    }
                  />
                </Field>
                <label className="switch-row switch-row--compact">
                  <input
                    type="checkbox"
                    checked={selected.muted}
                    onChange={(event) =>
                      patchClips(clips.map((clip, i) => (i === index ? { ...clip, muted: event.target.checked } : clip)))
                    }
                  />
                  <span>
                    <strong>静音原片段</strong>
                    <small>保留画面，仅使用配乐声轨</small>
                  </span>
                </label>
                <div className="trim-actions">
                  <Button size="sm" variant="secondary" icon={<ArrowUp size={13} />} onClick={() => moveClip(-1)}>
                    前移
                  </Button>
                  <Button size="sm" variant="secondary" icon={<ArrowDown size={13} />} onClick={() => moveClip(1)}>
                    后移
                  </Button>
                  <Button size="sm" variant="ghost" icon={<Trash2 size={13} />} onClick={deleteClip}>
                    移除
                  </Button>
                </div>
                <InlineNotice tone="info">
                  {selectedAsset ? `${selectedAsset.width ?? "?"}×${selectedAsset.height ?? "?"} · 源时长 ${formatDuration(selectedAsset.durationMs)}` : "素材缺失"}
                </InlineNotice>
              </div>
            ) : (
              <EmptyState compact icon={<Film size={20} />} title="选择一个片段" description="在下方时间线点击片段，调整入出点与顺序。" />
            )
          ) : null}

          {tab === "subs" ? (
            <div className="editor-tab">
              <SectionHeader
                title="字幕轨"
                subtitle="导出时可烧录进画面"
                action={
                  <Button
                    size="sm"
                    variant="ghost"
                    icon={<Plus size={13} />}
                    onClick={() => {
                      const start = Math.round(currentTime);
                      void updateTimeline({
                        subtitles: [
                          ...timeline.subtitles.map((s) => ({ startMs: s.startMs, endMs: s.endMs, text: s.text })),
                          { startMs: start, endMs: start + 2400, text: "新字幕" },
                        ],
                      });
                    }}
                  >
                    添加
                  </Button>
                }
              />
              <div className="subtitle-list">
                {timeline.subtitles.map((subtitle, i) => (
                  <div className="subtitle-item" key={subtitle.id}>
                    <div className="subtitle-item__time">
                      <Editable
                        value={String(subtitle.startMs)}
                        onSave={(value) =>
                          void updateTimeline({
                            subtitles: timeline.subtitles.map((s, si) =>
                              si === i ? { startMs: Number(value), endMs: s.endMs, text: s.text } : { startMs: s.startMs, endMs: s.endMs, text: s.text },
                            ),
                          })
                        }
                        type="number"
                      />
                      <span>→</span>
                      <Editable
                        value={String(subtitle.endMs)}
                        onSave={(value) =>
                          void updateTimeline({
                            subtitles: timeline.subtitles.map((s, si) =>
                              si === i ? { startMs: s.startMs, endMs: Number(value), text: s.text } : { startMs: s.startMs, endMs: s.endMs, text: s.text },
                            ),
                          })
                        }
                        type="number"
                      />
                    </div>
                    <Editable
                      value={subtitle.text}
                      onSave={(value) =>
                        void updateTimeline({
                          subtitles: timeline.subtitles.map((s, si) =>
                            si === i ? { startMs: s.startMs, endMs: s.endMs, text: value } : { startMs: s.startMs, endMs: s.endMs, text: s.text },
                          ),
                        })
                      }
                      rows={2}
                    />
                    <IconButton
                      label="删除字幕"
                      onClick={() =>
                        void updateTimeline({
                          subtitles: timeline.subtitles
                            .filter((_, si) => si !== i)
                            .map((s) => ({ startMs: s.startMs, endMs: s.endMs, text: s.text })),
                        })
                      }
                    >
                      <Trash2 size={14} />
                    </IconButton>
                  </div>
                ))}
                {!timeline.subtitles.length ? (
                  <EmptyState compact icon={<Clock3 size={20} />} title="还没有字幕" description="自动装配时会从镜头对白生成，也可以手动添加。" />
                ) : null}
              </div>
            </div>
          ) : null}

          {tab === "music" ? (
            <div className="editor-tab">
              <SectionHeader title="配乐与卡点" subtitle="搜索正版曲库，或让 AI 写一段不撞曲的配乐" />
              <div className="music-current">
                <div className="music-current__art">
                  <AudioLines size={20} />
                </div>
                <div>
                  <strong>{musicAsset?.name ?? "还没有配乐"}</strong>
                  <small>
                    {timeline.bpm ? `${timeline.bpm} BPM · ${timeline.beatGridMs?.length ?? 0} 个节拍` : "分析节拍后可自动卡点"}
                  </small>
                </div>
                <StatusPill status={timeline.musicAssetId ? "ready" : "draft"} label={timeline.musicAssetId ? "已应用" : "未应用"} />
              </div>
              {musicAsset ? (
                <audio ref={audioRef} src={assetUrl(musicAsset.absPath)} controls className="audio-bar" />
              ) : null}
              <div className="volume-row">
                <Volume2 size={15} />
                <input
                  type="range"
                  min={0}
                  max={1.5}
                  step={0.05}
                  value={volume}
                  onChange={(e) => setVolume(Number(e.target.value))}
                  onMouseUp={saveVolume}
                  onTouchEnd={saveVolume}
                />
                <span>{Math.round(volume * 100)}%</span>
              </div>
              <div className="beat-actions">
                <Button size="sm" variant="secondary" icon={<Waves size={14} />} onClick={() => void analyzeBeats()} disabled={!musicAsset}>
                  分析节拍
                </Button>
                <Button size="sm" variant="soft" icon={<Scissors size={14} />} onClick={() => void autoBeatSync()} disabled={!timeline.beatGridMs?.length}>
                  自动卡点
                </Button>
              </div>
              {musicAnalysis ? (
                <div className="beat-viz">
                  {musicAnalysis.waveform.map((value, i) => (
                    <i key={i} style={{ height: `${Math.max(8, value * 100)}%` }} />
                  ))}
                  <div className="beat-viz__meta">
                    <span>{musicAnalysis.bpm} BPM</span>
                    <span>{Math.round(musicAnalysis.confidence * 100)}% 置信度</span>
                  </div>
                </div>
              ) : (
                <div className="beat-viz beat-viz--idle">
                  {Array.from({ length: 64 }).map((_, i) => (
                    <i key={i} style={{ height: `${20 + Math.abs(Math.sin(i * 0.5)) * 55}%` }} />
                  ))}
                  <span className="beat-viz__hint">等待节拍分析</span>
                </div>
              )}

              <div className="music-section">
                <div className="music-search">
                  <Search size={15} />
                  <input
                    className="input"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && void searchMusic(search)}
                    placeholder="搜索正版曲库：雨夜 / 悬疑 / 钢琴…"
                  />
                  <Button size="sm" variant="secondary" loading={musicSearching} onClick={() => void searchMusic(search)}>
                    搜索
                  </Button>
                </div>
                {musicTracks.length ? (
                  <div className="track-list">
                    {musicTracks.map((track) => (
                      <TrackRow key={track.id} track={track} onUse={() => void importMusic(track)} />
                    ))}
                  </div>
                ) : (
                  <p className="muted small">Jamendo 搜索需要 Client ID；没有配置时也可以用下方 AI 生成。</p>
                )}
              </div>

              <div className="music-section music-generate">
                <p className="panel-label">AI 原创配乐</p>
                <Field label="音乐描述">
                  <textarea className="textarea" rows={2} value={musicPrompt} onChange={(e) => setMusicPrompt(e.target.value)} />
                </Field>
                <div className="form-grid form-grid--two">
                  <Field label="时长">
                    <select className="input" value={musicDuration} onChange={(e) => setMusicDuration(Number(e.target.value))}>
                      <option value={16}>16 秒</option>
                      <option value={24}>24 秒</option>
                      <option value={32}>32 秒</option>
                      <option value={60}>60 秒</option>
                    </select>
                  </Field>
                  <Field label="生成方式">
                    <select className="input" value={musicProvider} onChange={(e) => setMusicProvider(e.target.value)}>
                      {musicProviders.map((provider) => (
                        <option key={provider.id} value={provider.id}>
                          {provider.label}
                        </option>
                      ))}
                    </select>
                  </Field>
                </div>
                <label className="switch-row switch-row--compact switch-row--vertical">
                  <input type="checkbox" checked={instrumental} onChange={(e) => setInstrumental(e.target.checked)} />
                  <span>
                    <strong>纯音乐</strong>
                    <small>不生成人声</small>
                  </span>
                </label>
                <Button
                  variant="primary"
                  icon={<Sparkles size={15} />}
                  loading={musicGenerating}
                  onClick={() =>
                    void generateMusic({
                      prompt: musicPrompt,
                      durationMs: musicDuration * 1000,
                      instrumental,
                      providerId: musicProvider,
                      model: null,
                      attachToTimeline: true,
                    })
                  }
                >
                  生成并应用
                </Button>
                <p className="muted tiny">离线 Mock 会合成一段 100 BPM 的可卡点配乐；配置 Key 后可选 Stability / ElevenLabs / 火山引擎纯音乐。</p>
              </div>

              {audioAssets.length ? (
                <div className="music-section">
                  <p className="panel-label">项目音频库</p>
                  <div className="audio-library">
                    {audioAssets.map((asset) => (
                      <button
                        key={asset.id}
                        className={cx("audio-library__item", timeline.musicAssetId === asset.id && "is-active")}
                        onClick={() => applyMusic(asset.id)}
                      >
                        <Music size={14} />
                        <span>
                          <strong>{asset.name}</strong>
                          <small>{formatDuration(asset.durationMs)} · {asset.providerId ?? "本地"}</small>
                        </span>
                        {timeline.musicAssetId === asset.id ? <Check size={14} /> : <Plus size={14} />}
                      </button>
                    ))}
                  </div>
                </div>
              ) : null}
            </div>
          ) : null}

          {tab === "export" ? (
            <div className="editor-tab">
              <SectionHeader title="导出成片" subtitle="H.264 / AAC · MP4 · faststart" />
              <Field label="文件名">
                <input className="input" value={exportName} onChange={(e) => setExportName(e.target.value)} />
              </Field>
              <div className="form-grid form-grid--two">
                <Field label="画幅">
                  <select className="input" value={timeline.aspectRatio} onChange={(e) => changeFormat(e.target.value)}>
                    {FORMATS.map((f) => (
                      <option key={f.id} value={f.id}>{f.label}</option>
                    ))}
                  </select>
                </Field>
                <Field label="帧率">
                  <select className="input" value={fps} onChange={(e) => setFps(Number(e.target.value))}>
                    <option value={24}>24 fps</option>
                    <option value={30}>30 fps</option>
                    <option value={60}>60 fps</option>
                  </select>
                </Field>
              </div>
              <label className="switch-row switch-row--compact">
                <input type="checkbox" checked={burnSubtitles} onChange={(e) => setBurnSubtitles(e.target.checked)} />
                <span>
                  <strong>烧录字幕</strong>
                  <small>英文/中文自动换行，底部安全区</small>
                </span>
              </label>
              <div className="export-summary">
                <div><span>分辨率</span><strong>{timeline.width}×{timeline.height}</strong></div>
                <div><span>时长</span><strong>{formatDuration(totalMs)}</strong></div>
                <div><span>片段</span><strong>{clips.length}</strong></div>
                <div><span>配乐</span><strong>{musicAsset ? "有" : "无"}</strong></div>
              </div>
              <Button variant="primary" icon={<Download size={15} />} onClick={exportNow} disabled={!clips.length}>
                开始渲染
              </Button>
              <Button variant="secondary" icon={<FileJson size={15} />} onClick={() => void exportProvenance()}>
                导出溯源清单
              </Button>
              <p className="muted tiny">
                渲染使用本机 FFmpeg。{settings?.ffmpegPath ? `已发现：${settings.ffmpegPath}` : "若未安装 FFmpeg，请先到设置中配置。"}
              </p>
              {snapshot.jobs
                .filter((job) => job.kind === "export")
                .slice(0, 3)
                .map((job) => (
                  <div className="export-history" key={job.id}>
                    <div>
                      <strong>{job.requestSummary}</strong>
                      <small>{relativeTime(job.createdAt)} · {job.providerLabel}</small>
                    </div>
                    <StatusPill status={job.status} />
                  </div>
                ))}
            </div>
          ) : null}
        </aside>
      </div>

      <section className="panel timeline-panel">
        <div className="timeline-head">
          <div>
            <p className="eyebrow">TIMELINE</p>
            <h3>{timeline.name}</h3>
          </div>
          <div className="timeline-head__meta">
            <span>{formatDuration(totalMs)}</span>
            <span>{clips.length} clips</span>
            <span>{timeline.bpm ? `♩ ${timeline.bpm}` : "未分析 BPM"}</span>
          </div>
        </div>
        <div className="timeline-scroll">
          <div className="timeline-canvas" style={{ width: trackWidth }}>
            <div className="timeline-ruler">
              {Array.from({ length: Math.floor(durationSec / 5) + 1 }).map((_, i) => (
                <span key={i} style={{ left: i * 5 * pxPerSec }}>
                  {String(Math.floor((i * 5) / 60)).padStart(2, "0")}:{String((i * 5) % 60).padStart(2, "0")}
                </span>
              ))}
            </div>
            <div className="timeline-track">
              <div className="track-head">V1</div>
              <div className="track-lane">
                {clips.map((clip, i) => {
                  const asset = snapshot.assets.find((a) => a.id === clip.assetId);
                  return (
                    <button
                      key={clip.id}
                      className={cx("timeline-clip", index === i && "is-active")}
                      style={{ width: Math.max(54, (clip.durationMs / 1000) * pxPerSec) }}
                      onClick={() => {
                        setIndex(i);
                        setTab("clip");
                      }}
                    >
                      {asset?.thumbPath ? <img src={assetUrl(asset.thumbPath)} alt="" /> : null}
                      <span className="timeline-clip__scrim" />
                      <span className="timeline-clip__label">{clip.label}</span>
                      <span className="timeline-clip__time">{(clip.durationMs / 1000).toFixed(1)}s</span>
                    </button>
                  );
                })}
                {!clips.length ? <div className="timeline-empty">还没有片段 · 从“重新装配”开始</div> : null}
              </div>
            </div>
            <div className="timeline-track timeline-track--audio">
              <div className="track-head">A1</div>
              <div className="track-lane">
                <div className={cx("music-clip", !musicAsset && "is-empty")}>
                  {musicAsset ? (
                    <>
                      <span className="music-clip__name"><Music size={12} /> {musicAsset.name}</span>
                      <div className="waveform-mini">
                        {Array.from({ length: Math.max(12, Math.floor(trackWidth / 9)) }).map((_, i) => (
                          <i
                            key={i}
                            style={{
                              height: `${18 + Math.abs(Math.sin(i * 0.55) * Math.cos(i * 0.17)) * 60}%`,
                            }}
                          />
                        ))}
                      </div>
                      {(timeline.beatGridMs ?? []).map((beat, i) =>
                        beat / 1000 * pxPerSec <= trackWidth ? (
                          <span key={i} className="beat-marker" style={{ left: (beat / 1000) * pxPerSec }} />
                        ) : null,
                      )}
                    </>
                  ) : (
                    <span className="music-clip__empty">无配乐 · 在右侧搜索或生成</span>
                  )}
                </div>
              </div>
            </div>
            <div className="timeline-track timeline-track--voice">
              <div className="track-head">V1</div>
              <div className="track-lane">
                {voiceClips.map((clip, i) => (
                  <span
                    key={`${clip.assetId}-${i}`}
                    className="voice-chip"
                    style={{
                      left: (clip.start / 1000) * pxPerSec,
                      width: Math.max(60, (clip.duration / 1000) * pxPerSec),
                    }}
                    title={clip.label}
                  >
                    <Mic size={10} />
                    {clip.label}
                  </span>
                ))}
                {!voiceClips.length ? (
                  <div className="timeline-empty">暂无配音 · 在分镜工坊点击「全部配音」</div>
                ) : null}
              </div>
            </div>
            <div className="timeline-track timeline-track--subs">
              <div className="track-head">S1</div>
              <div className="track-lane">
                {timeline.subtitles.map((subtitle) => (
                  <span
                    key={subtitle.id}
                    className="subtitle-chip"
                    style={{
                      left: (subtitle.startMs / 1000) * pxPerSec,
                      width: Math.max(40, ((subtitle.endMs - subtitle.startMs) / 1000) * pxPerSec),
                    }}
                    title={subtitle.text}
                  >
                    {subtitle.text}
                  </span>
                ))}
              </div>
            </div>
            <div
              className="playhead"
              style={{ left: (Math.min(totalMs, currentTime) / 1000) * pxPerSec }}
            />
          </div>
        </div>
        <div className="timeline-foot">
          <span><Play size={12} /> Space 播放（使用原生控件）</span>
          <span><Scissors size={12} /> 点击片段编辑入出点</span>
          <span><Wand2 size={12} /> 节拍网格 {timeline.beatGridMs?.length ?? 0} 个点</span>
        </div>
      </section>
    </div>
  );
}

function TrackRow({ track, onUse }: { track: MusicTrack; onUse: () => void }) {
  const [playing, setPlaying] = useState(false);
  const audio = useRef<HTMLAudioElement | null>(null);
  const toggle = () => {
    if (!audio.current) {
      audio.current = new Audio(assetUrl(track.previewUrl));
      audio.current.onended = () => setPlaying(false);
    }
    if (playing) audio.current.pause();
    else void audio.current.play();
    setPlaying(!playing);
  };
  return (
    <div className="track-row">
      <button className="track-row__play" onClick={toggle} aria-label={playing ? "暂停" : "试听"}>
        {playing ? <Pause size={14} /> : <Play size={14} />}
      </button>
      <div className="track-row__body">
        <strong>{track.title}</strong>
        <small>{track.artist} · {formatDuration(track.durationMs)} · {track.license}</small>
      </div>
      <button className="track-row__use" onClick={onUse}>
        使用
      </button>
    </div>
  );
}
