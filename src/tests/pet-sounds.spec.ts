import { expect, test } from "@playwright/test";

import { createAppHarness, pethoverWithSounds } from "./app-harness";

function soundState(responsePaused = false) {
  return {
    currentPetId: pethoverWithSounds.id,
    pets: [pethoverWithSounds],
    onboardingComplete: false,
    responsePaused,
    petInteractions: { enableClickSounds: true, cooldownStyle: "normal" as const },
  };
}

test("enabled interaction sound plays on successful click", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: soundState(),
  });
  const page = await harness.openPage("pet");

  await page.locator(".pet-sprite-frame").dispatchEvent("click", { button: 0, detail: 1 });

  await expect.poll(() => harness.playedAudioUrls(page)).toEqual([
    "/pets/pethover/pethover/audio/click.mp3",
  ]);
});

test("disabled pet sounds suppress interaction playback", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      ...soundState(),
      petInteractions: { enableClickSounds: false, cooldownStyle: "normal" },
    },
  });
  const page = await harness.openPage("pet");

  await page.locator(".pet-sprite-frame").dispatchEvent("click", { button: 0, detail: 1 });
  await page.waitForTimeout(100);

  expect(await harness.playedAudioUrls(page)).toEqual([]);
});

test("cooldown-suppressed gesture does not replay interaction sound", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: soundState(),
  });
  const page = await harness.openPage("pet");
  const spriteFrame = page.locator(".pet-sprite-frame");

  await spriteFrame.dispatchEvent("click", { button: 0, detail: 1 });
  await expect.poll(() => harness.playedAudioUrls(page)).toEqual([
    "/pets/pethover/pethover/audio/click.mp3",
  ]);

  await harness.clearPlayedAudioUrls(page);
  await spriteFrame.dispatchEvent("click", { button: 0, detail: 1 });
  await page.waitForTimeout(100);

  expect(await harness.playedAudioUrls(page)).toEqual([]);
});

test("agent state transition plays mapped agent sound once", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: soundState(),
  });
  const page = await harness.openPage("pet");
  await expect(page.locator(".pet-sprite")).toHaveAttribute("data-pet-state", "idle");

  await harness.emitRuntimeUpdate(page, {
    currentState: { state: "running" },
    messages: [{ agent: "codex", displayName: "Codex", text: "editing", updatedAtMs: 1 }],
  });
  await expect.poll(() => harness.playedAudioUrls(page)).toEqual([
    "/pets/pethover/pethover/audio/tap.mp3",
  ]);

  await harness.clearPlayedAudioUrls(page);
  await harness.emitRuntimeUpdate(page, {
    currentState: { state: "running" },
    messages: [{ agent: "codex", displayName: "Codex", text: "still editing", updatedAtMs: 2 }],
  });
  await page.waitForTimeout(100);

  expect(await harness.playedAudioUrls(page)).toEqual([]);
});

test("paused response updates do not play agent sounds", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: soundState(true),
  });
  const page = await harness.openPage("pet");
  await expect(page.locator(".pet-sprite")).toHaveAttribute("data-pet-state", "idle");

  await harness.emitRuntimeUpdate(page, {
    currentState: { state: "jumping" },
    messages: [{ agent: "codex", displayName: "Codex", text: "thinking", updatedAtMs: 1 }],
  });
  await page.waitForTimeout(100);

  expect(await harness.playedAudioUrls(page)).toEqual([]);
});
