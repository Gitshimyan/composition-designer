import type { PropertyContract } from "../types";

/** 把规范化配置值转换成表单草稿字符串。 */
export function toDraftValue(value: unknown, type: string): string {
  if (value === undefined || value === null) return "";
  if (type === "boolean") return value ? "true" : "false";
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

/** 把表单草稿解析成 `ConfigValue` 支持的 JSON 值；不重复 Core 的契约校验。 */
export function parseDraft(
  draft: string,
  property: PropertyContract,
): { value: unknown } | { error: string } {
  if (property.type === "boolean") return { value: draft === "true" };
  if (property.type === "integer") {
    if (!/^-?\d+$/.test(draft.trim())) return { error: "Expected an integer" };
    const value = Number.parseInt(draft, 10);
    if (!Number.isSafeInteger(value)) return { error: "Integer is out of range" };
    return { value };
  }
  if (property.type === "float") {
    if (draft.trim() === "") return { error: "Expected a number" };
    const value = Number.parseFloat(draft);
    if (Number.isNaN(value) || !Number.isFinite(value)) {
      return { error: "Expected a finite number" };
    }
    return { value };
  }
  if (property.type === "array") {
    try {
      const value = JSON.parse(draft);
      if (!Array.isArray(value)) return { error: "Expected an array" };
      return { value };
    } catch {
      return { error: "Expected valid JSON array" };
    }
  }
  if (property.type === "enum" && !property.values.includes(draft)) {
    return { error: "Choose one of the declared values" };
  }
  return { value: draft };
}

/**
 * 模块切换时判断现有配置值能否原样保留。只比较属性类型的表层形状，
 * 具体 bounds/default/契约一致性仍由 Core 校验。
 */
export function isConfigValueCompatible(
  value: unknown,
  property: PropertyContract,
): boolean {
  if (value === undefined) return false;
  switch (property.type) {
    case "string":
      return typeof value === "string";
    case "integer":
      return typeof value === "number" && Number.isInteger(value);
    case "float":
      return typeof value === "number" && Number.isFinite(value);
    case "boolean":
      return typeof value === "boolean";
    case "enum":
      return typeof value === "string" && property.values.includes(value);
    case "array":
      return Array.isArray(value);
    default:
      return false;
  }
}

/** 必填属性的初始草稿：优先保留兼容的当前值，否则使用属性类型的空值。 */
export function initialDraftFor(
  property: PropertyContract,
  current: unknown,
): string {
  if (isConfigValueCompatible(current, property)) {
    return toDraftValue(current, property.type);
  }
  return defaultDraftFor(property);
}

export function defaultDraftFor(property: PropertyContract): string {
  if (property.type === "boolean") return "true";
  if (property.type === "enum") return property.values[0] ?? "";
  if (property.type === "array") return "[]";
  return "";
}
