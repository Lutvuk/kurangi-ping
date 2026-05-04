import type { HTMLAttributes, ReactNode } from "react";

export type PanelProps = HTMLAttributes<HTMLElement> & {
  eyebrow?: string;
  title?: string;
  actions?: ReactNode;
  children: ReactNode;
};

export function Panel({ eyebrow, title, actions, children, className, ...props }: PanelProps) {
  const composedClassName = ["kp-panel", className].filter(Boolean).join(" ");

  return (
    <section className={composedClassName} {...props}>
      {eyebrow || title || actions ? (
        <header className="kp-panel-header">
          <div className="kp-panel-heading">
            {eyebrow ? <p className="kp-label">{eyebrow}</p> : null}
            {title ? <h1 className="kp-title">{title}</h1> : null}
          </div>
          {actions ? <div className="kp-panel-actions">{actions}</div> : null}
        </header>
      ) : null}
      <div className="kp-panel-body">{children}</div>
    </section>
  );
}
