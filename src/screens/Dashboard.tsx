import {
  ArrowUpRight,
  CheckCircle2,
  Clapperboard,
  Clock3,
  Film,
  FolderKanban,
  Image as ImageIcon,
  Layers,
  Lock,
  Music,
  Plus,
  Scissors,
  Sparkles,
  Users,
  Wand2,
} from "lucide-react";
import { assetUrl, formatCurrency, formatDuration, relativeTime } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { ProjectSummary } from "@/lib/types";
import {
  Button,
  EmptyState,
  IconButton,
  Progress,
  SectionHeader,
  StatusPill,
  cx,
} from "@/components/ui";

export function ProjectDashboard() {
  const projects = useAppStore((s) => s.projects);
  const snapshot = useAppStore((s) => s.snapshot);
  const system = useAppStore((s) => s.system);
  const settings = useAppStore((s) => s.settings);
  const openProject = useAppStore((s) => s.openProject);
  const setNewProject = useAppStore((s) => s.setShowNewProject);
  const setRoute = useAppStore((s) => s.setRoute);
  const archive = useAppStore((s) => s.archiveProject);

  const latest = projects[0];
  const activeSummary = snapshot
    ? projects.find((p) => p.id === snapshot.project.id) ?? latest
    : latest;
  const counts = activeSummary?.counts;
  const cover = activeSummary?.coverPath;

  return (
    <div className="dashboard">
      <section className="dashboard-hero">
        <div className="dashboard-hero__copy">
          <p className="eyebrow">SALADCART / AI COMIC DRAMA WORKSTATION</p>
          <h1>
            把故事，
            <br />
            送上轨道。
          </h1>
          <p className="dashboard-hero__lede">
            剧本、角色、分镜、视频、配乐与剪辑共用一个项目图。每个镜头生成前，自动编译角色锁定、场景和连续性——
            先看清要花什么，再按下去。
          </p>
          <div className="dashboard-hero__actions">
            <Button variant="primary" size="lg" icon={<Plus size={17} />} onClick={() => setNewProject(true)}>
              开一部新漫剧
            </Button>
            {latest ? (
              <Button
                variant="secondary"
                size="lg"
                icon={<Clapperboard size={17} />}
                onClick={() => void openProject(latest.id)}
              >
                继续「{latest.name}」
              </Button>
            ) : null}
          </div>
          <div className="hero-metrics">
            <div>
              <strong>{projects.length}</strong>
              <span>本机项目</span>
            </div>
            <div>
              <strong>{activeSummary?.counts.shots ?? 0}</strong>
              <span>项目镜头</span>
            </div>
            <div>
              <strong>{activeSummary?.counts.videos ?? 0}</strong>
              <span>已生成成片</span>
            </div>
            <div>
              <strong>{system?.ffmpegAvailable ? "ON" : "OFF"}</strong>
              <span>FFmpeg 引擎</span>
            </div>
          </div>
        </div>
        <div className="dashboard-hero__visual">
          <div className="hero-poster hero-poster--back" />
          <div className="hero-poster hero-poster--main">
            {cover ? <img src={assetUrl(cover)} alt="项目封面" /> : <div className="hero-poster__fallback"><img src="/brand/icon.png" alt="" /></div>}
            <div className="hero-poster__scrim" />
            <div className="hero-poster__caption">
              <p>{activeSummary?.genre || "AI COMIC DRAMA"}</p>
              <strong>{activeSummary?.name || "SaladCart Studio"}</strong>
              <span>{activeSummary?.logline || "剧本 → 角色 → 分镜 → 视频 → 配乐 → 成片"}</span>
            </div>
            <div className="hero-poster__tag">
              <span className="dot dot--live" /> CONTEXT READY
            </div>
          </div>
          <div className="hero-context-card">
            <div className="hero-context-card__head">
              <Sparkles size={14} />
              <span>下一次生成</span>
              <em>已锁定</em>
            </div>
            <p>沈知遥 · 黑色及肩短发 / 银灰挑染 / 深灰风衣</p>
            <div className="hero-context-card__row">
              <span>角色参考 × 2</span>
              <span>尾帧续接</span>
              <span>9:16 · 5s</span>
            </div>
            <div className="hero-context-card__cost">
              <span>预估</span>
              <strong>¥0.00</strong>
              <em>MOCK</em>
            </div>
          </div>
        </div>
      </section>

      <section className="dashboard-grid">
        <div className="panel dashboard-projects">
          <SectionHeader
            title="继续创作"
            subtitle="每个项目独立存储，可以整个文件夹迁移或备份。"
            action={
              <Button size="sm" variant="ghost" icon={<Plus size={14} />} onClick={() => setNewProject(true)}>
                新项目
              </Button>
            }
          />
          {projects.length ? (
            <div className="project-list">
              {projects.map((project) => (
                <ProjectRow
                  key={project.id}
                  project={project}
                  active={snapshot?.project.id === project.id}
                  onOpen={() => void openProject(project.id)}
                  onArchive={() => {
                    if (window.confirm(`把「${project.name}」移入回收区？项目文件不会被永久删除。`)) {
                      void archive(project.id);
                    }
                  }}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              icon={<FolderKanban size={26} />}
              title="还没有项目"
              description="建立第一个漫剧项目，或者从一句话立意开始生成完整分镜。"
              action={
                <Button variant="primary" icon={<Plus size={15} />} onClick={() => setNewProject(true)}>
                  新建项目
                </Button>
              }
            />
          )}
        </div>

        <div className="panel dashboard-pipeline">
          <SectionHeader title="生产流水线" subtitle={activeSummary ? `当前：${activeSummary.name}` : "创建项目后自动启用"} />
          <div className="pipeline-list">
            <PipelineStep icon={<Sparkles size={16} />} label="剧本与分镜" value={counts ? `${counts.episodes} 集 · ${counts.scenes} 场` : "等待开始"} hint={counts?.shots ? `${counts.shots} 个镜头` : "从一句立意生成"} done={Boolean(counts?.shots)} onClick={() => snapshot && setRoute("script")} />
            <PipelineStep icon={<Users size={16} />} label="角色锁定" value={counts ? `${counts.characters} 个角色` : "未建角色"} hint="定妆照 + 特征锁" done={Boolean(counts?.characters)} onClick={() => snapshot && setRoute("characters")} />
            <PipelineStep icon={<ImageIcon size={16} />} label="画面与抽卡" value={counts ? `${counts.images} 张素材` : "未生成"} hint="上下文预览 · 成本估算" done={Boolean(counts?.images)} onClick={() => snapshot && setRoute("shots")} />
            <PipelineStep icon={<Film size={16} />} label="视频镜头" value={counts ? `${counts.videos} 条成片` : "未生成"} hint="批量队列 · 尾帧续接" done={Boolean(counts?.videos)} onClick={() => snapshot && setRoute("shots")} />
            <PipelineStep icon={<Music size={16} />} label="配乐与卡点" value={snapshot?.timeline.bpm ? `${snapshot.timeline.bpm} BPM` : "可分析"} hint="AI 配乐 · 自动切点" done={Boolean(snapshot?.timeline.musicAssetId)} onClick={() => snapshot && setRoute("editor")} />
            <PipelineStep icon={<Scissors size={16} />} label="粗剪与交付" value={snapshot?.timeline.clips.length ? `${snapshot.timeline.clips.length} 个片段` : "待装配"} hint="字幕 · 竖屏导出" done={Boolean(snapshot?.timeline.clips.length)} onClick={() => snapshot && setRoute("editor")} />
          </div>
        </div>

        <div className="panel dashboard-jobs">
          <SectionHeader title="最近任务" subtitle={snapshot ? "来自当前打开的项目" : "打开项目查看任务"} />
          {snapshot?.jobs.length ? (
            <div className="dashboard-job-list">
              {snapshot.jobs.slice(0, 5).map((job) => (
                <div key={job.id} className="dashboard-job">
                  <span className={cx("job-dot", job.status)} />
                  <div>
                    <strong>{job.requestSummary}</strong>
                    <small>{job.providerLabel} · {relativeTime(job.createdAt)}</small>
                  </div>
                  <div className="dashboard-job__right">
                    {job.status === "running" || job.status === "queued" ? (
                      <Progress value={job.progress} label={`${job.progress}%`} />
                    ) : (
                      <StatusPill status={job.status} />
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="dashboard-jobs__empty">
              <Clock3 size={18} />
              <p>任务历史会在这里出现。生成过程可随时切页，完成后自动写回项目。</p>
            </div>
          )}
        </div>

        <div className="panel dashboard-system">
          <SectionHeader
            title="本机工作站"
            action={
              <IconButton label="打开设置" onClick={() => setRoute("settings")}>
                <ArrowUpRight size={16} />
              </IconButton>
            }
          />
          <div className="system-rows">
            <div>
              <span className={cx("status-dot", system?.ffmpegAvailable ? "is-good" : "is-bad")} />
              <div>
                <strong>FFmpeg</strong>
                <small>{system?.ffmpegAvailable ? system.ffmpegVersion ?? "已安装" : "未检测到，导出不可用"}</small>
              </div>
            </div>
            <div>
              <span className={cx("status-dot", settings?.providers.some((p) => p.apiKeySet) ? "is-good" : "is-warm")} />
              <div>
                <strong>供应商</strong>
                <small>
                  {settings?.providers.filter((p) => p.apiKeySet).length ?? 0} 个已配置 Key · 默认 {settings?.providers.find((p) => p.id === settings.defaults.video)?.label ?? "Mock"}
                </small>
              </div>
            </div>
            <div>
              <span className="status-dot is-good" />
              <div>
                <strong>项目存储</strong>
                <small>{system?.dataDir ?? "本地应用数据目录"}</small>
              </div>
            </div>
          </div>
          <Button size="sm" variant="secondary" onClick={() => setRoute("settings")}>
            管理供应商与引擎
          </Button>
        </div>
      </section>

      <section className="dashboard-features">
        <Feature icon={<Lock size={18} />} title="角色不漂移" text="特征锁 + 定妆照 + 三视图自动注入每个镜头。" />
        <Feature icon={<Layers size={18} />} title="上下文可见" text="生成前看到完整提示词、参考图、连续性与成本。" />
        <Feature icon={<Wand2 size={18} />} title="批量抽卡" text="任务队列、并发、重试与版本保留，不再手工等。" />
        <Feature icon={<Music size={18} />} title="音乐自动卡点" text="搜索/生成配乐，分析 BPM，让切点落在节拍上。" />
      </section>
    </div>
  );
}

function ProjectRow({
  project,
  active,
  onOpen,
  onArchive,
}: {
  project: ProjectSummary;
  active: boolean;
  onOpen: () => void;
  onArchive: () => void;
}) {
  return (
    <div className={cx("project-row", active && "is-active")} onClick={onOpen}>
      <div className="project-row__cover">
        {project.coverPath ? <img src={assetUrl(project.coverPath)} alt={project.name} /> : <span>{project.name.slice(0, 1)}</span>}
      </div>
      <div className="project-row__body">
        <div className="project-row__title">
          <strong>{project.name}</strong>
          <span className="project-row__genre">{project.genre || "漫剧"}</span>
          {active ? <span className="project-row__active">已打开</span> : null}
        </div>
        <p>{project.logline || project.premise || "尚未生成故事梗概"}</p>
        <div className="project-row__meta">
          <span><Users size={12} /> {project.counts.characters}</span>
          <span><Film size={12} /> {project.counts.shots}</span>
          <span><ImageIcon size={12} /> {project.counts.videos} 成片</span>
          <span>{relativeTime(project.updatedAt)}</span>
        </div>
      </div>
      <div className="project-row__side">
        <span className="project-row__aspect">{project.aspectRatio}</span>
        <Button size="sm" variant="ghost" onClick={(event) => { event.stopPropagation(); onOpen(); }}>
          打开
        </Button>
        <IconButton label="归档" onClick={(event) => { event.stopPropagation(); onArchive(); }}>
          <FolderKanban size={14} />
        </IconButton>
      </div>
    </div>
  );
}

function PipelineStep({
  icon,
  label,
  value,
  hint,
  done,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  hint: string;
  done: boolean;
  onClick: () => void;
}) {
  return (
    <button className={cx("pipeline-step", done && "is-done")} onClick={onClick}>
      <span className="pipeline-step__icon">{done ? <CheckCircle2 size={16} /> : icon}</span>
      <span className="pipeline-step__body">
        <strong>{label}</strong>
        <small>{hint}</small>
      </span>
      <span className="pipeline-step__value">{value}</span>
    </button>
  );
}

function Feature({ icon, title, text }: { icon: React.ReactNode; title: string; text: string }) {
  return (
    <article className="feature-card">
      <span>{icon}</span>
      <div>
        <strong>{title}</strong>
        <p>{text}</p>
      </div>
    </article>
  );
}
