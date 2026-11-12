// The pictures of the open notes (D-47): reads them once from the store, keeps an address for
// each and hands them to the editor as an `ImageHost`.
import type { Attachment } from "../../types";
import type { ImageHost } from "./attachmentImage";

export interface ImageApi {
  add(bytes: Uint8Array): Promise<Attachment>;
  read(id: string): Promise<Uint8Array>;
}

export interface UrlFactory {
  create(blob: Blob): string;
  revoke(url: string): void;
}

const browserUrls: UrlFactory = {
  create: (blob) => URL.createObjectURL(blob),
  revoke: (url) => URL.revokeObjectURL(url),
};

/** Types that may be stored; the backend checks the bytes again. */
export const IMAGE_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/gif",
  "image/webp",
]);

/** The images among `files`, in order. */
export function imageFiles(files: Iterable<File> | null | undefined): File[] {
  return Array.from(files ?? []).filter((file) => IMAGE_TYPES.has(file.type));
}

export class ImageCache implements ImageHost {
  // Bookkeeping, not UI state.
  readonly #urls = new Map<string, Promise<string | null>>();
  readonly #api: ImageApi;
  readonly #show: (id: string, mime: string, url: string) => void;
  readonly #failed: (message: string) => void;
  readonly #factory: UrlFactory;

  constructor(options: {
    api: ImageApi;
    /** Shows a picture large, once its address is known. */
    show: (id: string, mime: string, url: string) => void;
    /** Reports a file that could not be stored. */
    failed: (message: string) => void;
    urls?: UrlFactory;
  }) {
    this.#api = options.api;
    this.#show = options.show;
    this.#failed = options.failed;
    this.#factory = options.urls ?? browserUrls;
  }

  url(id: string, mime: string): Promise<string | null> {
    let found = this.#urls.get(id);
    if (!found) {
      found = this.#api
        .read(id)
        .then((bytes) =>
          this.#factory.create(new Blob([bytes as BlobPart], { type: mime })),
        )
        .catch(() => null);
      this.#urls.set(id, found);
    }
    return found;
  }

  async add(file: File): Promise<{ id: string; mime: string } | null> {
    try {
      const stored = await this.#api.add(
        new Uint8Array(await file.arrayBuffer()),
      );
      return { id: stored.id, mime: stored.mime };
    } catch (error) {
      this.#failed(typeof error === "string" ? error : "");
      return null;
    }
  }

  preview(id: string, mime: string): void {
    void this.url(id, mime).then((url) => {
      if (url) this.#show(id, mime, url);
    });
  }

  dispose(): void {
    for (const pending of this.#urls.values()) {
      void pending.then((url) => url && this.#factory.revoke(url));
    }
    this.#urls.clear();
  }
}
