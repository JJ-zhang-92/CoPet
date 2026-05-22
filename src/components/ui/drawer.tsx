import type { HTMLAttributes, ReactNode } from "react";
import { X } from "lucide-react";

import { cn } from "../../lib/utils";
import { Button } from "./button";

type DrawerProps = HTMLAttributes<HTMLDivElement> & {
  children: ReactNode;
  closeLabel?: string;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  overlayLabel?: string;
};

export function Drawer({
  children,
  className,
  closeLabel = "Close drawer",
  onOpenChange,
  open,
  overlayLabel = "Close drawer",
  ...props
}: DrawerProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="ui-drawer-root">
      <button
        aria-label={overlayLabel}
        className="ui-drawer-overlay"
        onClick={() => onOpenChange(false)}
        type="button"
      />
      <div
        aria-modal="true"
        className={cn("ui-drawer-content", className)}
        role="dialog"
        {...props}
      >
        <Button
          aria-label={closeLabel}
          className="ui-drawer-close"
          onClick={() => onOpenChange(false)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <X aria-hidden="true" />
        </Button>
        {children}
      </div>
    </div>
  );
}

export function DrawerHeader({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("ui-drawer-header", className)} {...props} />;
}

export function DrawerTitle({
  className,
  ...props
}: HTMLAttributes<HTMLHeadingElement>) {
  return <h2 className={cn("ui-drawer-title", className)} {...props} />;
}

export function DrawerDescription({
  className,
  ...props
}: HTMLAttributes<HTMLParagraphElement>) {
  return (
    <p className={cn("ui-drawer-description", className)} {...props} />
  );
}

export function DrawerBody({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("ui-drawer-body", className)} {...props} />;
}
