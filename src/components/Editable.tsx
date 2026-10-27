import { useEffect, useRef, useState } from "react";

interface EditableProps {
  value: string;
  onSave: (value: string) => void;
  className?: string;
  placeholder?: string;
  rows?: number;
  type?: "text" | "number";
  min?: number;
  max?: number;
  mono?: boolean;
}

export function Editable({
  value,
  onSave,
  className,
  placeholder,
  rows = 1,
  type = "text",
  min,
  max,
  mono,
}: EditableProps) {
  const [draft, setDraft] = useState(value);
  const focused = useRef(false);
  useEffect(() => {
    if (!focused.current) setDraft(value);
  }, [value]);

  const commit = () => {
    focused.current = false;
    if (draft !== value) {
      const next = type === "number" ? String(Number(draft) || 0) : draft;
      if (next !== value) onSave(next);
    }
  };

  if (rows > 1) {
    return (
      <textarea
        className={`editable ${mono ? "editable--mono" : ""} ${className ?? ""}`}
        value={draft}
        rows={rows}
        placeholder={placeholder}
        onFocus={() => (focused.current = true)}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
      />
    );
  }

  return (
    <input
      className={`editable ${mono ? "editable--mono" : ""} ${className ?? ""}`}
      value={draft}
      type={type}
      min={min}
      max={max}
      placeholder={placeholder}
      onFocus={() => (focused.current = true)}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={commit}
      onKeyDown={(event) => {
        if (event.key === "Enter") event.currentTarget.blur();
        if (event.key === "Escape") {
          setDraft(value);
          focused.current = false;
          event.currentTarget.blur();
        }
      }}
    />
  );
}
