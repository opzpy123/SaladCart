import { useMemo } from "react";
import {
  ArrowRight,
  Clapperboard,
  Clock3,
  FileText,
  Film,
  Image as ImageIcon,
  Lock,
  Mic,
  Sparkles,
  Wand2,
} from "lucide-react";
import { assetUrl, formatDuration } from "@/lib/bridge";
import { useAppStore, selectAllShots, selectCurrentScene, selectCurrentShot } from "@/store/useAppStore";
import type { Shot, UpdateShotInput } from "@/lib/types";
import {
  Button,
  EmptyState,
  IconButton,
  SectionHeader,
  StatusPill,
  cx,
} from "@/components/ui";
import { Editable } from "@/components/Editable";

export function ScriptStudio() {
  const snapshot = useAppStore((s) => s.snapshot);
  const currentScene = useAppStore(selectCurrentScene);
  const currentShot = useAppStore(selectCurrentShot);
  const selectScene = useAppStore((s) => s.selectScene);
  const selectShot = useAppStore((s) => s.selectShot);
  const setScriptDialog = useAppStore((s) => s.setShowScriptDialog);
  const saveScene = useAppStore((s) => s.saveScene);
  const saveShot = useAppStore((s) => s.saveShot);
  const generateShot = useAppStore((s) => s.generateShot);
  const generateVoice = useAppStore((s) => s.generateVoice);
  const openContext = useAppStore((s) => s.openShotContext);
  const setShotStatus = useAppStore((s) => s.setShotStatus);
  const autoAssemble = useAppStore((s) => s.autoAssemble);

  const shots = useMemo(() => (snapshot ? selectAllShots(snapshot) : []), [snapshot]);
  const totalDuration = shots.reduce((sum, shot) => sum + shot.durationMs, 0);
  const rendered = shots.filter((shot) => shot.videoAssetId).length;

  if (!snapshot) return null;

  return (
    <div className="studio-page">
      <header className="workspace-head">
        <div>
          <p className="eyebrow">SCRIPT / 剧本工坊</p>
          <h1>{snapshot.project.name}</h1>
          <p className="workspace-head__sub">
            {snapshot.project.logline || "从一个能拍成镜头的故事开始。"}
          </p>
        </div>
        <div className="workspace-head__actions">
          <div className="head-stat">
            <span>分镜</span>
            <strong>{shots.length}</strong>
          </div>
          <div className="head-stat">
            <span>成片</span>
            <strong>
              {rendered}/{shots.length}
            </strong>
          </div>
          <Button variant="secondary" icon={<Wand2 size={15} />} onClick={() => setScriptDialog(true)}>
            AI 生成 / 重写
          </Button>
          <Button
            variant="primary"
            icon={<Clapperboard size={15} />}
            onClick={() => void autoAssemble()}
            disabled={!rendered}
            title={!rendered ? "先为镜头生成视频" : "按剧本顺序装配时间线"}
          >
            装配时间线
          </Button>
        </div>
      </header>

      <div className="script-layout">
        <aside className="panel scene-nav">
          <SectionHeader
            title="剧集与场景"
            subtitle={`${snapshot.episodes.length} 集 · ${snapshot.episodes.reduce((n, e) => n + e.scenes.length, 0)} 场`}
          />
          <div className="scene-nav__list">
            {snapshot.episodes.map((episode) => (
              <div key={episode.id} className="episode-block">
                <div className="episode-block__title">
                  <span>{String(episode.orderIndex + 1).padStart(2, "0")}</span>
                  <div>
                    <strong>{episode.title}</strong>
                    <small>{episode.scenes.length} 场 · {episode.summary}</small>
                  </div>
                </div>
                {episode.scenes.map((scene) => {
                  const sceneShots = scene.shots;
                  const videoCount = sceneShots.filter((s) => s.videoAssetId).length;
                  return (
                    <button
                      key={scene.id}
                      className={cx("scene-nav__item", currentScene?.id === scene.id && "is-active")}
                      onClick={() => selectScene(scene.id)}
                    >
                      <span className="scene-nav__index">{scene.orderIndex + 1}</span>
                      <span className="scene-nav__body">
                        <strong>{scene.title}</strong>
                        <small>
                          {scene.slugline || "未设置场景头"} · {sceneShots.length} 镜
                        </small>
                      </span>
                      <span className={cx("scene-nav__pip", videoCount === sceneShots.length && sceneShots.length > 0 && "is-ready")} />
                    </button>
                  );
                })}
              </div>
            ))}
            {!snapshot.episodes.length ? (
              <EmptyState
                compact
                icon={<FileText size={20} />}
                title="还没有剧本"
                description="让 AI 从一句立意拆出角色、场景和可生成的分镜。"
                action={
                  <Button variant="primary" size="sm" icon={<Sparkles size={14} />} onClick={() => setScriptDialog(true)}>
                    生成剧本
                  </Button>
                }
              />
            ) : null}
          </div>
          <div className="scene-nav__foot">
            <div className="mini-label">视觉风格</div>
            <p>{snapshot.project.styleBible.visualStyle}</p>
            <div className="palette-row">
              {snapshot.project.styleBible.palette
                .split(/[,，]/)
                .slice(0, 4)
                .map((color) => (
                  <span key={color}>{color.trim()}</span>
                ))}
            </div>
          </div>
        </aside>

        <main className="panel script-paper">
          {!currentScene ? (
            <EmptyState
              icon={<FileText size={24} />}
              title="先写一集"
              description="用 AI 生成结构化的分集、场景和镜头，后面每个镜头都会自动继承角色与风格。"
              action={
                <Button variant="primary" icon={<Sparkles size={15} />} onClick={() => setScriptDialog(true)}>
                  生成剧本与分镜
                </Button>
              }
            />
          ) : (
            <>
              <div className="script-paper__top">
                <div className="script-scene-index">
                  <span>SCENE</span>
                  <strong>{String(currentScene.orderIndex + 1).padStart(2, "0")}</strong>
                </div>
                <div className="script-scene-title">
                  <Editable
                    value={currentScene.title}
                    onSave={(value) => void saveScene(currentScene.id, { title: value })}
                    className="editable--title"
                    placeholder="场景标题"
                  />
                  <Editable
                    value={currentScene.slugline}
                    onSave={(value) => void saveScene(currentScene.id, { slugline: value })}
                    className="editable--slugline"
                    placeholder="EXT. 地点 - 时间"
                    mono
                  />
                </div>
                <StatusPill status={currentScene.status} />
              </div>

              <div className="script-meta">
                <label>
                  <span>地点</span>
                  <Editable
                    value={currentScene.location}
                    onSave={(value) => void saveScene(currentScene.id, { location: value })}
                  />
                </label>
                <label>
                  <span>时间</span>
                  <Editable
                    value={currentScene.timeOfDay}
                    onSave={(value) => void saveScene(currentScene.id, { timeOfDay: value })}
                  />
                </label>
                <div className="script-meta__stat">
                  <Clock3 size={14} />
                  {currentScene.shots.length} 镜 ·{" "}
                  {formatDuration(currentScene.shots.reduce((sum, s) => sum + s.durationMs, 0))}
                </div>
              </div>

              <div className="script-block">
                <div className="script-block__head">
                  <span>场景目标 / 情绪</span>
                  <span className="autosave">自动保存</span>
                </div>
                <Editable
                  value={currentScene.synopsis}
                  onSave={(value) => void saveScene(currentScene.id, { synopsis: value })}
                  rows={3}
                  className="editable--body"
                  placeholder="这一场推动什么？观众应该看到什么变化？"
                />
              </div>

              <div className="script-block">
                <div className="script-block__head">
                  <span>剧本正文</span>
                  <span className="script-block__hint">可直接改写，AI 生成镜头时会自动带上</span>
                </div>
                <Editable
                  value={currentScene.scriptText}
                  onSave={(value) => void saveScene(currentScene.id, { scriptText: value })}
                  rows={8}
                  className="editable--screenplay"
                  placeholder="动作、对白、转场提示……"
                  mono
                />
              </div>

              <div className="shot-list-head">
                <div>
                  <h3>分镜</h3>
                  <p>每张卡是一段 3–10 秒的可生成单元。点击进入详细指令。</p>
                </div>
                <div className="shot-list-head__legend">
                  <span><i className="dot dot--green" /> 已有成片</span>
                  <span><i className="dot dot--amber" /> 待审片</span>
                </div>
              </div>

              <div className="shot-rows">
                {currentScene.shots.map((shot, index) => (
                  <ShotRow
                    key={shot.id}
                    shot={shot}
                    index={index}
                    active={currentShot?.id === shot.id}
                    onSelect={() => selectShot(shot.id)}
                    onImage={() => void generateShot(shot.id, "image")}
                    onVideo={() => void generateShot(shot.id, "video")}
                    onContext={() => void openContext(shot.id, shot.videoAssetId ? "video" : "image")}
                  />
                ))}
                {!currentScene.shots.length ? (
                  <EmptyState
                    compact
                    icon={<Film size={20} />}
                    title="这一场还没有分镜"
                    description="重新生成剧本，或从其他场景复制镜头结构。"
                  />
                ) : null}
              </div>
            </>
          )}
        </main>

        <aside className="panel inspector">
          {currentShot ? (
            <ShotInspector
              shot={currentShot}
              onSave={(patch) => void saveShot(currentShot.id, patch)}
              onGenerateImage={() => void generateShot(currentShot.id, "image")}
              onGenerateVideo={() => void generateShot(currentShot.id, "video")}
              onGenerateVoice={() => void generateVoice(currentShot.id)}
              onContext={() => void openContext(currentShot.id, "video")}
              onApprove={() => void setShotStatus(currentShot.id, "approved")}
              onReject={() => void setShotStatus(currentShot.id, "needs_review")}
            />
          ) : (
            <EmptyState
              compact
              icon={<ImageIcon size={20} />}
              title="选择一个镜头"
              description="查看并编辑它的提示词、角色、连续性和生成进度。"
            />
          )}
        </aside>
      </div>
    </div>
  );
}

function ShotRow({
  shot,
  index,
  active,
  onSelect,
  onImage,
  onVideo,
  onContext,
}: {
  shot: Shot;
  index: number;
  active: boolean;
  onSelect: () => void;
  onImage: () => void;
  onVideo: () => void;
  onContext: () => void;
}) {
  const snapshot = useAppStore((s) => s.snapshot);
  const asset = snapshot?.assets.find((a) => a.id === (shot.videoAssetId ?? shot.imageAssetId));
  const characters = snapshot?.characters.filter((c) => shot.characterIds.includes(c.id)) ?? [];
  return (
    <div className={cx("shot-row", active && "is-active")} onClick={onSelect}>
      <span className="shot-row__index">{String(index + 1).padStart(2, "0")}</span>
      <div className="shot-row__thumb">
        {asset ? <img src={assetUrl(asset.thumbPath ?? asset.absPath)} alt={shot.title} /> : <span>NO FRAME</span>}
        {shot.videoAssetId ? <span className="shot-row__play"><Film size={13} /></span> : null}
      </div>
      <div className="shot-row__body">
        <div className="shot-row__title">
          <strong>{shot.title}</strong>
          <StatusPill status={shot.status} />
        </div>
        <p>{shot.description}</p>
        <div className="shot-row__meta">
          <span>{shot.camera || "未设机位"}</span>
          <span>{shot.lens || "35mm"}</span>
          <span>{formatDuration(shot.durationMs)}</span>
          {characters.length ? <span>{characters.map((c) => c.name).join(" / ")}</span> : null}
        </div>
      </div>
      <div className="shot-row__actions" onClick={(event) => event.stopPropagation()}>
        <IconButton label="检查上下文" onClick={onContext}>
          <Sparkles size={15} />
        </IconButton>
        <IconButton label="生成画面" onClick={onImage}>
          <ImageIcon size={15} />
        </IconButton>
        <IconButton label="生成视频" onClick={onVideo}>
          <Film size={15} />
        </IconButton>
      </div>
    </div>
  );
}

function ShotInspector({
  shot,
  onSave,
  onGenerateImage,
  onGenerateVideo,
  onGenerateVoice,
  onContext,
  onApprove,
  onReject,
}: {
  shot: Shot;
  onSave: (patch: UpdateShotInput) => void;
  onGenerateImage: () => void;
  onGenerateVideo: () => void;
  onGenerateVoice: () => void;
  onContext: () => void;
  onApprove: () => void;
  onReject: () => void;
}) {
  const snapshot = useAppStore((s) => s.snapshot);
  const openContext = useAppStore((s) => s.openShotContext);
  const activeJob = snapshot?.jobs.find(
    (job) => job.targetId === shot.id && (job.status === "running" || job.status === "queued"),
  );
  const characters = snapshot?.characters ?? [];
  const asset = snapshot?.assets.find((a) => a.id === (shot.videoAssetId ?? shot.imageAssetId));
  const voiceAsset = snapshot?.assets.find((a) => a.id === shot.voiceAssetId);
  const orderedShots = snapshot?.episodes.flatMap((e) => e.scenes.flatMap((s) => s.shots)) ?? [];
  const currentIndex = orderedShots.findIndex((item) => item.id === shot.id);
  const immediatePrevious = currentIndex > 0 ? orderedShots[currentIndex - 1] : null;
  const previous = orderedShots.find((s) => s.id === shot.continuityFromShotId);

  return (
    <div className="inspector__inner">
      <div className="inspector__head">
        <div>
          <p className="eyebrow">SHOT {String(shot.orderIndex + 1).padStart(2, "0")}</p>
          <h3>{shot.title}</h3>
        </div>
        <StatusPill status={shot.status} />
      </div>

      <div className="inspector-preview">
        {asset ? (
          shot.videoAssetId ? (
            <video src={assetUrl(asset.absPath)} poster={assetUrl(asset.thumbPath)} controls preload="metadata" />
          ) : (
            <img src={assetUrl(asset.absPath)} alt={shot.title} />
          )
        ) : (
          <div className="inspector-preview__empty">
            <ImageIcon size={22} />
            <span>从上下文生成第一版画面</span>
          </div>
        )}
      </div>

      {activeJob ? (
        <div className="inspector-job">
          <div>
            <span className="dot dot--live" />
            <strong>{activeJob.kind === "video" ? "视频生成中" : "画面生成中"}</strong>
            <span>{activeJob.stage}</span>
          </div>
          <div className="inspector-job__bar">
            <i style={{ width: `${activeJob.progress}%` }} />
          </div>
          <small>{activeJob.providerLabel} · {activeJob.model}</small>
        </div>
      ) : null}

      <div className="inspector-actions">
        <Button variant="primary" icon={<Film size={15} />} onClick={onGenerateVideo}>
          生成视频
        </Button>
        <Button variant="secondary" icon={<ImageIcon size={15} />} onClick={onGenerateImage}>
          生成画面
        </Button>
        <Button variant="ghost" icon={<Sparkles size={15} />} onClick={onContext}>
          上下文
        </Button>
      </div>
      <div className="inspector-actions">
        <Button variant="ghost" icon={<Mic size={15} />} onClick={onGenerateVoice}>
          {voiceAsset ? "重新配音" : "生成配音"}
        </Button>
      </div>
      {voiceAsset ? (
        <div className="inspector-section">
          <label className="inspector-label">配音 · 系统语音</label>
          <audio className="audio-bar" controls src={assetUrl(voiceAsset.absPath)} />
          <small className="muted tiny">导出时自动混入具体入点，与字幕/配乐一起输出。</small>
        </div>
      ) : null}
      <div className="inspector-actions inspector-actions--review">
        <Button size="sm" variant="ghost" onClick={onApprove}>
          通过
        </Button>
        <Button size="sm" variant="ghost" onClick={onReject}>
          退回修改
        </Button>
        <Button
          size="sm"
          variant="ghost"
          icon={<ArrowRight size={13} />}
          onClick={() => void openContext(shot.id, "image")}
        >
          画面预览
        </Button>
      </div>

      <div className="inspector-section">
        <label className="inspector-label">镜头描述</label>
        <Editable value={shot.description} onSave={(value) => onSave({ description: value })} rows={4} />
      </div>
      <div className="inspector-section">
        <label className="inspector-label">对白 / 表演提示</label>
        <Editable value={shot.dialogue} onSave={(value) => onSave({ dialogue: value })} rows={3} />
      </div>
      <div className="inspector-section">
        <label className="inspector-label">生成提示词</label>
        <Editable value={shot.prompt} onSave={(value) => onSave({ prompt: value })} rows={5} className="editable--prompt" />
      </div>

      <div className="inspector-grid">
        <label className="mini-field">
          <span>机位</span>
          <Editable value={shot.camera} onSave={(value) => onSave({ camera: value })} />
        </label>
        <label className="mini-field">
          <span>镜头</span>
          <Editable value={shot.lens} onSave={(value) => onSave({ lens: value })} />
        </label>
        <label className="mini-field">
          <span>时长（秒）</span>
          <Editable
            value={String(Math.round(shot.durationMs / 1000))}
            onSave={(value) => onSave({ durationMs: Math.max(1, Number(value)) * 1000 })}
            type="number"
            min={1}
            max={30}
          />
        </label>
        <label className="mini-field">
          <span>Seed</span>
          <Editable
            value={shot.seed ? String(shot.seed) : ""}
            onSave={(value) => onSave({ seed: Number(value) || 0 })}
            type="number"
            placeholder="固定角色抽卡"
          />
        </label>
      </div>

      <div className="inspector-section">
        <label className="inspector-label">出场角色</label>
        <div className="cast-chips">
          {characters.map((character) => {
            const active = shot.characterIds.includes(character.id);
            return (
              <button
                key={character.id}
                className={cx("cast-chip", active && "is-active")}
                onClick={() =>
                  onSave({
                    characterIds: active
                      ? shot.characterIds.filter((id) => id !== character.id)
                      : [...shot.characterIds, character.id],
                  })
                }
              >
                {active ? <Lock size={11} /> : null}
                {character.name}
              </button>
            );
          })}
          {!characters.length ? <span className="muted">先在角色工坊创建角色。</span> : null}
        </div>
      </div>

      <label className="switch-row switch-row--compact">
        <input
          type="checkbox"
          checked={Boolean(shot.continuityFromShotId)}
          disabled={!immediatePrevious && !shot.continuityFromShotId}
          onChange={(event) =>
            onSave(
              event.target.checked
                ? { continuityFromShotId: immediatePrevious?.id ?? "" }
                : { clearContinuity: true },
            )
          }
        />
        <span>
          <strong>尾帧续接</strong>
          <small>
            {shot.continuityFromShotId
              ? `从上一镜头${previous ? `「${previous.title}」` : ""}提取尾帧作为首帧`
              : immediatePrevious
                ? `开启后从「${immediatePrevious.title}」提取尾帧`
                : "这是第一个镜头，没有可续接的前序画面"}
          </small>
        </span>
      </label>

      <div className="inspector-foot">
        <span>版本 v{shot.version || 1}</span>
        <span>{shot.providerId || "尚未生成"}</span>
      </div>
    </div>
  );
}
