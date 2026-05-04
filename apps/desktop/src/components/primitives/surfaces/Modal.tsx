import { useEffect, useRef } from "react";
import type { KeyboardEvent, ReactNode } from "react";

export type ModalProps = {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
};

function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector =
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  return Array.from(container.querySelectorAll<HTMLElement>(selector)).filter(
    (element) => !element.hasAttribute("disabled") && element.getAttribute("aria-hidden") !== "true"
  );
}

export function Modal({ open, title, onClose, children }: ModalProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open || !dialogRef.current) {
      return;
    }
    const focusables = getFocusableElements(dialogRef.current);
    if (focusables.length > 0) {
      focusables[0].focus();
    }
  }, [open]);

  if (!open) {
    return null;
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!dialogRef.current) {
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key !== "Tab") {
      return;
    }

    const focusables = getFocusableElements(dialogRef.current);
    if (focusables.length === 0) {
      return;
    }

    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    const active = document.activeElement;

    if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
      return;
    }
    if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  };

  return (
    <div className="kp-modal-overlay" role="presentation" onClick={onClose}>
      <div
        className="kp-modal-panel"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={handleKeyDown}
        ref={dialogRef}
      >
        <header className="kp-modal-header">
          <h2 className="kp-modal-title">{title}</h2>
          <button
            type="button"
            className="kp-modal-close kp-primitive-focus"
            aria-label="Close modal"
            onClick={onClose}
          >
            x
          </button>
        </header>
        <div className="kp-modal-body">{children}</div>
      </div>
    </div>
  );
}
