import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { I18nProvider } from "../../app/I18nProvider";
import { AgentInputForm } from "./AgentInputForm";

const FIELDS = [
  { key: "products", label: "Products", kind: "file", required: true, example: "a,b\n1,2" },
  { key: "threshold", label: "Threshold", kind: "number", required: false, example: "10" },
  { key: "notes", label: "Notes", kind: "text", required: false },
];

function renderForm(props = {}) {
  return render(
    <I18nProvider>
      <AgentInputForm fields={FIELDS} value={{}} onChange={() => {}} mode="form" setMode={() => {}} {...props} />
    </I18nProvider>
  );
}

describe("AgentInputForm", () => {
  it("renders a control per input field", () => {
    renderForm();
    expect(screen.getByText(/Products/)).toBeInTheDocument();
    expect(screen.getByText(/Threshold/)).toBeInTheDocument();
    expect(screen.getByText(/Notes/)).toBeInTheDocument();
    expect(screen.getByRole("spinbutton")).toBeInTheDocument();
  });

  it("loads the example value for a numeric field", () => {
    const onChange = vi.fn();
    renderForm({ onChange });
    const loadButtons = screen.getAllByText(/Örnek yükle|Load example/i);
    // click the example button for the numeric field (second field)
    fireEvent.click(loadButtons[1]);
    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ threshold: 10 }));
  });

  it("switches to the advanced JSON editor", () => {
    const setMode = vi.fn();
    renderForm({ setMode });
    fireEvent.click(screen.getByText(/Gelişmiş|Advanced/i));
    expect(setMode).toHaveBeenCalledWith("advanced");
  });
});
