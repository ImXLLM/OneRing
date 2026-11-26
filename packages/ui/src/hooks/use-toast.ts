"use client";

import { useSyncExternalStore } from "react";

export type ToastVariant = "default" | "success" | "destructive" | "gold";

export interface ToastItem {
  id: number;
  title: string;
  description?: string;
  variant?: ToastVariant;
}

let items: ToastItem[] = [];
let counter = 0;
const listeners = new Set<() => void>();

function emit() {
  for (const l of listeners) l();
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot() {
  return items;
}

/** Imperative call — from anywhere in the client: toast({ title: "...", variant: "success" }) */
export function toast(input: Omit<ToastItem, "id">) {
  const id = ++counter;
  items = [...items, { id, ...input }];
  emit();
  const AUTO_DISMISS_MS = 4500;
  setTimeout(() => dismiss(id), AUTO_DISMISS_MS);
  return id;
}

export function dismiss(id: number) {
  items = items.filter((t) => t.id !== id);
  emit();
}

export function useToast() {
  const toasts = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  return { toasts, dismiss };
}
