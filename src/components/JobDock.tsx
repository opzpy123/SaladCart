import { useState } from "react";
import {
  ChevronDown,
  ChevronUp,
  CircleStop,
  Film,
  ListChecks,
  Loader2,
  RefreshCw,
  X,
} from "lucide-react";
import { formatError, invoke } from "@/lib/bridge";
import type { GenerationJob } from "@/lib/types";
import { useAppStore } from "@/store/useAppStore";
import { Button, IconButton, Progress, StatusPill, cx } from "./ui";

const KIND_LABEL: Record<string, string> = {
  script: "剧本",
  image: "画面",
  video: "视频",
  export: "导出",
  music: "配乐",
};

export function JobDock() {
  const snapshot = useAppStore((s) => s.snapshot);
  const open = useAppStore((s) => s.jobDockOpen);
  const setOpen = useAppStore((s) => s.setJobDockOpen);
  const handleJob = useAppStore((s) => s.handleJob);
  const pushToast = useAppStore((s) => s.pushToast);
  const [actionId, setActionId] = useState<string | null>(null);

  if (!snapshot) return null;
  const jobs = snapshot.jobs;
  const active = jobs.filter((job) => job.status === "running" || job.status === "queued");
  const recent = jobs.slice(0, 8);
  if (!active.length && !open) return null;

  const cancel = async (job: GenerationJob) => {
    setActionId(job.id);
    try {
      const next = await invoke<GenerationJob>("cancel_job", {
        projectId: job.projectId,
        jobId: job.id,
      });
      handleJob(next);
    } catch (error) {
      pushToast("error", "取消失败", formatError(error));
    } finally {
      setActionId(null);
    }
  };

  const retry = async (job: GenerationJob) => {
    setActionId(job.id);
    try {
      const next = await invoke<GenerationJob>("retry_job", {
        projectId: job.projectId,
        jobId: job.id,
      });
      handleJob(next);
      pushToast("info", "已重新入队", next.requestSummary);
    } catch (error) {
      pushToast("error", "重试失败", formatError(error));
    } finally {
      setActionId(null);
    }
  };

  if (!open) {
    const primary = active[0];
    return (
      <button className="job-pill" onClick={() => setOpen(true)}>
        <span className="job-pill__icon">
          {active.length ? <Loader2 size={16} className="spin" /> : <ListChecks size={16} />}
        </span>
        <span>
          <strong>{active.length ? `${active.length} 个任务在运行` : "任务记录"}</strong>
          <small>{primary ? primary.stage : "查看最近的生成"}</small>
        </span>
        <span className="job-pill__progress">
          {primary ? `${primary.progress}%` : <ChevronUp size={15} />}
        </span>
      </button>
    );
  }

  return (
    <aside className="job-dock">
      <header className="job-dock__head">
        <div>
          <p className="eyebrow">GENERATION QUEUE</p>
          <h3>生成任务</h3>
        </div>
        <div className="job-dock__head-actions">
          <span className="job-dock__count">{active.length} 运行中</span>
          <IconButton label="收起" onClick={() => setOpen(false)}>
            <ChevronDown size={17} />
          </IconButton>
        </div>
      </header>
      <div className="job-list">
        {recent.length ? (
          recent.map((job) => (
            <div key={job.id} className={cx("job-item", `job-item--${job.status}`)}>
              <div className="job-item__main">
                <div className="job-item__title">
                  <span className={cx("job-kind", `job-kind--${job.kind}`)}>
                    {job.kind === "video" ? <Film size={13} /> : job.kind === "music" ? <span>♪</span> : null}
                    {KIND_LABEL[job.kind] ?? job.kind}
                  </span>
                  <strong>{job.requestSummary}</strong>
                </div>
                <div className="job-item__meta">
                  <span>{job.providerLabel}</span>
                  <span>{job.model}</span>
                  <span>{job.costEstimate ? `≈ ¥${job.costEstimate.toFixed(2)}` : "零成本"}</span>
                </div>
                {job.status === "running" || job.status === "queued" ? (
                  <Progress value={job.progress} label={`${job.progress}% · ${job.stage}`} />
                ) : null}
                {job.error ? <p className="job-item__error">{job.error}</p> : null}
              </div>
              <div className="job-item__side">
                <StatusPill status={job.status} />
                {job.status === "running" || job.status === "queued" ? (
                  <IconButton label="取消任务" onClick={() => cancel(job)} disabled={actionId === job.id}>
                    <CircleStop size={16} />
                  </IconButton>
                ) : job.status === "failed" || job.status === "canceled" ? (
                  <IconButton label="重试" onClick={() => retry(job)} disabled={actionId === job.id}>
                    <RefreshCw size={15} />
                  </IconButton>
                ) : null}
              </div>
            </div>
          ))
        ) : (
          <div className="job-list__empty">还没有任务记录。</div>
        )}
      </div>
      <footer className="job-dock__foot">
        <span>并发 {snapshot.jobs.filter((j) => j.status === "running").length} 个任务</span>
        <Button size="sm" variant="ghost" icon={<X size={13} />} onClick={() => setOpen(false)}>
          收起
        </Button>
      </footer>
    </aside>
  );
}
