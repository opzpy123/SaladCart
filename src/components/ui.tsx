import type { ButtonHTMLAttributes, ReactNode } from "react";
import { useEffect } from "react";
import { createPortal } from "react-dom";
import {
  AlertTriangle,
  CheckCircle2,
  ImageOff,
  Loader2,
  X,
} from "lucide-react";

export function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "outline" | "soft";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  icon?: ReactNode;
}

export function Button({
  variant = "secondary",
  size = "md",
  loading,
  icon,
  className,
  children,
  disabled,
  ...rest
}: ButtonProps) {
  return (
    <button
      className={cx("btn", `btn--${variant}`, `btn--${size}`, className)}
      disabled={disabled || loading}
      {...rest}
    >
      {loading ? <Loader2 className="spin" size={15} /> : icon}
      {children ? <span>{children}</span> : null}
    </button>
  );
}

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  label: string;
  children: ReactNode;
  active?: boolean;
}

export function IconButton({ label, children, className, active, ...rest }: IconButtonProps) {
  return (
    <button
      className={cx("icon-btn", active && "is-active", className)}
      aria-label={label}
      title={label}
      {...rest}
    >
      {children}
    </button>
  );
}

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  children: ReactNode;
  footer?: ReactNode;
  wide?: boolean;
}

export function Modal({ open, onClose, title, subtitle, children, footer, wide }: ModalProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;
  return createPortal(
    <div className="modal-backdrop" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <div className={cx("modal", wide && "modal--wide")} role="dialog" aria-modal="true" aria-label={title}>
        <header className="modal__head">
          <div>
            <p className="eyebrow">SALADCART STUDIO</p>
            <h2>{title}</h2>
            {subtitle ? <p className="modal__subtitle">{subtitle}</p> : null}
          </div>
          <IconButton label="关闭" onClick={onClose}>
            <X size={18} />
          </IconButton>
        </header>
        <div className="modal__body">{children}</div>
        {footer ? <footer className="modal__foot">{footer}</footer> : null}
      </div>
    </div>,
    document.body,
  );
}

export function Field({
  label,
  hint,
  children,
  className,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <label className={cx("field", className)}>
      <span className="field__label">{label}</span>
      {children}
      {hint ? <span className="field__hint">{hint}</span> : null}
    </label>
  );
}

export function SectionHeader({
  title,
  subtitle,
  action,
  className,
}: {
  title: string;
  subtitle?: string;
  action?: ReactNode;
  className?: string;
}) {
  return (
    <div className={cx("section-head", className)}>
      <div>
        <h3>{title}</h3>
        {subtitle ? <p>{subtitle}</p> : null}
      </div>
      {action}
    </div>
  );
}

const STATUS_MAP: Record<string, { label: string; tone: string }> = {
  draft: { label: "草稿", tone: "neutral" },
  queued: { label: "排队中", tone: "neutral" },
  running: { label: "生成中", tone: "active" },
  succeeded: { label: "已完成", tone: "good" },
  ready: { label: "可查看", tone: "good" },
  needs_review: { label: "待审片", tone: "warn" },
  approved: { label: "已通过", tone: "good" },
  failed: { label: "失败", tone: "bad" },
  canceled: { label: "已取消", tone: "neutral" },
  generating: { label: "生成中", tone: "active" },
};

export function StatusPill({ status, label }: { status: string; label?: string }) {
  const item = STATUS_MAP[status] ?? { label: status || "未知", tone: "neutral" };
  return (
    <span className={cx("status-pill", `status-pill--${item.tone}`)}>
      <span className="status-pill__dot" />
      {label ?? item.label}
    </span>
  );
}

export function EmptyState({
  icon,
  title,
  description,
  action,
  compact,
}: {
  icon?: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
  compact?: boolean;
}) {
  return (
    <div className={cx("empty", compact && "empty--compact")}>
      <div className="empty__icon">{icon ?? <AlertTriangle size={22} />}</div>
      <h3>{title}</h3>
      <p>{description}</p>
      {action ? <div className="empty__action">{action}</div> : null}
    </div>
  );
}

export function Progress({ value, label, tone }: { value: number; label?: string; tone?: "lime" | "red" }) {
  return (
    <div className={cx("progress", tone && `progress--${tone}`)}>
      <div className="progress__track">
        <div className="progress__bar" style={{ width: `${Math.max(0, Math.min(100, value))}%` }} />
      </div>
      {label ? <span className="progress__label">{label}</span> : null}
    </div>
  );
}

export function Thumb({
  src,
  alt,
  className,
  aspect,
  icon,
  onClick,
}: {
  src?: string | null;
  alt: string;
  className?: string;
  aspect?: string;
  icon?: ReactNode;
  onClick?: () => void;
}) {
  return (
    <div
      className={cx("thumb", className)}
      style={aspect ? { aspectRatio: aspect } : undefined}
      onClick={onClick}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      {src ? (
        <img src={src} alt={alt} loading="lazy" onError={(event) => (event.currentTarget.style.display = "none")} />
      ) : (
        <div className="thumb__fallback">
          <ImageOff size={22} />
          <span>{alt}</span>
        </div>
      )}
      {icon ? <div className="thumb__icon">{icon}</div> : null}
    </div>
  );
}

export function Segmented<T extends string>({
  options,
  value,
  onChange,
  size = "md",
}: {
  options: Array<{ value: T; label: string; icon?: ReactNode }>;
  value: T;
  onChange: (value: T) => void;
  size?: "sm" | "md";
}) {
  return (
    <div className={cx("segmented", size === "sm" && "segmented--sm")}>
      {options.map((option) => (
        <button
          key={option.value}
          className={cx("segmented__item", option.value === value && "is-active")}
          onClick={() => onChange(option.value)}
          type="button"
        >
          {option.icon}
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function InlineNotice({
  tone = "info",
  children,
}: {
  tone?: "info" | "warn" | "error" | "good";
  children: ReactNode;
}) {
  return (
    <div className={cx("notice", `notice--${tone}`)}>
      {tone === "good" ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />}
      <div>{children}</div>
    </div>
  );
}

export function Divider({ label }: { label?: string }) {
  return <div className="divider">{label ? <span>{label}</span> : null}</div>;
}

export function Skeleton({ lines = 3 }: { lines?: number }) {
  return (
    <div className="skeleton">
      {Array.from({ length: lines }).map((_, index) => (
        <div key={index} className="skeleton__line" style={{ width: `${92 - index * 11}%` }} />
      ))}
    </div>
  );
}
