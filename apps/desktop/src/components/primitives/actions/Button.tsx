import type { ButtonHTMLAttributes } from "react";
import type { PrimitiveIntent } from "../shared/types";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  intent?: PrimitiveIntent;
};

const classByIntent: Record<PrimitiveIntent, string> = {
  primary: "kp-button--primary",
  secondary: "kp-button--secondary",
  destructive: "kp-button--destructive"
};

export function Button({
  intent = "secondary",
  className,
  type = "button",
  ...props
}: ButtonProps) {
  const intentClass = classByIntent[intent];
  const composedClassName = ["kp-button", "kp-primitive-focus", intentClass, className]
    .filter(Boolean)
    .join(" ");

  return <button type={type} className={composedClassName} {...props} />;
}
