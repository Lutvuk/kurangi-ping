import type { InputHTMLAttributes } from "react";

export type InputProps = InputHTMLAttributes<HTMLInputElement>;

export function Input({ className, ...props }: InputProps) {
  const composedClassName = ["kp-input", "kp-primitive-focus", className]
    .filter(Boolean)
    .join(" ");

  return <input className={composedClassName} {...props} />;
}
