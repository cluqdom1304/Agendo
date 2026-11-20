<script lang="ts">
  import type { Capture } from "../../naturalInput";
  import type { EventOccurrence, Tag, Task } from "../../types";
  import CalendarWeek from "./CalendarWeek.svelte";

  interface Props {
    date: string;
    today: string;
    nowMinutes: number;
    tasksByDate: Map<string, Task[]>;
    eventsByDate?: Map<string, EventOccurrence[]>;
    label: string;
    onCreate: (
      title: string,
      date: string,
      time: string,
      duration?: number,
      capture?: Capture,
    ) => Promise<boolean>;
    onOpenTask: (task: Task) => void;
    onOpenEvent?: (eventId: string) => void;
    onMoveEvent?: (
      occurrence: EventOccurrence,
      date: string,
      start: number,
    ) => void;
    onResizeEvent?: (occurrence: EventOccurrence, end: number) => void;
    onMoveTask?: (task: Task, date: string, start: number) => void;
    onResizeTask?: (task: Task, end: number) => void;
    tagOf?: (item: Pick<Task, "tagIds">) => Tag | undefined;
    moreTags?: (task: Task) => number;
    /** Pressed on an event: the parent starts dragging it towards a sidebar tag. */
    dragTask?: (event: PointerEvent, task: Task) => void;
  }

  let { date, ...rest }: Props = $props();
</script>

<!-- A day is the time grid with a single column. -->
<CalendarWeek dates={[date]} {...rest} />
