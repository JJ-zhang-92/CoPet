import { expect, test } from "@playwright/test";

import { createAppHarness, pethover } from "./app-harness";

test("pet window suppresses agent messages while responsePaused is true", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: pethover.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [pethover],
      onboardingComplete: false,
      petWindowSize: 30,
      responsePaused: true,
    },
  });

  const petPage = await harness.openPage("pet");
  // Wait for the initial render to settle.
  await expect(petPage.locator(".pet-window-stack")).toBeVisible();
  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(0);

  // Emit a runtime update while paused — must be ignored.
  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __pethoverTestEmit: (e: string, p: unknown) => void })
      .__pethoverTestEmit(event, payload);
  }, {
    event: "pet-state-changed",
    payload: {
      currentState: { state: "running", sinceMs: 1000, idleAfterMs: null },
      messages: [
        {
          agent: "claude",
          displayName: "Claude",
          text: "thinking",
          updatedAtMs: 1000,
        },
      ],
    },
  });

  // Allow any propagation to flush. The message bubble must NOT appear.
  await petPage.waitForTimeout(150);
  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(0);

  // Unpause via app-state-changed, then emit another runtime update — must render now.
  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __pethoverTestEmit: (e: string, p: unknown) => void })
      .__pethoverTestEmit(event, payload);
  }, {
    event: "pethover-app-state-changed",
    payload: {
      currentPetId: pethover.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [pethover],
      onboardingComplete: false,
      petWindowSize: 30,
      responsePaused: false,
    },
  });

  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __pethoverTestEmit: (e: string, p: unknown) => void })
      .__pethoverTestEmit(event, payload);
  }, {
    event: "pet-state-changed",
    payload: {
      currentState: { state: "running", sinceMs: 2000, idleAfterMs: null },
      messages: [
        {
          agent: "claude",
          displayName: "Claude",
          text: "thinking",
          updatedAtMs: 2000,
        },
      ],
    },
  });

  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(1);
  await expect(petPage.locator('.pet-agent-text')).toHaveText("thinking");
});

test("pausing messages hides already visible pet message bubbles", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: pethover.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [pethover],
      onboardingComplete: false,
      petWindowSize: 30,
      responsePaused: false,
    },
  });

  const petPage = await harness.openPage("pet");
  await expect(petPage.locator(".pet-window-stack")).toBeVisible();

  await harness.emitRuntimeUpdate(petPage, {
    currentState: { state: "running", sinceMs: 1000, idleAfterMs: null },
    messages: [
      {
        agent: "codex",
        displayName: "Codex",
        text: "running tests",
        updatedAtMs: 1000,
      },
    ],
  });

  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(1);

  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __pethoverTestEmit: (e: string, p: unknown) => void })
      .__pethoverTestEmit(event, payload);
  }, {
    event: "pethover-app-state-changed",
    payload: {
      currentPetId: pethover.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [pethover],
      onboardingComplete: false,
      petWindowSize: 30,
      responsePaused: true,
    },
  });

  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(0);
});
