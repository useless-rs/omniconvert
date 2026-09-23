export type JobState = "queued" | "running" | "done" | "failed" | "cancelled";

export interface Job {
  id: number;
  file: File | { name: string; path?: string };
  target: string;
  state: JobState;
  progress: number;
  message: string;
  cancelled: boolean;
}

let nextId = 1;
export function makeJob(file: File | { name: string; path?: string }, target: string): Job {
  return { id: nextId++, file, target, state: "queued", progress: 0, message: "", cancelled: false };
}
