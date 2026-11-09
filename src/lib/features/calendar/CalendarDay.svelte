<script lang="ts">
  import type { Tag, Task } from "../../types";
  import CalendarWeek from "./CalendarWeek.svelte";

  interface Props {
    date: string;
    today: string;
    nowMinutes: number;
    tasksByDate: Map<string, Task[]>;
    label: string;
    onCreate: (
      title: string,
      date: string,
      time: string,
      duration?: number,
    ) => Promise<boolean>;
    onOpenTask: (task: Task) => void;
    tagOf?: (task: Task) => Tag | undefined;
    /** Pressed on an event: the parent starts dragging it towards a sidebar tag. */
    dragTask?: (event: PointerEvent, task: Task) => void;
  }

  let { date, ...rest }: Props = $props();
</script>

<!-- A day is the time grid with a single column. -->
<CalendarWeek dates={[date]} {...rest} />
