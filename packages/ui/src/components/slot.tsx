import { Children, cloneElement, isValidElement, type ReactElement, type HTMLAttributes } from "react";

type SlotProps = HTMLAttributes<HTMLElement> & { children?: React.ReactNode };

/**
 * Slot سبک — جایگزین بدون‌وابستگیِ @radix-ui/react-slot.
 * کلاس و props والد را روی تنها فرزندِ element دار می‌نشیند.
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
