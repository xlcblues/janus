import { CheckCircle2 } from "lucide-react";

interface ToastProps {
  message: string;
}

export function Toast({ message }: ToastProps) {
  return (
    <div className="toast" role="status" aria-live="polite">
      <CheckCircle2 size={17} aria-hidden="true" />
      <span>{message}</span>
    </div>
  );
}
