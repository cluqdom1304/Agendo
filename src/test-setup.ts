import { configure } from "@testing-library/svelte";

// The editor is loaded with a dynamic import: the first `findBy*` after a cold start can exceed the 1 s default under CPU load.
configure({ asyncUtilTimeout: 5_000 });
