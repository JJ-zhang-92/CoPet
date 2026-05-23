import { expect, test } from "@playwright/test";

import { createAppHarness, copet } from "./app-harness";

test("pet window hides agent messages when agentMessageVisible is false", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: false,
    },
  });

  const petPage = await harness.openPage("pet");
  // Wait for the initial render to settle.
  await expect(petPage.locator(".pet-window-stack")).toBeVisible();
  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(0);

  // Emit a runtime update while messages are hidden — the bubble must stay hidden.
  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __copetTestEmit: (e: string, p: unknown) => void })
      .__copetTestEmit(event, payload);
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

  // Show messages via app-state-changed, then emit another runtime update — must render now.
  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __copetTestEmit: (e: string, p: unknown) => void })
      .__copetTestEmit(event, payload);
  }, {
    event: "copet-app-state-changed",
    payload: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: true,
    },
  });

  await petPage.evaluate(({ event, payload }) => {
    (window as unknown as { __copetTestEmit: (e: string, p: unknown) => void })
      .__copetTestEmit(event, payload);
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

test("hiding messages removes already visible pet message bubbles immediately", async ({ browser }) => {
  const harness = await createAppHarness(browser, {
    state: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: true,
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
    (window as unknown as { __copetTestEmit: (e: string, p: unknown) => void })
      .__copetTestEmit(event, payload);
  }, {
    event: "copet-app-state-changed",
    payload: {
      currentPetId: copet.id,
      locale: "en-US",
      localePreference: "en-US",
      pets: [copet],
      onboardingComplete: false,
      petWindowSize: 30,
      agentMessageVisible: false,
    },
  });

  await expect(petPage.locator('[data-testid="pet-agent-message"]')).toHaveCount(0);
});
