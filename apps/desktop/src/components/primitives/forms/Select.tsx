import type { SelectHTMLAttributes } from "react";

export type SelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
};

export type SelectProps = Omit<SelectHTMLAttributes<HTMLSelectElement>, "size"> & {
  options: SelectOption[];
};

export function Select({ className, options, ...props }: SelectProps) {
  const composedClassName = ["kp-select", "kp-primitive-focus", className]
    .filter(Boolean)
    .join(" ");

  return (
    <select className={composedClassName} tabIndex={props.tabIndex ?? 0} {...props}>
      {options.map((option) => (
        <option key={option.value} value={option.value} disabled={option.disabled}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
