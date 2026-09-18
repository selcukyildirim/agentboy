import { describe, expect, it } from "vitest";
import {
  defaultInputFromSchema,
  groupByDepartment,
  missingRequiredFields,
} from "./agentInput";

const FIELDS = [
  { key: "bank_statement", label: "Bank", kind: "file", required: true, example: "a,b\n1,2" },
  { key: "tolerance", label: "Tolerance", kind: "number", required: false, example: "0.01" },
  { key: "options", label: "Options", kind: "json", required: true, example: '[{"x":1}]' },
];

describe("agentInput helpers", () => {
  it("builds default input from examples by kind", () => {
    const input = defaultInputFromSchema(FIELDS);
    expect(input.bank_statement).toBe("a,b\n1,2");
    expect(input.tolerance).toBe(0.01);
    expect(Array.isArray(input.options)).toBe(true);
    expect(input.options[0].x).toBe(1);
  });

  it("skips fields without example", () => {
    const input = defaultInputFromSchema([{ key: "x", kind: "text", required: true }]);
    expect("x" in input).toBe(false);
  });

  it("detects missing required fields", () => {
    const missing = missingRequiredFields(FIELDS, { bank_statement: "" });
    expect(missing).toContain("bank_statement");
    expect(missing).toContain("options");
    expect(missing).not.toContain("tolerance");
  });

  it("groups agents by department", () => {
    const groups = groupByDepartment([
      { department: "Finance", name: "A" },
      { department: "Finance", name: "B" },
      { department: "Sales", name: "C" },
    ]);
    const finance = groups.find((g) => g.department === "Finance");
    expect(finance.items).toHaveLength(2);
  });
});
