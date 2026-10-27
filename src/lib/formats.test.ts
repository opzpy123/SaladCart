import { describe, expect, it } from "vitest";
import { formatDuration, formatTimecode, relativeTime } from "./bridge";
import { mockInvoke } from "./mock";

describe("time formatting", () => {
  it("formats durations", () => {
    expect(formatDuration(0)).toBe("0s");
    expect(formatDuration(5_000)).toBe("5s");
    expect(formatDuration(65_000)).toBe("1:05");
  });

  it("formats timecode at 30fps", () => {
    expect(formatTimecode(0)).toBe("00:00:00");
    expect(formatTimecode(65_500)).toBe("01:05:15");
  });

  it("formats relative dates", () => {
    expect(relativeTime(new Date().toISOString())).toBe("刚刚");
  });
});

describe("browser demo bridge", () => {
  it("exposes a project with a complete production graph", async () => {
    const projects = await mockInvoke<Array<{ id: string; name: string }>>("list_projects");
    expect(projects.length).toBeGreaterThan(0);
    const snapshot = await mockInvoke<{
      characters: unknown[];
      episodes: Array<{ scenes: Array<{ shots: unknown[] }> }>;
      assets: unknown[];
      timeline: { clips: unknown[]; musicAssetId: string | null };
    }>("open_project", { projectId: projects[0].id });
    expect(snapshot.characters.length).toBeGreaterThan(0);
    expect(snapshot.episodes[0].scenes[0].shots.length).toBeGreaterThan(0);
    expect(snapshot.assets.length).toBeGreaterThan(0);
    expect(snapshot.timeline.clips.length).toBeGreaterThan(0);
    expect(snapshot.timeline.musicAssetId).toBeTruthy();
  });

  it("compiles context and searches licensed music", async () => {
    const projects = await mockInvoke<Array<{ id: string }>>("list_projects");
    const snapshot = await mockInvoke<{
      episodes: Array<{ scenes: Array<{ shots: Array<{ id: string }> }> }>;
    }>("open_project", { projectId: projects[0].id });
    const shotId = snapshot.episodes[0].scenes[0].shots[0].id;
    const context = await mockInvoke<{ userPrompt: string; characterRefs: unknown[] }>(
      "get_context_preview",
      {
        projectId: projects[0].id,
        request: {
          kind: "video",
          targetType: "shot",
          targetId: shotId,
          options: { useContinuity: true },
        },
      },
    );
    expect(context.userPrompt).toContain("PROJECT STYLE");
    expect(context.characterRefs.length).toBeGreaterThan(0);

    const result = await mockInvoke<{ tracks: Array<{ license: string }> }>("search_music", {
      providerId: "jamendo-music",
      query: "rain",
      limit: 4,
    });
    expect(result.tracks.length).toBeGreaterThan(0);
    expect(result.tracks[0].license).toBeTruthy();
  });

  it("analyzes beats and snaps cuts", async () => {
    const projects = await mockInvoke<Array<{ id: string }>>("list_projects");
    const beat = await mockInvoke<{ bpm: number; beatGridMs: number[] }>("analyze_music_beats", {
      projectId: projects[0].id,
    });
    expect(beat.bpm).toBe(100);
    expect(beat.beatGridMs.length).toBeGreaterThan(10);
    const synced = await mockInvoke<{ alignedCuts: number; totalCuts: number }>("auto_beat_sync", {
      projectId: projects[0].id,
    });
    expect(synced.totalCuts).toBeGreaterThan(0);
    expect(synced.alignedCuts).toBeGreaterThan(0);
  });
});
