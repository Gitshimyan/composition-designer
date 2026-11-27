import { useMemo, useState } from "react";
import type {
  Binding,
  EditCommand,
  Instance,
  ModuleDefinition,
  ReviewOptions,
} from "../types";
import {
  bindingParentOptions,
  moduleKey,
  moveBindingCommands,
  slotChildren,
  slotOccupancy,
} from "./instanceEditing";

interface BindingEditorDialogProps {
  binding: Binding;
  child: Instance;
  instances: Instance[];
  bindings: Binding[];
  modules: ModuleDefinition[];
  suspended?: boolean;
  onCancel: () => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

export default function BindingEditorDialog({
  binding,
  child,
  instances,
  bindings,
  modules,
  suspended = false,
  onCancel,
  onReview,
}: BindingEditorDialogProps) {
  const options = useMemo(
    () => bindingParentOptions(instances, bindings, modules, child),
    [instances, bindings, modules, child],
  );
  const initialParentId =
    options.find((option) => option.instance.id === binding.parent)?.instance
      .id ??
    options[0]?.instance.id ??
    binding.parent;
  const initialSlots =
    options.find((option) => option.instance.id === initialParentId)?.slots ??
    [];
  const initialSlotId = initialSlots.some((slot) => slot.id === binding.slot)
    ? binding.slot
    : initialSlots[0]?.id ?? binding.slot;

  const [parentId, setParentId] = useState(initialParentId);
  const [slotId, setSlotId] = useState(initialSlotId);
  const [localError, setLocalError] = useState<string | null>(null);

  const parentOption = options.find(
    (option) => option.instance.id === parentId,
  );
  const slots = parentOption?.slots ?? [];
  const activeSlot = slots.find((slot) => slot.id === slotId) ?? slots[0];

  function handleParentChange(nextParentId: string) {
    setParentId(nextParentId);
    const nextSlots =
      options.find((option) => option.instance.id === nextParentId)?.slots ??
      [];
    if (!nextSlots.some((slot) => slot.id === slotId)) {
      setSlotId(nextSlots[0]?.id ?? "");
    }
    setLocalError(null);
  }

  function submit() {
    if (!activeSlot) {
      setLocalError("No Slot of the selected parent accepts this module.");
      return;
    }
    if (parentId === binding.parent && activeSlot.id === binding.slot) {
      setLocalError("Choose a different parent or slot.");
      return;
    }
    setLocalError(null);
    onReview(
      moveBindingCommands(binding.id, parentId, activeSlot.id, child.id),
      `Move ${child.id}: ${binding.parent}.${binding.slot} → ${parentId}.${activeSlot.id}`,
      { selectAfter: child.id, focusAfter: true },
    );
  }

  return (
    <div
      className={`dialog-veil ${suspended ? "is-suspended" : ""}`}
      aria-hidden={suspended || undefined}
    >
      <section
        className="dialog-card dialog-card-wide"
        role="dialog"
        aria-modal="true"
        aria-labelledby="binding-editor-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">Binding editor</div>
            <h2 id="binding-editor-title">Move {child.id}</h2>
          </div>
          <button
            type="button"
            className="dialog-close"
            aria-label="Close"
            onClick={onCancel}
          >
            ×
          </button>
        </div>

        <div className="dialog-slot-meta">
          <span>binding {binding.id}</span>
          <span>
            currently {binding.parent}.{binding.slot}
          </span>
          <span>child stays fixed</span>
        </div>

        <div className="dialog-fields">
          <label htmlFor="binding-parent">Parent instance</label>
          <select
            id="binding-parent"
            value={parentId}
            onChange={(event) => handleParentChange(event.target.value)}
          >
            {options.map((option) => (
              <option key={option.instance.id} value={option.instance.id}>
                {option.instance.id} · {moduleKey(option.module)}
              </option>
            ))}
            {options.length === 0 && (
              <option value={binding.parent}>{binding.parent}</option>
            )}
          </select>
        </div>

        {slots.length > 0 ? (
          <div className="slot-choice-list">
            <div className="section-label">Target Slot</div>
            {slots.map((slot) => {
              const count = slotOccupancy(bindings, parentId, slot.id);
              const children = slotChildren(bindings, parentId, slot.id);
              const full =
                slot.cardinality !== "many" && count >= 1;
              return (
                <label className="slot-choice" key={slot.id}>
                  <input
                    type="radio"
                    name="binding-slot"
                    value={slot.id}
                    checked={activeSlot?.id === slot.id}
                    onChange={() => {
                      setSlotId(slot.id);
                      setLocalError(null);
                    }}
                  />
                  <span className="slot-choice-body">
                    <span className="slot-choice-head">
                      <code>{slot.id}</code>
                      <span className={`slot-cardinality is-${slot.cardinality}`}>
                        {slot.cardinality}
                      </span>
                      <span className="slot-count">{count} bound</span>
                      {full && <span className="slot-full">full</span>}
                    </span>
                    <span className="slot-accepts">
                      accepts {slot.accepts.join(", ")}
                      {children.length > 0 ? ` · ${children.join(", ")}` : ""}
                    </span>
                  </span>
                </label>
              );
            })}
          </div>
        ) : (
          <div className="dialog-note">
            No parent module has a Slot that accepts {child.module.id}.
          </div>
        )}

        {localError && <div className="config-error">{localError}</div>}

        <div className="dialog-actions">
          <button type="button" className="button button-ghost" onClick={onCancel}>
            Cancel
          </button>
          <button
            type="button"
            className="button button-primary"
            disabled={!activeSlot || options.length === 0}
            onClick={submit}
          >
            Review move
          </button>
        </div>
        <p className="dialog-note">
          Slot candidates are filtered by the module contract. Core revalidates
          acceptance, cardinality, cycles and required structure before commit.
        </p>
      </section>
    </div>
  );
}
