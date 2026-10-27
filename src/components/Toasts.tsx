import { AlertTriangle, CheckCircle2, Info, X } from "lucide-react";
import { useAppStore } from "@/store/useAppStore";
import { IconButton, cx } from "./ui";

export function Toasts() {
  const toasts = useAppStore((s) => s.toasts);
  const remove = useAppStore((s) => s.removeToast);
  if (!toasts.length) return null;
  return (
    <div className="toast-stack">
      {toasts.map((toast) => (
        <div key={toast.id} className={cx("toast", `toast--${toast.kind}`)}>
          <span className="toast__icon">
            {toast.kind === "success" ? (
              <CheckCircle2 size={17} />
            ) : toast.kind === "error" ? (
              <AlertTriangle size={17} />
            ) : (
              <Info size={17} />
            )}
          </span>
          <div className="toast__body">
            <strong>{toast.title}</strong>
            {toast.message ? <p>{toast.message}</p> : null}
          </div>
          <IconButton label="关闭" onClick={() => remove(toast.id)}>
            <X size={14} />
          </IconButton>
        </div>
      ))}
    </div>
  );
}
