import { useMemo, useState } from "react";
import {
  Check,
  ChevronRight,
  Clapperboard,
  Film,
  Image as ImageIcon,
  Layers,
  Mic,
  Play,
  SlidersHorizontal,
  Sparkles,
} from "lucide-react";
import { assetUrl, formatDuration } from "@/lib/bridge";
import { useAppStore, selectAllShots } from "@/store/useAppStore";
import type { Shot } from "@/lib/types";
import {
  Button,
  EmptyState,
  Progress,
  SectionHeader,
  Segmented,
  StatusPill,
  cx,
} from "@/components/ui";

type Filter = "all" | "needs_review" | "draft" | "approved";

export function ShotStudio() {
  const snapshot = useAppStore((s) => s.snapshot);
  const selected = useAppStore((s) => s.selectedShotIds);
  const toggle = useAppStore((s) => s.toggleShotSelection);
  const setSelection = useAppStore((s) => s.setShotSelection);
  const generateShot = useAppStore((s) => s.generateShot);
  const generateVoices = useAppStore((s) => s.generateVoices);
  const batchGenerate = useAppStore((s) => s.batchGenerate);
  const openContext = useAppStore((s) => s.openShotContext);
  const selectShot = useAppStore((s) => s.selectShot);
  const setRoute = useAppStore((s) => s.setRoute);
  const autoAssemble = useAppStore((s) => s.autoAssemble);
  const [filter, setFilter] = useState<Filter>("all");
  const [continuity, setContinuity] = useState(true);

  const shots = useMemo(() => (snapshot ? selectAllShots(snapshot) : []), [snapshot]);
  const filtered = shots.filter((shot) => filter === "all" || shot.status === filter);
  const videos = shots.filter((shot) => shot.videoAssetId).length;
  const reviewing = shots.filter((shot) => shot.status === "needs_review").length;
  const dialogueShots = shots.filter((shot) => shot.dialogue.trim()).map((shot) => shot.id);
  const selectedDialogueShots = selected.filter((id) =>
    shots.find((shot) => shot.id === id)?.dialogue.trim(),
  );

  if (!snapshot) return null;

  const openInScript = (shot: Shot) => {
    selectShot(shot.id);
    setRoute("script");
  };

  return (
    <div className="studio-page">
      <header className="workspace-head">
        <div>
          <p className="eyebrow">SHOTS / 分镜与生成</p>
          <h1>先预览，再花钱。</h1>
          <p className="workspace-head__sub">
            每个镜头都能看到最终发送的上下文、角色参考和成本。批量抽卡不再盲盒。
          </p>
        </div>
        <div className="workspace-head__actions">
          <div className="head-stat">
            <span>镜头</span>
            <strong>{shots.length}</strong>
          </div>
          <div className="head-stat">
            <span>成片</span>
            <strong>{videos}/{shots.length}</strong>
          </div>
          <div className="head-stat">
            <span>待审</span>
            <strong>{reviewing}</strong>
          </div>
          <Button
            variant="secondary"
            icon={<Mic size={15} />}
            onClick={() => void generateVoices(dialogueShots)}
            disabled={!dialogueShots.length}
          >
            全部配音
          </Button>
          <Button variant="secondary" icon={<Clapperboard size={15} />} onClick={() => void autoAssemble()} disabled={!videos}>
            装配时间线
          </Button>
        </div>
      </header>

      <div className="shot-board">
        <div className="shot-board__toolbar">
          <Segmented
            size="sm"
            value={filter}
            onChange={(value) => setFilter(value)}
            options={[
              { value: "all", label: `全部 ${shots.length}` },
              { value: "needs_review", label: `待审 ${reviewing}` },
              { value: "draft", label: "草稿" },
              { value: "approved", label: "已通过" },
            ]}
          />
          <div className="shot-board__right">
            <label className="checkline">
              <input type="checkbox" checked={continuity} onChange={(e) => setContinuity(e.target.checked)} />
              批量开启尾帧续接
            </label>
            <span className="muted small">
              默认 {snapshot.project.aspectRatio} · 视频 {snapshot.assets.filter((a) => a.kind === "video").length} 条
            </span>
          </div>
        </div>

        {selected.length ? (
          <div className="batch-bar">
            <span className="batch-bar__count">{selected.length}</span>
            <div>
              <strong>已选择 {selected.length} 个镜头</strong>
              <small>按当前默认供应商批量排队，可随时取消。</small>
            </div>
            <div className="batch-bar__actions">
              <Button size="sm" variant="ghost" onClick={() => setSelection([])}>
                清空
              </Button>
              <Button
                size="sm"
                variant="secondary"
                icon={<ImageIcon size={14} />}
                onClick={() => void batchGenerate(selected, "image", continuity)}
              >
                批量画面
              </Button>
              <Button
                size="sm"
                variant="primary"
                icon={<Film size={14} />}
                onClick={() => void batchGenerate(selected, "video", continuity)}
              >
                批量视频
              </Button>
              <Button
                size="sm"
                variant="secondary"
                icon={<Mic size={14} />}
                onClick={() => void generateVoices(selectedDialogueShots)}
                disabled={!selectedDialogueShots.length}
              >
                批量配音
              </Button>
            </div>
          </div>
        ) : null}

        {filtered.length ? (
          <div className="shot-grid">
            {filtered.map((shot) => {
              const scene = snapshot.episodes
                .flatMap((e) => e.scenes)
                .find((s) => s.id === shot.sceneId);
              const asset = snapshot.assets.find((a) => a.id === (shot.videoAssetId ?? shot.imageAssetId));
              const cast = snapshot.characters.filter((c) => shot.characterIds.includes(c.id));
              const job = snapshot.jobs.find(
                (j) =>
                  j.targetId === shot.id &&
                  (j.status === "running" || j.status === "queued"),
              );
              const checked = selected.includes(shot.id);
              return (
                <article key={shot.id} className={cx("shot-card", checked && "is-selected")}>
                  <div
                    className="shot-card__visual"
                    onClick={() => openInScript(shot)}
                    role="button"
                    tabIndex={0}
                  >
                    {asset ? (
                      <img src={assetUrl(asset.thumbPath ?? asset.absPath)} alt={shot.title} loading="lazy" />
                    ) : (
                      <div className="shot-card__placeholder">
                        <ImageIcon size={24} />
                        <span>等待第一帧</span>
                      </div>
                    )}
                    <div className="shot-card__scrim" />
                    <button
                      className={cx("select-box", checked && "is-checked")}
                      onClick={(event) => {
                        event.stopPropagation();
                        toggle(shot.id);
                      }}
                      aria-label={checked ? "取消选择" : "选择镜头"}
                    >
                      {checked ? <Check size={13} /> : null}
                    </button>
                    <div className="shot-card__topright">
                      <StatusPill status={shot.status} />
                    </div>
                    {shot.videoAssetId ? (
                      <span className="shot-card__play"><Play size={17} fill="currentColor" /></span>
                    ) : null}
                    {job ? (
                      <div className="shot-card__progress">
                        <span><Sparkles size={12} className="spin" /> {job.stage}</span>
                        <Progress value={job.progress} />
                      </div>
                    ) : null}
                    <div className="shot-card__caption">
                      <span>{scene?.title ?? "场景"}</span>
                      <strong>{shot.title}</strong>
                    </div>
                  </div>
                  <div className="shot-card__body">
                    <div className="shot-card__meta">
                      <span>{shot.camera || "机位未定"}</span>
                      <span>{formatDuration(shot.durationMs)}</span>
                      <span>{cast.length ? cast.map((c) => c.name).join(" / ") : "无角色"}</span>
                    </div>
                    <p>{shot.description}</p>
                    <div className="shot-card__actions">
                      <Button size="sm" variant="ghost" icon={<SlidersHorizontal size={13} />} onClick={() => void openContext(shot.id, "image")}>
                        上下文
                      </Button>
                      <Button size="sm" variant="ghost" icon={<ImageIcon size={13} />} onClick={() => void generateShot(shot.id, "image")}>
                        画面
                      </Button>
                      <Button size="sm" variant="soft" icon={<Film size={13} />} onClick={() => void generateShot(shot.id, "video")}>
                        视频
                      </Button>
                      <button className="link-arrow" onClick={() => openInScript(shot)}>
                        详情 <ChevronRight size={13} />
                      </button>
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
        ) : (
          <EmptyState
            icon={<Layers size={26} />}
            title={filter === "all" ? "还没有分镜" : "这个筛选下没有镜头"}
            description="从剧本工坊生成分镜，或切换筛选条件。每个镜头包含机位、时长、角色和生成指令。"
            action={
              <Button variant="primary" icon={<Sparkles size={15} />} onClick={() => useAppStore.getState().setShowScriptDialog(true)}>
                生成分镜
              </Button>
            }
          />
        )}

        <div className="shot-board__foot">
          <SectionHeader
            title="批量生产提示"
            subtitle="先固定角色 seed 与定妆照，再用批量视频；失败的镜头可单独重试，不会阻塞其他任务。"
          />
          <div className="pipeline-chips">
            <span>1 · 锁定角色</span>
            <i />
            <span>2 · 生成关键帧</span>
            <i />
            <span>3 · 批量图生视频</span>
            <i />
            <span>4 · 自动卡点装配</span>
          </div>
        </div>
      </div>
    </div>
  );
}
