import type { HTMLAttributes } from "react";

export type CardProps = HTMLAttributes<HTMLElement> & {
  as?: "section" | "article" | "div";
};

export function Card({ as = "section", className, ...props }: CardProps) {
  const Component = as;
  const composedClassName = ["kp-card", className].filter(Boolean).join(" ");
  return <Component className={composedClassName} {...props} />;
}
