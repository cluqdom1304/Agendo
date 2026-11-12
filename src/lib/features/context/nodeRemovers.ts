// How to delete the node an editor draws for a link or a task reference: the node views
// register it, and the context menu (which only sees the DOM) calls it.

export const nodeRemovers = new WeakMap<Element, () => void>();
