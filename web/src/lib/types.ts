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

interface GameInfo {
  appid: number;
  name: string;
  headerImage: string | null;
}

export interface VideoBrick {
  kind: "video";
  id: string;
  url: string;
  author: Author | null;
  title: string;
  poster: string | null;
  playlist: string;
  aspectRatio: AspectRatio | null;
  source: "bluesky" | "steam";
  game: GameInfo | null;
  createdAt: string;
  likeCount: number;
}

export type Brick = PostBrick | BlogBrick | VideoBrick;

export interface FeedResponse {
  items: Brick[];
  cursor: string | null;
}
