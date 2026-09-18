/** Helpers for building agent input payloads from manifest input schemas. */

export function defaultInputFromSchema(fields = []) {
  const input = {};
  fields.forEach((field) => {
    if (field.example == null) return;
    if (field.kind === "number") {
      input[field.key] = Number(field.example);
    } else if (field.kind === "json") {
      try {
        input[field.key] = JSON.parse(field.example);
      } catch {
        input[field.key] = field.example;
      }
    } else {
      input[field.key] = field.example;
    }
  });
  return input;
}

export function missingRequiredFields(fields = [], value = {}) {
  return fields
    .filter((field) => field.required)
    .filter((field) => {
      const v = value[field.key];
      return v === undefined || v === null || v === "";
    })
    .map((field) => field.key);
}

export function groupByDepartment(agents = []) {
  const map = new Map();
  agents.forEach((agent) => {
    const list = map.get(agent.department) || [];
    list.push(agent);
    map.set(agent.department, list);
  });
  return Array.from(map.entries()).map(([department, items]) => ({
    department,
    items,
  }));
}
