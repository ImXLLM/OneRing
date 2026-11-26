import { Children, cloneElement, isValidElement, type ReactElement, type HTMLAttributes } from "react";

type SlotProps = HTMLAttributes<HTMLElement> & { children?: React.ReactNode };

/**
 * Lightweight Slot — a dependency-free replacement for @radix-ui/react-slot.
 * Merges the parent's className and props onto its single element child.
 */
export function Slot({ children, className, ...props }: SlotProps) {
  const child = Children.only(children);
  if (!isValidElement(child)) return null;
  const childProps = child.props as { className?: string };
  return cloneElement(child as ReactElement<{ className?: string }>, {
    ...props,
    className: className ? `${className} ${childProps.className ?? ""}` : childProps.className,
  });
}
