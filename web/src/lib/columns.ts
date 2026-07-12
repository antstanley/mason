/** Single source of truth for wall column count, measured on the CONTAINER
 *  (not the viewport) so skeletons and laid bricks always agree. */
export function colsForWidth(width: number): number {
  if (width < 592) return 1;
  if (width < 976) return 2;
  if (width < 1392) return 3;
  return 4;
}
