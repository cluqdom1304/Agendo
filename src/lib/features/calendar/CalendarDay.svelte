<script lang="ts">
  import type { Task } from "../../types";
  import CalendarWeek from "./CalendarWeek.svelte";

  interface Props {
    date: string;
    today: string;
    tasksByDate: Map<string, Task[]>;
    label: string;
    onCreate: (title: string, date: string, time: string) => Promise<boolean>;
    onOpenTask: (task: Task) => void;
  }

  let { date, ...rest }: Props = $props();
</script>

<!-- A day is the time grid with a single column. -->
<CalendarWeek dates={[date]} {...rest} />
