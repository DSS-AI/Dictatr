import { useEffect, useRef, useState } from "react";

interface Props { text: string; enabled: boolean; }

export default function InfoTip({ text, enabled }: Props) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const ref = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!open || !ref.current) return;
    const r = ref.current.getBoundingClientRect();
    setPos({ left: r.left + r.width / 2, top: r.bottom + 6 });
  }, [open]);

  if (!enabled) return null;

  return (
    <span
      ref={ref}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        width: 13, height: 13,
        marginLeft: 5,
        borderRadius: "50%",
        background: "var(--primary-dim)",
        color: "var(--text)",
        fontSize: 8,
        fontWeight: 700,
        cursor: "help",
        userSelect: "none",
        verticalAlign: "middle",
      }}
    >
      ?
      {open && pos && (
        <span
          style={{
            position: "fixed",
            left: pos.left,
            top: pos.top,
            transform: "translateX(-50%)",
            background: "#1a1a28",
            color: "#e8e8f0",
            border: "1px solid var(--field-border)",
            borderRadius: 6,
            padding: "8px 10px",
            maxWidth: 360,
            fontSize: 12,
            fontWeight: 400,
            lineHeight: 1.4,
            boxShadow: "0 4px 16px rgba(0,0,0,0.5)",
            zIndex: 9999,
            pointerEvents: "none",
            whiteSpace: "normal",
          }}
        >
          {text}
        </span>
      )}
    </span>
  );
}
