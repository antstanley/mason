// Mirrors mortar's serde output (server/src/model.rs)

export interface Author {
  did: string;
  handle: string;
  displayName: string | null;
  avatar: string | null;
}

interface AspectRatio {
  width: number;
  height: number;
}

interface ImageEmbed {
  src: string;
  alt: string;
  aspectRatio: AspectRatio | null;
}

interface ExternalEmbed {
  uri: string;
  title: string;
  description: string;
  thumb: string | null;
}

/** Present when a `!warn` label covers a brick's media behind a reveal. Absent
 *  (not null) when there is nothing to cover, since mortar skips serializing it. */
export interface Blur {
  label: string;
}

export interface PostBrick {
  kind: "post";
  id: string;
  url: string;
  author: Author;
  text: string;
  createdAt: string;
  likeCount: number;
  repostCount: number;
  images: ImageEmbed[];
  external: ExternalEmbed | null;
  blur?: Blur;
}

interface Publication {
  name: string;
  url: string;
  icon: string | null;
}

export interface BlogBrick {
  kind: "blog";
  id: string;
  url: string;
  author: Author;
  title: string;
  description: string | null;
  coverImage: string | null;
  publication: Publication;
  tags: string[];
  publishedAt: string;
}

export interface VideoBrick {
  kind: "video";
  id: string;
  url: string;
  author: Author;
  title: string;
  poster: string | null;
  playlist: string;
  aspectRatio: AspectRatio | null;
  source: "bluesky" | "streamplace";
  createdAt: string;
  likeCount: number;
  /** Streamplace only: this stream is happening right now. */
  live: boolean;
  viewerCount: number | null;
  durationMs: number | null;
  /** What the streamer says they are doing ("music", a game, ...). */
  activity: string | null;
  blur?: Blur;
}

export type Brick = PostBrick | BlogBrick | VideoBrick;

export interface FeedResponse {
  items: Brick[];
  cursor: string | null;
  /** Only present on a preview response: whether the wall is still warming, so
   *  the client keeps polling and reflowing the first screen until it settles. */
  warming?: boolean;
}
