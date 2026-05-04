import type { ButtonHTMLAttributes } from "react";
import type { PrimitiveIntent } from "../shared/types";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  intent?: PrimitiveIntent;
  ariaLabel?: string;
};

const classByIntent: Record<PrimitiveIntent, string> = {
  primary: "kp-button--primary",
  secondary: "kp-button--secondary",
  destructive: "kp-button--destructive"
};

export function Button({
  intent = "secondary",
  ariaLabel,
  className,
  type = "button",
  ...props
}: ButtonProps) {
  const intentClass = classByIntent[intent];
  const composedClassName = ["kp-button", "kp-primitive-focus", intentClass, className]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      type={type}
      aria-label={ariaLabel}
      className={composedClassName}
      tabIndex={props.tabIndex ?? 0}
      {...props}
    />
  );
}
