/** Single source of truth for wall column count, measured on the CONTAINER
 *  (not the viewport) so skeletons and laid bricks always agree. */
export function colsForWidth(width: number): number {
  // a phone should still see a WALL — one-column is the doomscroll feed we
  // positioned against
  if (width < 340) return 1;
  if (width < 640) return 2;
  if (width < 1024) return 3;
  if (width < 1392) return 4;
  return 5;
}
