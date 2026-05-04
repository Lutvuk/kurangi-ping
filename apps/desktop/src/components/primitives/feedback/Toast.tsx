import type { HTMLAttributes, ReactNode } from "react";

export type ToastTone = "info" | "warning" | "error" | "success";

export type ToastProps = HTMLAttributes<HTMLDivElement> & {
  tone?: ToastTone;
  title: string;
  description?: ReactNode;
};

const toneClassByValue: Record<ToastTone, string> = {
  info: "kp-toast--info",
  warning: "kp-toast--warning",
  error: "kp-toast--error",
  success: "kp-toast--success"
};

export function Toast({
  tone = "info",
  title,
  description,
  className,
  ...props
}: ToastProps) {
  const composedClassName = ["kp-toast", toneClassByValue[tone], className]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={composedClassName} role="status" aria-live="polite" {...props}>
      <strong className="kp-toast-title">{title}</strong>
      {description ? <p className="kp-toast-description">{description}</p> : null}
    </div>
  );
}
