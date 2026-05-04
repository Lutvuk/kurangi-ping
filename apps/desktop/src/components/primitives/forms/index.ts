export { Input } from "./Input";
export type { InputProps } from "./Input";
export { Select } from "./Select";
export type { SelectOption, SelectProps } from "./Select";

export const formPrimitiveNames = ["Input", "Select"] as const;

export type FormPrimitiveName = (typeof formPrimitiveNames)[number];
