// The arithmetic of KitZoomCanvas, pure so it is tested without a browser. A view is a scale and
// the offset of the content's top-left corner in the viewport, in px.

export interface View {
  scale: number;
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface ZoomLimits {
  min: number;
  max: number;
}

export function clamp(n: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, n));
}

/** The scale at which the whole content fits the viewport inside `pad` px. */
export function fitScale(viewport: Size, content: Size, pad: number): number {
  if (content.width <= 0 || content.height <= 0) return 1;
  const w = Math.max(1, viewport.width - 2 * pad);
  const h = Math.max(1, viewport.height - 2 * pad);
  return Math.min(w / content.width, h / content.height);
}

/** Centres the content on an axis where it is smaller than the viewport; else `pad` from the start. */
export function place(viewport: Size, content: Size, scale: number, pad: number): View {
  const w = content.width * scale;
  const h = content.height * scale;
  return {
    scale,
    x: w + 2 * pad <= viewport.width ? (viewport.width - w) / 2 : pad,
    y: h + 2 * pad <= viewport.height ? (viewport.height - h) / 2 : pad,
  };
}

/**
 * The view a canvas opens at: the whole content when it fits at a readable scale (never larger
 * than 100 %); otherwise the readable scale, from the top, centred when it is narrower.
 */
export function initialView(viewport: Size, content: Size, readable: number, pad: number): View {
  const fit = fitScale(viewport, content, pad);
  const scale = fit >= readable ? Math.min(1, fit) : readable;
  return place(viewport, content, scale, pad);
}

/** The view fitted to the viewport, within the limits. */
export function fitView(viewport: Size, content: Size, limits: ZoomLimits, pad: number): View {
  return place(viewport, content, clamp(fitScale(viewport, content, pad), limits.min, limits.max), pad);
}

/** Zooms by `factor` about the viewport point (px, py), which stays where it is. */
export function zoomAt(view: View, factor: number, px: number, py: number, limits: ZoomLimits): View {
  const scale = clamp(view.scale * factor, limits.min, limits.max);
  const k = scale / view.scale;
  return { scale, x: px - (px - view.x) * k, y: py - (py - view.y) * k };
}

/** The factor one wheel event zooms by: a pinch (ctrlKey) is finer than a wheel. */
export function wheelFactor(deltaY: number, deltaMode: number, pinch: boolean): number {
  const px = deltaMode === 1 ? deltaY * 16 : deltaMode === 2 ? deltaY * 400 : deltaY;
  return Math.exp(-px * (pinch ? 0.01 : 0.0015));
}
